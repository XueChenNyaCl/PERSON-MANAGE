use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;
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
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<ClassResponse>>, AppError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    if let Some(pool) = state.pool {
        let (items, total) =
            list_classes(&pool, query.search.as_deref(), query.grade, page, limit).await?;

        Ok(Json(ListResponse {
            items,
            total,
            page,
            limit,
        }))
    } else {
        Ok(Json(ListResponse {
            items: Vec::new(),
            total: 0,
            page,
            limit,
        }))
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<ClassCreate>,
) -> Result<Json<ClassResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    let class = create_class(&pool, payload).await?;
    Ok(Json(class))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ClassResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    let class = get_class(&pool, id).await?;
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
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
        let manager = PermissionManager::new(pool.clone());
        manager.require_permission(user_id, "class.update_teacher").await?;
    }
    
    let class = update_class(&pool, id, payload).await?;
    Ok(Json(class))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查删除班级权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "class.delete").await?;

    delete_class(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// 获取班级的学生列表
pub async fn get_class_students(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PersonResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    let students = get_class_students_list(&pool, id).await?;
    Ok(Json(students))
}

// 获取班级的老师列表
pub async fn get_class_teachers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PersonResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

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

    let students: Vec<PersonResponse> = rows.into_iter().map(|row| {
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
            status: row.status.expect("Student status is required")
        })
    }).collect();

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

    let teachers: Vec<PersonResponse> = rows.into_iter().map(|row| {
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
            classes: Vec::new(), // 需要额外查询
            title: row.title,
            hire_date: row.hire_date
        })
    }).collect();

    Ok(teachers)
}

async fn list_classes(
    pool: &sqlx::PgPool,
    search: Option<&str>,
    grade: Option<i16>,
    page: i64,
    limit: i64,
) -> Result<(Vec<ClassResponse>, i64), AppError> {
    let offset = (page - 1) * limit;

    let total = if let Some(s) = search {
        if let Some(g) = grade {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM classes c WHERE c.name ILIKE $1 AND c.grade = $2",
            )
            .bind(format!("%{}%", s))
            .bind(g)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM classes c WHERE c.name ILIKE $1")
                .bind(format!("%{}%", s))
                .fetch_one(pool)
                .await?
        }
    } else if let Some(g) = grade {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM classes c WHERE c.grade = $1")
            .bind(g)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM classes c")
            .fetch_one(pool)
            .await?
    };

    let rows = if let Some(s) = search {
        if let Some(g) = grade {
            sqlx::query_as::<_, ClassWithTeacher>(
                "SELECT c.id, c.name, c.grade, c.teacher_id, c.academic_year, c.created_at,
                        p.name as teacher_name
                 FROM classes c
                 LEFT JOIN persons p ON c.teacher_id = p.id
                 WHERE c.name ILIKE $1 AND c.grade = $2
                 ORDER BY c.created_at DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(format!("%{}%", s))
            .bind(g)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, ClassWithTeacher>(
                "SELECT c.id, c.name, c.grade, c.teacher_id, c.academic_year, c.created_at,
                        p.name as teacher_name
                 FROM classes c
                 LEFT JOIN persons p ON c.teacher_id = p.id
                 WHERE c.name ILIKE $1
                 ORDER BY c.created_at DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(format!("%{}%", s))
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    } else if let Some(g) = grade {
        sqlx::query_as::<_, ClassWithTeacher>(
            "SELECT c.id, c.name, c.grade, c.teacher_id, c.academic_year, c.created_at,
                    p.name as teacher_name
             FROM classes c
             LEFT JOIN persons p ON c.teacher_id = p.id
             WHERE c.grade = $1
             ORDER BY c.created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(g)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ClassWithTeacher>(
            "SELECT c.id, c.name, c.grade, c.teacher_id, c.academic_year, c.created_at,
                    p.name as teacher_name
             FROM classes c
             LEFT JOIN persons p ON c.teacher_id = p.id
             ORDER BY c.created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

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
    let teacher_id = payload.teacher_id.and_then(|id_str| Uuid::parse_str(&id_str).ok());

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
        permission_manager.add_class_permissions_for_teacher(teacher_id, id).await
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
        let new_teacher_id = payload.teacher_id.as_ref().and_then(|id_str| Uuid::parse_str(id_str).ok());
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
                permission_manager.add_class_permissions_for_teacher(new_teacher_id, id).await
                    .map_err(|e| AppError::InternalWithMessage(format!("植入新班主任权限失败: {}", e)))?;
                
                // 如果存在旧班主任，移除其权限
                if let Some(old_teacher_id) = old_teacher_id {
                    permission_manager.remove_class_permissions_for_teacher(old_teacher_id, id).await
                        .map_err(|e| AppError::InternalWithMessage(format!("移除旧班主任权限失败: {}", e)))?;
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
                permission_manager.remove_class_permissions_for_teacher(old_teacher_id, id).await
                    .map_err(|e| AppError::InternalWithMessage(format!("移除旧班主任权限失败: {}", e)))?;
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
