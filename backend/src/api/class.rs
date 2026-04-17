use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::data_scope::{ensure_class_access, get_accessible_class_ids};
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;
use crate::core::redis::cache::CacheKey;
use crate::models::class::{Class, ClassCreate, ClassResponse, ClassUpdate};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
    pub grade: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<ClassResponse>>, AppError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let accessible_class_ids = get_accessible_class_ids(&pool, user_id, &claims.role).await?;

    // 优先使用 DatabaseService（带缓存）
    if let Some(db_service) = &state.db_service {
        let cache_key = CacheKey::classes_list(Some(&format!(
            "{}:{}:{:?}:{:?}",
            page, limit, query.search, query.grade
        )));

        // 尝试从缓存获取
        match db_service
            .query_cached::<Vec<ClassResponse>>(&cache_key)
            .await
        {
            Ok(Some(cached)) => {
                tracing::trace!("Cache hit for classes list: {}", cache_key);
                let total = cached.len() as i64;
                return Ok(Json(ListResponse {
                    items: cached,
                    total,
                    page,
                    limit,
                }));
            }
            Ok(None) => {
                tracing::trace!("Cache miss for classes list: {}", cache_key);
            }
            Err(e) => {
                tracing::warn!("Cache read error: {}, falling back to database", e);
            }
        }
    }

    // 回退到直接数据库查询
    let (items, total) = list_classes(
        &pool,
        query.search.as_deref(),
        query.grade,
        page,
        limit,
        &accessible_class_ids,
    )
    .await?;

    // 写入缓存
    if let Some(db_service) = &state.db_service {
        let cache_key = CacheKey::classes_list(Some(&format!(
            "{}:{}:{:?}:{:?}",
            page, limit, query.search, query.grade
        )));
        if let Err(e) = db_service.cache_set(&cache_key, &items, None).await {
            tracing::warn!("Failed to cache classes list: {}", e);
        }
    }

    Ok(Json(ListResponse {
        items,
        total,
        page,
        limit,
    }))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<ClassCreate>,
) -> Result<Json<ClassResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    // 尝试使用写入缓冲（如果可用）
    if let Some(db_service) = &state.db_service {
        if let Some(buffer) = db_service.get_write_buffer() {
            let class_data = serde_json::to_value(&payload)
                .map_err(|e| AppError::InternalWithMessage(format!("序列化失败: {}", e)))?;

            match buffer.buffer_insert("classes", class_data).await {
                Ok(_) => {
                    tracing::info!("Class creation buffered successfully");
                    // 使班级列表缓存失效
                    if let Err(e) = db_service.invalidate_entity_cache("classes", None).await {
                        tracing::warn!("Failed to invalidate classes cache after buffer: {}", e);
                    }
                    // 返回临时响应
                    return Ok(Json(ClassResponse {
                        id: Uuid::new_v4(),
                        name: payload.name.clone(),
                        grade: payload.grade as i16,
                        teacher_id: payload.teacher_id.and_then(|s| Uuid::parse_str(&s).ok()),
                        teacher_name: None,
                        academic_year: payload.academic_year.clone(),
                        created_at: chrono::Utc::now(),
                    }));
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to buffer class creation, falling back to immediate: {}",
                        e
                    );
                }
            }
        }
    }

    // 直接写入数据库（回退方案）
    let class = create_class(&pool, payload).await?;

    // 使班级列表缓存失效
    if let Some(db_service) = &state.db_service {
        if let Err(e) = db_service.invalidate_entity_cache("classes", None).await {
            tracing::warn!("Failed to invalidate classes cache after create: {}", e);
        }
    }

    Ok(Json(class))
}

pub async fn get(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<ClassResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    ensure_class_access(&pool, user_id, &claims.role, id).await?;

    // 优先使用 DatabaseService（带缓存）
    if let Some(db_service) = &state.db_service {
        let cache_key = CacheKey::class(&id.to_string());

        // 尝试从缓存获取
        match db_service.query_cached::<ClassResponse>(&cache_key).await {
            Ok(Some(cached)) => {
                tracing::trace!("Cache hit for class: {}", cache_key);
                return Ok(Json(cached));
            }
            Ok(None) => {
                tracing::trace!("Cache miss for class: {}", cache_key);
            }
            Err(e) => {
                tracing::warn!("Cache read error: {}, falling back to database", e);
            }
        }
    }

    // 回退到直接数据库查询
    let class = get_class(&pool, id).await?;

    // 写入缓存
    if let Some(db_service) = &state.db_service {
        let cache_key = CacheKey::class(&id.to_string());
        if let Err(e) = db_service.cache_set(&cache_key, &class, None).await {
            tracing::warn!("Failed to cache class: {}", e);
        }
    }

    Ok(Json(class))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ClassUpdate>,
) -> Result<Json<ClassResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    // 检查权限：如果尝试更新班主任，需要class.update_teacher权限
    if payload.teacher_id.is_some() {
        // 使用新的权限系统检查用户是否有class.update_teacher权限
        let user_id =
            Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
        let manager = PermissionManager::new(pool.clone());
        manager
            .require_permission(user_id, "class.update_teacher")
            .await?;
    }

    // 尝试使用写入缓冲（如果可用）
    if let Some(db_service) = &state.db_service {
        if let Some(buffer) = db_service.get_write_buffer() {
            let update_data = serde_json::to_value(&payload)
                .map_err(|e| AppError::InternalWithMessage(format!("序列化失败: {}", e)))?;

            match buffer
                .buffer_update("classes", &id.to_string(), update_data)
                .await
            {
                Ok(_) => {
                    tracing::info!("Class update buffered successfully: {}", id);
                    // 使缓存失效
                    if let Err(e) = db_service
                        .invalidate_entity_cache("classes", Some(&id.to_string()))
                        .await
                    {
                        tracing::warn!("Failed to invalidate class cache after buffer: {}", e);
                    }
                    // 返回更新后的班级信息（从数据库获取最新数据或构造临时响应）
                    let class = get_class(&pool, id).await?;
                    return Ok(Json(class));
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to buffer class update, falling back to immediate: {}",
                        e
                    );
                }
            }
        }
    }

    // 直接更新数据库（回退方案）
    let class = update_class(&pool, id, payload).await?;

    // 使缓存失效
    if let Some(db_service) = &state.db_service {
        if let Err(e) = db_service
            .invalidate_entity_cache("classes", Some(&id.to_string()))
            .await
        {
            tracing::warn!("Failed to invalidate class cache after update: {}", e);
        }
    }

    Ok(Json(class))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    // 检查删除班级权限
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "class.delete").await?;

    // 尝试使用写入缓冲（如果可用）
    if let Some(db_service) = &state.db_service {
        if let Some(buffer) = db_service.get_write_buffer() {
            match buffer.buffer_delete("classes", &id.to_string()).await {
                Ok(_) => {
                    tracing::info!("Class deletion buffered successfully: {}", id);
                    // 使缓存失效
                    if let Err(e) = db_service
                        .invalidate_entity_cache("classes", Some(&id.to_string()))
                        .await
                    {
                        tracing::warn!("Failed to invalidate class cache after buffer: {}", e);
                    }
                    return Ok(StatusCode::NO_CONTENT);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to buffer class deletion, falling back to immediate: {}",
                        e
                    );
                }
            }
        }
    }

    // 直接删除数据库（回退方案）
    delete_class(&pool, id).await?;

    // 使缓存失效
    if let Some(db_service) = &state.db_service {
        if let Err(e) = db_service
            .invalidate_entity_cache("classes", Some(&id.to_string()))
            .await
        {
            tracing::warn!("Failed to invalidate class cache after delete: {}", e);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// 获取班级的学生列表
pub async fn get_class_students(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PersonResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    ensure_class_access(&pool, user_id, &claims.role, id).await?;

    let students = get_class_students_list(&pool, id).await?;
    Ok(Json(students))
}

// 获取班级的老师列表
pub async fn get_class_teachers(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PersonResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    ensure_class_access(&pool, user_id, &claims.role, id).await?;

    let teachers = get_class_teachers_list(&pool, id).await?;
    Ok(Json(teachers))
}

use crate::models::person::{PersonResponse, StudentResponse, TeacherResponse};

async fn get_class_students_list(
    pool: &sqlx::PgPool,
    class_id: Uuid,
) -> Result<Vec<PersonResponse>, AppError> {
    let rows: Vec<_> = sqlx::query!(
        "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type,
                s.student_no, s.enrollment_date, s.status
         FROM persons p
         JOIN students s ON p.id = s.person_id
         WHERE s.class_id = $1
         ORDER BY p.name",
        class_id
    )
    .fetch_all(pool)
    .await?;

    let students: Vec<PersonResponse> = rows
        .into_iter()
        .map(|row| {
            PersonResponse::Student(StudentResponse {
                id: row.id,
                name: row.name,
                gender: row.gender,
                birthday: row.birthday,
                phone: row.phone,
                email: row.email,
                student_no: row.student_no,
                class_id: Some(class_id),
                class_name: None, // 需要额外查询
                enrollment_date: row.enrollment_date,
                status: row.status.expect("Student status is required"),
            })
        })
        .collect();

    Ok(students)
}

async fn get_class_teachers_list(
    pool: &sqlx::PgPool,
    class_id: Uuid,
) -> Result<Vec<PersonResponse>, AppError> {
    let rows: Vec<_> = sqlx::query!(
        "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type,
                t.employee_no, t.department_id, t.title, t.hire_date,
                tc.is_main_teacher
         FROM persons p
         JOIN teachers t ON p.id = t.person_id
         JOIN teacher_class tc ON t.person_id = tc.teacher_id
         WHERE tc.class_id = $1
         ORDER BY tc.is_main_teacher DESC, p.name",
        class_id
    )
    .fetch_all(pool)
    .await?;

    let teachers: Vec<PersonResponse> = rows
        .into_iter()
        .map(|row| {
            PersonResponse::Teacher(TeacherResponse {
                id: row.id,
                name: row.name,
                gender: row.gender,
                birthday: row.birthday,
                phone: row.phone,
                email: row.email,
                employee_no: row.employee_no,
                department_id: row.department_id,
                department_name: None, // 需要额外查询
                classes: Vec::new(),   // 需要额外查询
                title: row.title,
                hire_date: row.hire_date,
            })
        })
        .collect();

    Ok(teachers)
}

async fn list_classes(
    pool: &sqlx::PgPool,
    search: Option<&str>,
    grade: Option<i16>,
    page: i64,
    limit: i64,
    accessible_class_ids: &[Uuid],
) -> Result<(Vec<ClassResponse>, i64), AppError> {
    if accessible_class_ids.is_empty() {
        return Ok((Vec::new(), 0));
    }

    use sqlx::{Postgres, QueryBuilder};

    let offset = (page - 1) * limit;

    let mut total_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT COUNT(*)::bigint AS total FROM classes c WHERE c.id IN (");
    {
        let mut separated = total_builder.separated(", ");
        for class_id in accessible_class_ids {
            separated.push_bind(class_id);
        }
    }
    total_builder.push(")");
    if let Some(s) = search {
        total_builder
            .push(" AND c.name ILIKE ")
            .push_bind(format!("%{}%", s));
    }
    if let Some(g) = grade {
        total_builder.push(" AND c.grade = ").push_bind(g);
    }
    let total: i64 = total_builder.build_query_scalar().fetch_one(pool).await?;

    let mut rows_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT c.id, c.name, c.grade, c.teacher_id, c.academic_year, c.created_at,
                p.name as teacher_name
         FROM classes c
         LEFT JOIN persons p ON c.teacher_id = p.id
         WHERE c.id IN (",
    );
    {
        let mut separated = rows_builder.separated(", ");
        for class_id in accessible_class_ids {
            separated.push_bind(class_id);
        }
    }
    rows_builder.push(")");
    if let Some(s) = search {
        rows_builder
            .push(" AND c.name ILIKE ")
            .push_bind(format!("%{}%", s));
    }
    if let Some(g) = grade {
        rows_builder.push(" AND c.grade = ").push_bind(g);
    }
    rows_builder
        .push(" ORDER BY c.created_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows = rows_builder
        .build_query_as::<ClassWithTeacher>()
        .fetch_all(pool)
        .await?;

    let items: Vec<ClassResponse> = rows.into_iter().map(|row| row.into_response()).collect();

    Ok((items, total))
}

async fn get_class(pool: &sqlx::PgPool, id: Uuid) -> Result<ClassResponse, AppError> {
    let row = sqlx::query_as::<_, ClassWithTeacher>(
        "SELECT c.id, c.name, c.grade, c.teacher_id, c.academic_year, c.created_at,
                p.name as teacher_name
         FROM classes c
         LEFT JOIN persons p ON c.teacher_id = p.id
         WHERE c.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(row.into_response())
}

async fn create_class(
    pool: &sqlx::PgPool,
    payload: ClassCreate,
) -> Result<ClassResponse, AppError> {
    let mut tx = pool.begin().await?;
    let id = Uuid::new_v4();

    // Convert i32 grade to i16 for database
    let grade = payload.grade as i16;

    // Convert string teacher_id to Uuid if provided
    let teacher_id = payload
        .teacher_id
        .and_then(|id_str| Uuid::parse_str(&id_str).ok());

    sqlx::query(
        "INSERT INTO classes (id, name, grade, teacher_id, academic_year)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
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
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 清除该班级其他老师的班主任标志
        sqlx::query(
            "UPDATE teacher_class SET is_main_teacher = false 
             WHERE class_id = $1 AND teacher_id != $2",
        )
        .bind(id)
        .bind(teacher_id)
        .execute(&mut *tx)
        .await?;

        // 为新班主任植入班级特定权限
        let permission_manager = PermissionManager::new(pool.clone());
        permission_manager
            .add_class_permissions_for_teacher(teacher_id, id)
            .await
            .map_err(|e| AppError::InternalWithMessage(format!("植入权限失败: {}", e)))?;
    }

    tx.commit().await?;
    get_class(pool, id).await
}

async fn update_class(
    pool: &sqlx::PgPool,
    id: Uuid,
    payload: ClassUpdate,
) -> Result<ClassResponse, AppError> {
    let mut tx = pool.begin().await?;

    // 获取原班级信息（用于处理班主任变更）
    let old_class = sqlx::query_as::<_, Class>("SELECT * FROM classes WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

    if let Some(name) = payload.name {
        sqlx::query("UPDATE classes SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(grade) = payload.grade {
        // Convert i32 grade to i16 for database
        let grade = grade as i16;
        sqlx::query("UPDATE classes SET grade = $1 WHERE id = $2")
            .bind(grade)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    // 处理班主任变更
    if payload.teacher_id.is_some() {
        // Convert string teacher_id to Uuid if provided
        let new_teacher_id = payload
            .teacher_id
            .as_ref()
            .and_then(|id_str| Uuid::parse_str(id_str).ok());
        let old_teacher_id = old_class.teacher_id;

        // 更新classes表的teacher_id
        sqlx::query("UPDATE classes SET teacher_id = $1 WHERE id = $2")
            .bind(new_teacher_id)
            .bind(id)
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
            .bind(id)
            .execute(&mut *tx)
            .await?;

            // 清除该班级其他老师的班主任标志
            sqlx::query(
                "UPDATE teacher_class SET is_main_teacher = false 
                 WHERE class_id = $1 AND teacher_id != $2",
            )
            .bind(id)
            .bind(new_teacher_id)
            .execute(&mut *tx)
            .await?;

            // 如果新班主任和旧班主任不同，处理权限变更
            if Some(new_teacher_id) != old_teacher_id {
                let permission_manager = PermissionManager::new(pool.clone());

                // 为新班主任植入权限
                permission_manager
                    .add_class_permissions_for_teacher(new_teacher_id, id)
                    .await
                    .map_err(|e| {
                        AppError::InternalWithMessage(format!("植入新班主任权限失败: {}", e))
                    })?;

                // 如果存在旧班主任，移除其权限
                if let Some(old_teacher_id) = old_teacher_id {
                    permission_manager
                        .remove_class_permissions_for_teacher(old_teacher_id, id)
                        .await
                        .map_err(|e| {
                            AppError::InternalWithMessage(format!("移除旧班主任权限失败: {}", e))
                        })?;
                }
            }
        } else {
            // 如果teacher_id被设置为空，清除该班级的所有班主任标志
            sqlx::query(
                "UPDATE teacher_class SET is_main_teacher = false 
                 WHERE class_id = $1",
            )
            .bind(id)
            .execute(&mut *tx)
            .await?;

            // 移除旧班主任的权限
            if let Some(old_teacher_id) = old_teacher_id {
                let permission_manager = PermissionManager::new(pool.clone());
                permission_manager
                    .remove_class_permissions_for_teacher(old_teacher_id, id)
                    .await
                    .map_err(|e| {
                        AppError::InternalWithMessage(format!("移除旧班主任权限失败: {}", e))
                    })?;
            }
        }
    }
    if let Some(academic_year) = payload.academic_year {
        sqlx::query("UPDATE classes SET academic_year = $1 WHERE id = $2")
            .bind(academic_year)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    get_class(pool, id).await
}

async fn delete_class(pool: &sqlx::PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM classes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct ClassWithTeacher {
    id: Uuid,
    name: String,
    grade: i16,
    teacher_id: Option<Uuid>,
    academic_year: String,
    created_at: chrono::DateTime<chrono::Utc>,
    teacher_name: Option<String>,
}

impl ClassWithTeacher {
    fn into_response(self) -> ClassResponse {
        ClassResponse {
            id: self.id,
            name: self.name,
            grade: self.grade,
            teacher_id: self.teacher_id,
            teacher_name: self.teacher_name,
            academic_year: self.academic_year,
            created_at: self.created_at,
        }
    }
}
