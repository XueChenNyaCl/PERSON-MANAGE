use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::client::RedisClient;
use super::error::{RedisError, Result};
use crate::core::config::BufferConfig;

const BUFFER_QUEUE_KEY: &str = "buffer:writes";
const BUFFER_DEAD_LETTER_KEY: &str = "buffer:dead_letter";
const MAX_RETRY_COUNT: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WriteOperation {
    Insert,
    Update { id: String },
    Delete { id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferedWrite {
    pub id: Uuid,
    pub operation: WriteOperation,
    pub table: String,
    pub data: Value,
    pub timestamp: DateTime<Utc>,
    pub retry_count: u32,
}

impl BufferedWrite {
    pub fn new_insert(table: &str, data: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            operation: WriteOperation::Insert,
            table: table.to_string(),
            data,
            timestamp: Utc::now(),
            retry_count: 0,
        }
    }

    pub fn new_update(table: &str, id: &str, data: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            operation: WriteOperation::Update { id: id.to_string() },
            table: table.to_string(),
            data,
            timestamp: Utc::now(),
            retry_count: 0,
        }
    }

    pub fn new_delete(table: &str, id: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            operation: WriteOperation::Delete { id: id.to_string() },
            table: table.to_string(),
            data: Value::Null,
            timestamp: Utc::now(),
            retry_count: 0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub queue_length: usize,
    pub dead_letter_length: usize,
    pub last_flush_time: Option<DateTime<Utc>>,
    pub total_flushed: u64,
    pub total_failed: u64,
}

pub struct WriteBuffer {
    redis_client: RedisClient,
    pg_pool: Option<PgPool>,
    config: BufferConfig,
    stats: Arc<Mutex<BufferStats>>,
}

impl WriteBuffer {
    pub fn new(redis_client: RedisClient, pg_pool: Option<PgPool>, config: BufferConfig) -> Self {
        Self {
            redis_client,
            pg_pool,
            config,
            stats: Arc::new(Mutex::new(BufferStats {
                queue_length: 0,
                dead_letter_length: 0,
                last_flush_time: None,
                total_flushed: 0,
                total_failed: 0,
            })),
        }
    }

    pub async fn buffer_insert(&self, table: &str, data: Value) -> Result<()> {
        let write = BufferedWrite::new_insert(table, data);
        self.push_to_buffer(&write).await
    }

    pub async fn buffer_update(&self, table: &str, id: &str, data: Value) -> Result<()> {
        let write = BufferedWrite::new_update(table, id, data);
        self.push_to_buffer(&write).await
    }

    pub async fn buffer_delete(&self, table: &str, id: &str) -> Result<()> {
        let write = BufferedWrite::new_delete(table, id);
        self.push_to_buffer(&write).await
    }

    async fn push_to_buffer(&self, write: &BufferedWrite) -> Result<()> {
        let queue_len = self.redis_client.llen(BUFFER_QUEUE_KEY).await?;

        if queue_len >= self.config.max_size {
            return Err(RedisError::BufferFull);
        }

        let json = serde_json::to_string(write)?;
        self.redis_client.lpush(BUFFER_QUEUE_KEY, &json).await?;

        debug!(
            "Buffered write queued: {} {:?} (queue length: {})",
            write.table, write.operation, queue_len
        );

        Ok(())
    }

    pub async fn flush(&self) -> Result<FlushResult> {
        let Some(pg_pool) = &self.pg_pool else {
            return Err(RedisError::OperationError(
                "PostgreSQL pool not available".to_string(),
            ));
        };

        let mut writes = Vec::new();
        let batch_size = self.config.batch_size;

        for _ in 0..batch_size {
            match self.redis_client.rpop(BUFFER_QUEUE_KEY).await? {
                Some(json) => match serde_json::from_str::<BufferedWrite>(&json) {
                    Ok(write) => writes.push(write),
                    Err(e) => {
                        error!("Failed to deserialize buffered write: {}", e);
                    }
                },
                None => break,
            }
        }

        if writes.is_empty() {
            return Ok(FlushResult {
                processed: 0,
                succeeded: 0,
                failed: 0,
            });
        }

        info!("Flushing {} buffered writes to PostgreSQL", writes.len());

        let mut succeeded = 0;
        let mut failed = 0;

        let mut by_table: HashMap<String, Vec<BufferedWrite>> = HashMap::new();
        for write in writes {
            by_table.entry(write.table.clone()).or_default().push(write);
        }

        for (table, table_writes) in by_table {
            for write in table_writes {
                match self.execute_write(pg_pool, &write).await {
                    Ok(_) => {
                        succeeded += 1;
                        debug!("Buffered write succeeded: {} {:?}", table, write.operation);
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        error!(
                            "Buffered write failed: {} {:?} - {}",
                            table, write.operation, error_msg
                        );
                        failed += 1;

                        // 检查是否是"not found"类错误，这类错误重试无意义
                        let is_not_found_error = error_msg.contains("not found")
                            || error_msg.contains("不存在")
                            || error_msg.contains("NotFound");

                        if is_not_found_error {
                            // 直接移到死信队列，不再重试
                            warn!(
                                "Write failed with 'not found' error, moving to dead letter immediately: {} {:?}",
                                table, write.operation
                            );
                            self.move_to_dead_letter(&write).await?;
                        } else if write.retry_count < MAX_RETRY_COUNT {
                            let mut retry_write = write.clone();
                            retry_write.retry_count += 1;
                            if let Err(e) = self.push_to_buffer(&retry_write).await {
                                error!("Failed to requeue write: {}", e);
                                self.move_to_dead_letter(&write).await?;
                            }
                        } else {
                            self.move_to_dead_letter(&write).await?;
                        }
                    }
                }
            }
        }

        let mut stats = self.stats.lock().await;
        stats.last_flush_time = Some(Utc::now());
        stats.total_flushed += succeeded as u64;
        stats.total_failed += failed as u64;
        drop(stats);

        info!(
            "Flush completed: {} succeeded, {} failed",
            succeeded, failed
        );

        Ok(FlushResult {
            processed: succeeded + failed,
            succeeded,
            failed,
        })
    }

    async fn execute_write(&self, pool: &PgPool, write: &BufferedWrite) -> anyhow::Result<()> {
        // 特殊处理 persons 表的操作（人员创建、更新、删除）
        if write.table == "persons" {
            match &write.operation {
                WriteOperation::Insert => {
                    return self.execute_person_create(pool, &write.data).await;
                }
                WriteOperation::Update { id } => {
                    return self.execute_person_update(pool, id, &write.data).await;
                }
                WriteOperation::Delete { id } => {
                    return self.execute_person_delete(pool, id).await;
                }
            }
        }

        // 特殊处理 classes 表的操作（班级创建、更新、删除）
        if write.table == "classes" {
            match &write.operation {
                WriteOperation::Insert => {
                    return self.execute_class_create(pool, &write.data).await;
                }
                WriteOperation::Update { id } => {
                    return self.execute_class_update(pool, id, &write.data).await;
                }
                WriteOperation::Delete { id } => {
                    return self.execute_class_delete(pool, id).await;
                }
            }
        }

        // 特殊处理 departments 表的操作（部门创建、更新、删除）
        if write.table == "departments" {
            match &write.operation {
                WriteOperation::Insert => {
                    return self.execute_department_create(pool, &write.data).await;
                }
                WriteOperation::Update { id } => {
                    return self.execute_department_update(pool, id, &write.data).await;
                }
                WriteOperation::Delete { id } => {
                    return self.execute_department_delete(pool, id).await;
                }
            }
        }

        // 特殊处理 chat_messages 表的操作
        if write.table == "chat_messages" {
            match &write.operation {
                WriteOperation::Insert => {
                    return self.execute_chat_message_create(pool, &write.data).await;
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Chat messages only support insert operation"
                    ));
                }
            }
        }

        match &write.operation {
            WriteOperation::Insert => {
                let data = write
                    .data
                    .as_object()
                    .ok_or_else(|| anyhow::anyhow!("Insert data must be an object"))?;

                let columns: Vec<String> = data.keys().cloned().collect();
                let values: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();

                let query = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    write.table,
                    columns.join(", "),
                    values.join(", ")
                );

                let mut sql_query = sqlx::query(&query);
                for value in data.values() {
                    sql_query = sql_query.bind(value.to_string());
                }

                sql_query.execute(pool).await?;
            }
            WriteOperation::Update { id } => {
                let data = write
                    .data
                    .as_object()
                    .ok_or_else(|| anyhow::anyhow!("Update data must be an object"))?;

                let sets: Vec<String> = data
                    .keys()
                    .enumerate()
                    .map(|(i, col)| format!("{} = ${}", col, i + 1))
                    .collect();

                let query = format!(
                    "UPDATE {} SET {} WHERE id = ${}",
                    write.table,
                    sets.join(", "),
                    sets.len() + 1
                );

                let mut sql_query = sqlx::query(&query);
                for value in data.values() {
                    sql_query = sql_query.bind(value.to_string());
                }
                sql_query = sql_query.bind(id);

                sql_query.execute(pool).await?;
            }
            WriteOperation::Delete { id } => {
                let query = format!("DELETE FROM {} WHERE id = $1", write.table);
                sqlx::query(&query).bind(id).execute(pool).await?;
            }
        }

        Ok(())
    }

    /// 执行人员创建（处理多表事务）
    async fn execute_person_create(&self, pool: &PgPool, data: &Value) -> anyhow::Result<()> {
        use crate::core::password::hash_password;
        use crate::models::person::PersonCreate;
        use uuid::Uuid;

        let payload: PersonCreate = serde_json::from_value(data.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize PersonCreate: {}", e))?;

        let person_id = Uuid::new_v4();

        // 转换日期字符串为NaiveDate
        let birthday = payload
            .birthday
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        let enrollment_date = payload
            .enrollment_date
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        let hire_date = payload
            .hire_date
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());

        // 根据人员类型确定username
        let username = match payload.type_.as_str() {
            "student" => payload
                .student_no
                .clone()
                .ok_or_else(|| anyhow::anyhow!("student_no is required for student"))?,
            "teacher" => payload
                .employee_no
                .clone()
                .ok_or_else(|| anyhow::anyhow!("employee_no is required for teacher"))?,
            "parent" => payload
                .phone
                .clone()
                .unwrap_or_else(|| person_id.to_string()),
            _ => return Err(anyhow::anyhow!("Invalid person type")),
        };

        // 加载权限模板
        let permission_template = crate::core::permission::load_default_template(&payload.type_)
            .map_err(|e| anyhow::anyhow!("加载权限模板失败: {}", e))?;

        let mut tx = pool.begin().await?;

        // 生成密码哈希
        let password_to_hash = payload.password.as_deref().unwrap_or("123456");
        let password_hash =
            hash_password(password_to_hash).map_err(|_| anyhow::anyhow!("密码哈希失败"))?;

        // 插入 persons 表
        sqlx::query(
            "INSERT INTO persons (id, name, username, password_hash, gender, birthday, phone, email, type, role)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(person_id)
        .bind(&payload.name)
        .bind(&username)
        .bind(&password_hash)
        .bind(payload.gender as i16)
        .bind(birthday)
        .bind(&payload.phone)
        .bind(&payload.email)
        .bind(&payload.type_)
        .bind(&payload.type_)
        .execute(&mut *tx)
        .await?;

        // 根据类型插入子表
        match payload.type_.as_str() {
            "student" => {
                let student_no = payload
                    .student_no
                    .ok_or_else(|| anyhow::anyhow!("student_no is required for student"))?;
                sqlx::query(
                    "INSERT INTO students (person_id, student_no, class_id, enrollment_date, status)
                     VALUES ($1, $2, $3, $4, 'enrolled')"
                )
                .bind(person_id)
                .bind(student_no)
                .bind(payload.class_id)
                .bind(enrollment_date)
                .execute(&mut *tx)
                .await?;
            }
            "teacher" => {
                let employee_no = payload
                    .employee_no
                    .ok_or_else(|| anyhow::anyhow!("employee_no is required for teacher"))?;
                sqlx::query(
                    "INSERT INTO teachers (person_id, employee_no, department_id, title, hire_date)
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(person_id)
                .bind(employee_no)
                .bind(payload.department_id)
                .bind(&payload.title)
                .bind(hire_date)
                .execute(&mut *tx)
                .await?;

                // 处理老师与班级的关联
                if let Some(classes) = payload.classes {
                    for class in classes {
                        sqlx::query(
                            "INSERT INTO teacher_class (teacher_id, class_id, is_main_teacher)
                             VALUES ($1, $2, $3)",
                        )
                        .bind(person_id)
                        .bind(class.class_id)
                        .bind(class.is_main_teacher)
                        .execute(&mut *tx)
                        .await?;

                        // 如果是班主任，更新classes表
                        if class.is_main_teacher {
                            sqlx::query("UPDATE classes SET teacher_id = $1 WHERE id = $2")
                                .bind(person_id)
                                .bind(class.class_id)
                                .execute(&mut *tx)
                                .await?;

                            sqlx::query(
                                "UPDATE teacher_class SET is_main_teacher = false
                                 WHERE class_id = $1 AND teacher_id != $2",
                            )
                            .bind(class.class_id)
                            .bind(person_id)
                            .execute(&mut *tx)
                            .await?;
                        }
                    }
                }
            }
            "parent" => {
                sqlx::query(
                    "INSERT INTO parents (person_id, wechat_openid, occupation)
                     VALUES ($1, $2, $3)",
                )
                .bind(person_id)
                .bind(&payload.wechat_openid)
                .bind(&payload.occupation)
                .execute(&mut *tx)
                .await?;
            }
            _ => {}
        }

        // 插入权限
        for item in &permission_template.permissions {
            let (permission_str, value) = if item.permission.starts_with('-') {
                (&item.permission[1..], false)
            } else {
                (item.permission.as_str(), true)
            };

            sqlx::query(
                "INSERT INTO user_permissions (user_id, permission, value, priority)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (user_id, permission) DO UPDATE SET
                 value = EXCLUDED.value, priority = EXCLUDED.priority",
            )
            .bind(person_id)
            .bind(permission_str)
            .bind(value)
            .bind(item.priority)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        info!(
            "Person created from buffer: {} (type: {})",
            person_id, payload.type_
        );

        Ok(())
    }

    /// 执行人员更新（处理多表事务）
    async fn execute_person_update(
        &self,
        pool: &PgPool,
        id: &str,
        data: &Value,
    ) -> anyhow::Result<()> {
        use crate::core::password::hash_password;
        use crate::models::person::PersonUpdate;
        use uuid::Uuid;

        let person_id =
            Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid person ID: {}", e))?;

        let payload: PersonUpdate = serde_json::from_value(data.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize PersonUpdate: {}", e))?;

        let mut tx = pool.begin().await?;

        // 获取人员类型
        let person_type: Option<String> =
            sqlx::query_scalar("SELECT type FROM persons WHERE id = $1")
                .bind(person_id)
                .fetch_optional(&mut *tx)
                .await?;

        let person_type = person_type.ok_or_else(|| anyhow::anyhow!("Person not found"))?;

        // 更新 persons 表
        if let Some(name) = payload.name {
            sqlx::query("UPDATE persons SET name = $1 WHERE id = $2")
                .bind(name)
                .bind(person_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(gender) = payload.gender {
            sqlx::query("UPDATE persons SET gender = $1 WHERE id = $2")
                .bind(gender as i16)
                .bind(person_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(birthday_str) = payload.birthday {
            if !birthday_str.is_empty() {
                if let Ok(birthday) = chrono::NaiveDate::parse_from_str(&birthday_str, "%Y-%m-%d") {
                    sqlx::query("UPDATE persons SET birthday = $1 WHERE id = $2")
                        .bind(birthday)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
            } else {
                sqlx::query("UPDATE persons SET birthday = NULL WHERE id = $1")
                    .bind(person_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        if payload.phone.is_some() {
            sqlx::query("UPDATE persons SET phone = $1 WHERE id = $2")
                .bind(&payload.phone)
                .bind(person_id)
                .execute(&mut *tx)
                .await?;
        }
        if payload.email.is_some() {
            sqlx::query("UPDATE persons SET email = $1 WHERE id = $2")
                .bind(&payload.email)
                .bind(person_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(password) = payload.password.as_ref() {
            if !password.is_empty() {
                let password_hash =
                    hash_password(password).map_err(|_| anyhow::anyhow!("Password hash failed"))?;
                sqlx::query("UPDATE persons SET password_hash = $1 WHERE id = $2")
                    .bind(password_hash)
                    .bind(person_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        // 根据类型更新子表
        match person_type.as_str() {
            "student" => {
                if let Some(student_no) = payload.student_no.as_ref() {
                    sqlx::query("UPDATE students SET student_no = $1 WHERE person_id = $2")
                        .bind(student_no)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                    sqlx::query("UPDATE persons SET username = $1 WHERE id = $2")
                        .bind(student_no)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
                if payload.class_id.is_some() {
                    sqlx::query("UPDATE students SET class_id = $1 WHERE person_id = $2")
                        .bind(payload.class_id)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
                if let Some(enrollment_date_str) = payload.enrollment_date {
                    if !enrollment_date_str.is_empty() {
                        if let Ok(enrollment_date) =
                            chrono::NaiveDate::parse_from_str(&enrollment_date_str, "%Y-%m-%d")
                        {
                            sqlx::query(
                                "UPDATE students SET enrollment_date = $1 WHERE person_id = $2",
                            )
                            .bind(enrollment_date)
                            .bind(person_id)
                            .execute(&mut *tx)
                            .await?;
                        }
                    } else {
                        sqlx::query(
                            "UPDATE students SET enrollment_date = NULL WHERE person_id = $1",
                        )
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                    }
                }
            }
            "teacher" => {
                if let Some(employee_no) = payload.employee_no.as_ref() {
                    sqlx::query("UPDATE teachers SET employee_no = $1 WHERE person_id = $2")
                        .bind(employee_no)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                    sqlx::query("UPDATE persons SET username = $1 WHERE id = $2")
                        .bind(employee_no)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
                if payload.department_id.is_some() {
                    sqlx::query("UPDATE teachers SET department_id = $1 WHERE person_id = $2")
                        .bind(payload.department_id)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
                if payload.title.is_some() {
                    sqlx::query("UPDATE teachers SET title = $1 WHERE person_id = $2")
                        .bind(&payload.title)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
                if let Some(hire_date_str) = payload.hire_date {
                    if !hire_date_str.is_empty() {
                        if let Ok(hire_date) =
                            chrono::NaiveDate::parse_from_str(&hire_date_str, "%Y-%m-%d")
                        {
                            sqlx::query("UPDATE teachers SET hire_date = $1 WHERE person_id = $2")
                                .bind(hire_date)
                                .bind(person_id)
                                .execute(&mut *tx)
                                .await?;
                        }
                    } else {
                        sqlx::query("UPDATE teachers SET hire_date = NULL WHERE person_id = $1")
                            .bind(person_id)
                            .execute(&mut *tx)
                            .await?;
                    }
                }

                // 处理老师与班级的关联
                if let Some(classes) = payload.classes {
                    sqlx::query("DELETE FROM teacher_class WHERE teacher_id = $1")
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;

                    for class in classes {
                        sqlx::query(
                            "INSERT INTO teacher_class (teacher_id, class_id, is_main_teacher)
                             VALUES ($1, $2, $3)",
                        )
                        .bind(person_id)
                        .bind(class.class_id)
                        .bind(class.is_main_teacher)
                        .execute(&mut *tx)
                        .await?;

                        if class.is_main_teacher {
                            sqlx::query("UPDATE classes SET teacher_id = $1 WHERE id = $2")
                                .bind(person_id)
                                .bind(class.class_id)
                                .execute(&mut *tx)
                                .await?;

                            sqlx::query(
                                "UPDATE teacher_class SET is_main_teacher = false
                                 WHERE class_id = $1 AND teacher_id != $2",
                            )
                            .bind(class.class_id)
                            .bind(person_id)
                            .execute(&mut *tx)
                            .await?;
                        }
                    }
                }
            }
            "parent" => {
                if payload.wechat_openid.is_some() {
                    sqlx::query("UPDATE parents SET wechat_openid = $1 WHERE person_id = $2")
                        .bind(&payload.wechat_openid)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
                if payload.occupation.is_some() {
                    sqlx::query("UPDATE parents SET occupation = $1 WHERE person_id = $2")
                        .bind(&payload.occupation)
                        .bind(person_id)
                        .execute(&mut *tx)
                        .await?;
                }
            }
            _ => {}
        }

        tx.commit().await?;
        info!(
            "Person updated from buffer: {} (type: {})",
            person_id, person_type
        );

        Ok(())
    }

    /// 执行人员删除（处理多表事务）
    async fn execute_person_delete(&self, pool: &PgPool, id: &str) -> anyhow::Result<()> {
        use uuid::Uuid;

        let person_id =
            Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid person ID: {}", e))?;

        // 由于外键约束，删除persons表会自动级联删除子表记录
        let result = sqlx::query("DELETE FROM persons WHERE id = $1")
            .bind(person_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            // 记录不存在，视为已成功删除（幂等性）
            warn!(
                "Person not found for deletion (may already be deleted): {}",
                person_id
            );
            return Ok(());
        }

        info!("Person deleted from buffer: {}", person_id);

        Ok(())
    }

    // ========== 班级操作的特殊处理 ==========

    /// 执行班级创建（处理多表事务和权限植入）
    async fn execute_class_create(&self, pool: &PgPool, data: &Value) -> anyhow::Result<()> {
        use crate::core::permission::PermissionManager;
        use crate::models::class::ClassCreate;
        use uuid::Uuid;

        let payload: ClassCreate = serde_json::from_value(data.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ClassCreate: {}", e))?;

        let class_id = Uuid::new_v4();
        let grade = payload.grade as i16;
        let teacher_id = payload
            .teacher_id
            .and_then(|id_str| Uuid::parse_str(&id_str).ok());

        let mut tx = pool.begin().await?;

        // 插入classes表
        sqlx::query(
            "INSERT INTO classes (id, name, grade, teacher_id, academic_year)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(class_id)
        .bind(&payload.name)
        .bind(grade)
        .bind(teacher_id)
        .bind(&payload.academic_year)
        .execute(&mut *tx)
        .await?;

        // 如果设置了班主任，同步到teacher_class表并植入权限
        if let Some(teacher_id) = teacher_id {
            // 插入teacher_class记录，设置is_main_teacher=true
            sqlx::query(
                "INSERT INTO teacher_class (teacher_id, class_id, is_main_teacher)
                 VALUES ($1, $2, true)
                 ON CONFLICT (teacher_id, class_id)
                 DO UPDATE SET is_main_teacher = true",
            )
            .bind(teacher_id)
            .bind(class_id)
            .execute(&mut *tx)
            .await?;

            // 清除该班级其他老师的班主任标志
            sqlx::query(
                "UPDATE teacher_class SET is_main_teacher = false
                 WHERE class_id = $1 AND teacher_id != $2",
            )
            .bind(class_id)
            .bind(teacher_id)
            .execute(&mut *tx)
            .await?;

            // 为新班主任植入班级特定权限
            let permission_manager = PermissionManager::new(pool.clone());
            permission_manager
                .add_class_permissions_for_teacher(teacher_id, class_id)
                .await
                .map_err(|e| anyhow::anyhow!("植入权限失败: {}", e))?;
        }

        tx.commit().await?;
        info!(
            "Class created from buffer: {} (name: {})",
            class_id, payload.name
        );

        Ok(())
    }

    /// 执行班级更新（处理班主任变更和权限变更）
    async fn execute_class_update(
        &self,
        pool: &PgPool,
        id: &str,
        data: &Value,
    ) -> anyhow::Result<()> {
        use crate::core::permission::PermissionManager;
        use crate::models::class::ClassUpdate;
        use uuid::Uuid;

        let class_id =
            Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid class ID: {}", e))?;

        let payload: ClassUpdate = serde_json::from_value(data.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ClassUpdate: {}", e))?;

        let mut tx = pool.begin().await?;

        // 获取原班级信息（用于处理班主任变更）
        let old_class: Option<(Option<Uuid>,)> =
            sqlx::query_as("SELECT teacher_id FROM classes WHERE id = $1")
                .bind(class_id)
                .fetch_optional(&mut *tx)
                .await?;

        let old_teacher_id = old_class.and_then(|(t,)| t);

        // 更新班级基本信息
        if let Some(name) = payload.name {
            sqlx::query("UPDATE classes SET name = $1 WHERE id = $2")
                .bind(name)
                .bind(class_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(grade) = payload.grade {
            let grade = grade as i16;
            sqlx::query("UPDATE classes SET grade = $1 WHERE id = $2")
                .bind(grade)
                .bind(class_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(academic_year) = payload.academic_year {
            sqlx::query("UPDATE classes SET academic_year = $1 WHERE id = $2")
                .bind(academic_year)
                .bind(class_id)
                .execute(&mut *tx)
                .await?;
        }

        // 处理班主任变更
        if payload.teacher_id.is_some() {
            let new_teacher_id = payload
                .teacher_id
                .as_ref()
                .and_then(|id_str| Uuid::parse_str(id_str).ok());

            // 更新classes表的teacher_id
            sqlx::query("UPDATE classes SET teacher_id = $1 WHERE id = $2")
                .bind(new_teacher_id)
                .bind(class_id)
                .execute(&mut *tx)
                .await?;

            // 同步到teacher_class表并处理权限
            if let Some(new_teacher_id) = new_teacher_id {
                // 插入或更新teacher_class记录，设置is_main_teacher=true
                sqlx::query(
                    "INSERT INTO teacher_class (teacher_id, class_id, is_main_teacher)
                     VALUES ($1, $2, true)
                     ON CONFLICT (teacher_id, class_id)
                     DO UPDATE SET is_main_teacher = true",
                )
                .bind(new_teacher_id)
                .bind(class_id)
                .execute(&mut *tx)
                .await?;

                // 清除该班级其他老师的班主任标志
                sqlx::query(
                    "UPDATE teacher_class SET is_main_teacher = false
                     WHERE class_id = $1 AND teacher_id != $2",
                )
                .bind(class_id)
                .bind(new_teacher_id)
                .execute(&mut *tx)
                .await?;

                // 如果新班主任和旧班主任不同，处理权限变更
                if Some(new_teacher_id) != old_teacher_id {
                    let permission_manager = PermissionManager::new(pool.clone());

                    // 为新班主任植入权限
                    permission_manager
                        .add_class_permissions_for_teacher(new_teacher_id, class_id)
                        .await
                        .map_err(|e| anyhow::anyhow!("植入新班主任权限失败: {}", e))?;

                    // 如果存在旧班主任，移除其权限
                    if let Some(old_teacher_id) = old_teacher_id {
                        permission_manager
                            .remove_class_permissions_for_teacher(old_teacher_id, class_id)
                            .await
                            .map_err(|e| anyhow::anyhow!("移除旧班主任权限失败: {}", e))?;
                    }
                }
            } else {
                // 如果teacher_id被设置为空，清除该班级的所有班主任标志
                sqlx::query(
                    "UPDATE teacher_class SET is_main_teacher = false
                     WHERE class_id = $1",
                )
                .bind(class_id)
                .execute(&mut *tx)
                .await?;

                // 移除旧班主任的权限
                if let Some(old_teacher_id) = old_teacher_id {
                    let permission_manager = PermissionManager::new(pool.clone());
                    permission_manager
                        .remove_class_permissions_for_teacher(old_teacher_id, class_id)
                        .await
                        .map_err(|e| anyhow::anyhow!("移除旧班主任权限失败: {}", e))?;
                }
            }
        }

        tx.commit().await?;
        info!("Class updated from buffer: {}", class_id);

        Ok(())
    }

    /// 执行班级删除（级联删除关联数据）
    async fn execute_class_delete(&self, pool: &PgPool, id: &str) -> anyhow::Result<()> {
        use uuid::Uuid;

        let class_id =
            Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid class ID: {}", e))?;

        // 由于外键约束，删除classes表会自动级联删除关联的teacher_class记录
        // students表中的class_id会被设置为NULL（如果外键允许）或阻止删除（如果外键不允许）
        let result = sqlx::query("DELETE FROM classes WHERE id = $1")
            .bind(class_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            // 记录不存在，视为已成功删除（幂等性）
            warn!(
                "Class not found for deletion (may already be deleted): {}",
                class_id
            );
            return Ok(());
        }

        info!("Class deleted from buffer: {}", class_id);

        Ok(())
    }

    // ========== 部门操作的特殊处理 ==========

    /// 执行部门创建
    async fn execute_department_create(&self, pool: &PgPool, data: &Value) -> anyhow::Result<()> {
        use crate::models::department::DepartmentCreate;
        use uuid::Uuid;

        let payload: DepartmentCreate = serde_json::from_value(data.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize DepartmentCreate: {}", e))?;

        let department_id = Uuid::new_v4();
        let parent_id = payload
            .parent_id
            .and_then(|id_str| Uuid::parse_str(&id_str).ok());

        sqlx::query(
            "INSERT INTO departments (id, name, parent_id)
             VALUES ($1, $2, $3)",
        )
        .bind(department_id)
        .bind(&payload.name)
        .bind(parent_id)
        .execute(pool)
        .await?;

        info!(
            "Department created from buffer: {} (name: {})",
            department_id, payload.name
        );

        Ok(())
    }

    /// 执行部门更新
    async fn execute_department_update(
        &self,
        pool: &PgPool,
        id: &str,
        data: &Value,
    ) -> anyhow::Result<()> {
        use crate::models::department::DepartmentUpdate;
        use uuid::Uuid;

        let department_id =
            Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid department ID: {}", e))?;

        let payload: DepartmentUpdate = serde_json::from_value(data.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize DepartmentUpdate: {}", e))?;

        if let Some(name) = payload.name {
            sqlx::query("UPDATE departments SET name = $1 WHERE id = $2")
                .bind(name)
                .bind(department_id)
                .execute(pool)
                .await?;
        }
        if payload.parent_id.is_some() {
            let parent_id = payload
                .parent_id
                .as_ref()
                .and_then(|id_str| Uuid::parse_str(id_str).ok());
            sqlx::query("UPDATE departments SET parent_id = $1 WHERE id = $2")
                .bind(parent_id)
                .bind(department_id)
                .execute(pool)
                .await?;
        }

        info!("Department updated from buffer: {}", department_id);

        Ok(())
    }

    /// 执行部门删除
    async fn execute_department_delete(&self, pool: &PgPool, id: &str) -> anyhow::Result<()> {
        use uuid::Uuid;

        let department_id =
            Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid department ID: {}", e))?;

        // 检查是否有子部门
        let has_children: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM departments WHERE parent_id = $1)")
                .bind(department_id)
                .fetch_one(pool)
                .await?;

        if has_children {
            return Err(anyhow::anyhow!(
                "Cannot delete department with child departments"
            ));
        }

        // 检查是否有关联的老师
        let has_teachers: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM teachers WHERE department_id = $1)")
                .bind(department_id)
                .fetch_one(pool)
                .await?;

        if has_teachers {
            return Err(anyhow::anyhow!(
                "Cannot delete department with associated teachers"
            ));
        }

        let result = sqlx::query("DELETE FROM departments WHERE id = $1")
            .bind(department_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            // 记录不存在，视为已成功删除（幂等性）
            warn!(
                "Department not found for deletion (may already be deleted): {}",
                department_id
            );
            return Ok(());
        }

        info!("Department deleted from buffer: {}", department_id);

        Ok(())
    }

    // ========== 聊天记录操作的特殊处理 ==========

    /// 执行聊天消息创建（更新会话时间和成员已读状态）
    async fn execute_chat_message_create(&self, pool: &PgPool, data: &Value) -> anyhow::Result<()> {
        use uuid::Uuid;

        let conversation_id = data
            .get("conversation_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| anyhow::anyhow!("Invalid conversation_id"))?;

        let sender_id = data
            .get("sender_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| anyhow::anyhow!("Invalid sender_id"))?;

        let content = data
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid content"))?;

        let message_type = data
            .get("message_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        let mut tx = pool.begin().await?;

        // 插入消息
        let message_id: Uuid = sqlx::query_scalar(
            "INSERT INTO chat_messages (conversation_id, sender_id, content, message_type)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
        )
        .bind(conversation_id)
        .bind(sender_id)
        .bind(content)
        .bind(message_type)
        .fetch_one(&mut *tx)
        .await?;

        // 更新会话时间
        sqlx::query("UPDATE chat_conversations SET updated_at = NOW() WHERE id = $1")
            .bind(conversation_id)
            .execute(&mut *tx)
            .await?;

        // 更新发送者的已读状态
        sqlx::query(
            "UPDATE chat_conversation_members SET last_read_at = NOW()
             WHERE conversation_id = $1 AND user_id = $2",
        )
        .bind(conversation_id)
        .bind(sender_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        info!(
            "Chat message created from buffer: {} (conversation: {})",
            message_id, conversation_id
        );

        Ok(())
    }

    async fn move_to_dead_letter(&self, write: &BufferedWrite) -> Result<()> {
        let json = serde_json::to_string(write)?;
        self.redis_client
            .lpush(BUFFER_DEAD_LETTER_KEY, &json)
            .await?;
        warn!(
            "Moved write to dead letter queue: {} {:?}",
            write.table, write.operation
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_stats(&self) -> Result<BufferStats> {
        let queue_length = self.redis_client.llen(BUFFER_QUEUE_KEY).await?;
        let dead_letter_length = self.redis_client.llen(BUFFER_DEAD_LETTER_KEY).await?;

        let mut stats = self.stats.lock().await;
        stats.queue_length = queue_length;
        stats.dead_letter_length = dead_letter_length;

        Ok(stats.clone())
    }

    pub async fn get_queue_length(&self) -> Result<usize> {
        self.redis_client.llen(BUFFER_QUEUE_KEY).await
    }

    pub fn start_flush_scheduler(self: Arc<Self>) {
        let interval_secs = self.config.flush_interval_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            info!(
                "Started buffer flush scheduler (interval: {}s)",
                interval_secs
            );

            loop {
                ticker.tick().await;

                match self.flush().await {
                    Ok(result) => {
                        if result.processed > 0 {
                            debug!("Scheduled flush completed: {:?}", result);
                        }
                    }
                    Err(e) => {
                        error!("Scheduled flush failed: {}", e);
                    }
                }
            }
        });
    }

    pub async fn force_flush(&self) -> Result<FlushResult> {
        info!("Force flushing buffer...");
        self.flush().await
    }

    /// 清空缓冲队列（将所有待处理的操作移到死信队列）
    #[allow(dead_code)]
    pub async fn clear_buffer(&self) -> Result<usize> {
        let mut count = 0;
        loop {
            match self.redis_client.rpop(BUFFER_QUEUE_KEY).await? {
                Some(json) => {
                    match serde_json::from_str::<BufferedWrite>(&json) {
                        Ok(write) => {
                            self.move_to_dead_letter(&write).await?;
                            count += 1;
                        }
                        Err(e) => {
                            error!("Failed to deserialize buffered write during clear: {}", e);
                        }
                    }
                }
                None => break,
            }
        }
        if count > 0 {
            warn!("Cleared {} buffered writes to dead letter queue", count);
        }
        Ok(count)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlushResult {
    pub processed: u32,
    pub succeeded: u32,
    pub failed: u32,
}
