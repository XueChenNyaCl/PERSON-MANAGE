use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::password::hash_password;
use crate::core::permission::PermissionManager;
use crate::models::person::{
    ParentResponse, Person, PersonCreate, PersonResponse, PersonUpdate, StudentResponse,
    TeacherResponse,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub r#type: Option<String>,
    pub search: Option<String>,
    pub class_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
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
) -> Result<Json<ListResponse<PersonResponse>>, AppError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    // 检查数据库连接
    if let Some(pool) = state.pool {
        let (items, total) = list_persons(
            &pool,
            query.r#type.as_deref(),
            query.search.as_deref(),
            query.class_id,
            query.department_id,
            page,
            limit,
        )
        .await?;

        Ok(Json(ListResponse {
            items,
            total,
            page,
            limit,
        }))
    } else {
        // 数据库连接失败，返回空列表
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
    Extension(claims): Extension<Claims>,
    Json(payload): Json<PersonCreate>,
) -> Result<Json<PersonResponse>, AppError> {
    println!("=== CREATE PERSON DEBUG ===");
    println!("Received payload: {:?}", payload);
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查创建人员权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "person.create").await?;

    let person = create_person(&pool, payload).await?;
    Ok(Json(person))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PersonResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::NotFound)?;

    let person = get_person(&pool, id).await?;
    Ok(Json(person))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PersonUpdate>,
) -> Result<Json<PersonResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查更新人员权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "person.update").await?;

    let person = update_person(&pool, id, payload).await?;
    Ok(Json(person))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查删除人员权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "person.delete").await?;

    delete_person(&pool, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct TeacherClassesQuery {
    pub teacher_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ClassWithTeacherInfo {
    pub id: Uuid,
    pub name: String,
    pub grade: i16,
    pub academic_year: String,
    pub is_main_teacher: bool,
}

pub async fn get_teacher_classes(
    State(state): State<AppState>,
    Query(query): Query<TeacherClassesQuery>,
) -> Result<Json<Vec<ClassWithTeacherInfo>>, AppError> {
    if let Some(pool) = state.pool {
        let rows = sqlx::query!(
            "SELECT c.id, 
                    COALESCE(c.name, '') as name, 
                    COALESCE(c.grade, 0) as grade, 
                    COALESCE(c.academic_year, '') as academic_year, 
                    COALESCE(tc.is_main_teacher, false) as is_main_teacher
             FROM classes c
             JOIN teacher_class tc ON c.id = tc.class_id
             WHERE tc.teacher_id = $1
             ORDER BY c.grade, c.name",
            query.teacher_id
        )
        .fetch_all(&pool)
        .await?;

        let classes: Vec<ClassWithTeacherInfo> = rows.into_iter().map(|row| {
            ClassWithTeacherInfo {
                id: row.id,
                name: row.name.unwrap_or_default(),
                grade: row.grade.unwrap_or(0) as i16,
                academic_year: row.academic_year.unwrap_or_default(),
                is_main_teacher: row.is_main_teacher.unwrap_or(false)
            }
        }).collect();

        Ok(Json(classes))
    } else {
        Ok(Json(Vec::new()))
    }
}

async fn list_persons(
    pool: &sqlx::PgPool,
    person_type: Option<&str>,
    search: Option<&str>,
    _class_id: Option<Uuid>,
    _department_id: Option<Uuid>,
    page: i64,
    limit: i64,
) -> Result<(Vec<PersonResponse>, i64), AppError> {
    let offset = (page - 1) * limit;
    
    // 处理空字符串情况，视为None
    let person_type = person_type.filter(|&t| !t.is_empty());

    let total = if let Some(t) = person_type {
        if let Some(s) = search {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM persons p WHERE p.type = $1 AND p.name ILIKE $2",
            )
            .bind(t)
            .bind(format!("%{}%", s))
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons p WHERE p.type = $1")
                .bind(t)
                .fetch_one(pool)
                .await?
        }
    } else if let Some(s) = search {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons p WHERE p.name ILIKE $1")
            .bind(format!("%{}%", s))
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons p")
            .fetch_one(pool)
            .await?
    };

    let rows = if let Some(t) = person_type {
        if let Some(s) = search {
            if let Some(class_id) = _class_id {
                // 根据人员类型使用不同的class_id过滤逻辑
                let query = if t == "teacher" {
                    // 老师通过teacher_class表关联
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     INNER JOIN teacher_class tc ON p.id = tc.teacher_id AND tc.class_id = $3
                     WHERE p.type = $1 AND p.name ILIKE $2
                     ORDER BY p.created_at DESC
                     LIMIT $4 OFFSET $5"
                } else if t == "student" {
                    // 学生通过students.class_id关联
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1 AND p.name ILIKE $2 AND s.class_id = $3
                     ORDER BY p.created_at DESC
                     LIMIT $4 OFFSET $5"
                } else {
                    // 其他类型（如家长）不支持class_id过滤，只按name搜索
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1 AND p.name ILIKE $2
                     ORDER BY p.created_at DESC
                     LIMIT $4 OFFSET $5"
                };
                
                sqlx::query_as::<_, PersonWithRelations>(query)
                    .bind(t)
                    .bind(format!("%{}%", s))
                    .bind(class_id)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await?
            } else if let Some(department_id) = _department_id {
                sqlx::query_as::<_, PersonWithRelations>(
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1 AND p.name ILIKE $2 AND t.department_id = $3
                     ORDER BY p.created_at DESC
                     LIMIT $4 OFFSET $5",
                )
                .bind(t)
                .bind(format!("%{}%", s))
                .bind(department_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, PersonWithRelations>(
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1 AND p.name ILIKE $2
                     ORDER BY p.created_at DESC
                     LIMIT $3 OFFSET $4",
                )
                .bind(t)
                .bind(format!("%{}%", s))
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?
            }
        } else {
            if let Some(class_id) = _class_id {
                // 根据人员类型使用不同的class_id过滤逻辑
                let query = if t == "teacher" {
                    // 老师通过teacher_class表关联
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     INNER JOIN teacher_class tc ON p.id = tc.teacher_id AND tc.class_id = $2
                     WHERE p.type = $1
                     ORDER BY p.created_at DESC
                     LIMIT $3 OFFSET $4"
                } else if t == "student" {
                    // 学生通过students.class_id关联
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1 AND s.class_id = $2
                     ORDER BY p.created_at DESC
                     LIMIT $3 OFFSET $4"
                } else {
                    // 其他类型（如家长）不支持class_id过滤
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1
                     ORDER BY p.created_at DESC
                     LIMIT $3 OFFSET $4"
                };
                
                sqlx::query_as::<_, PersonWithRelations>(query)
                    .bind(t)
                    .bind(class_id)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await?
            } else if let Some(department_id) = _department_id {
                sqlx::query_as::<_, PersonWithRelations>(
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1 AND t.department_id = $2
                     ORDER BY p.created_at DESC
                     LIMIT $3 OFFSET $4",
                )
                .bind(t)
                .bind(department_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, PersonWithRelations>(
                    "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                            s.student_no, s.class_id, s.enrollment_date, s.status,
                            t.employee_no, t.department_id, t.title, t.hire_date,
                            pa.wechat_openid, pa.occupation,
                            c.name as class_name, d.name as department_name
                     FROM persons p
                     LEFT JOIN students s ON p.id = s.person_id
                     LEFT JOIN teachers t ON p.id = t.person_id
                     LEFT JOIN parents pa ON p.id = pa.person_id
                     LEFT JOIN classes c ON s.class_id = c.id
                     LEFT JOIN departments d ON t.department_id = d.id
                     WHERE p.type = $1
                     ORDER BY p.created_at DESC
                     LIMIT $2 OFFSET $3",
                )
                .bind(t)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?
            }
        }
    } else if let Some(s) = search {
        sqlx::query_as::<_, PersonWithRelations>(
            "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                    s.student_no, s.class_id, s.enrollment_date, s.status,
                    t.employee_no, t.department_id, t.title, t.hire_date,
                    pa.wechat_openid, pa.occupation,
                    c.name as class_name, d.name as department_name
             FROM persons p
             LEFT JOIN students s ON p.id = s.person_id
             LEFT JOIN teachers t ON p.id = t.person_id
             LEFT JOIN parents pa ON p.id = pa.person_id
             LEFT JOIN classes c ON s.class_id = c.id
             LEFT JOIN departments d ON t.department_id = d.id
             WHERE p.name ILIKE $1
             ORDER BY p.created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(format!("%{}%", s))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, PersonWithRelations>(
            "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                    s.student_no, s.class_id, s.enrollment_date, s.status,
                    t.employee_no, t.department_id, t.title, t.hire_date,
                    pa.wechat_openid, pa.occupation,
                    c.name as class_name, d.name as department_name
             FROM persons p
             LEFT JOIN students s ON p.id = s.person_id
             LEFT JOIN teachers t ON p.id = t.person_id
             LEFT JOIN parents pa ON p.id = pa.person_id
             LEFT JOIN classes c ON s.class_id = c.id
             LEFT JOIN departments d ON t.department_id = d.id
             ORDER BY p.created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    let items: Vec<PersonResponse> = rows.into_iter().map(|row| {
        let mut response = row.into_response();
        // 如果是老师，暂时返回空的班级列表，详细信息通过get_person接口获取
        if let PersonResponse::Teacher(ref mut teacher_response) = response {
            teacher_response.classes = Vec::new();
        }
        response
    }).collect();

    Ok((items, total))
}

async fn get_person(pool: &sqlx::PgPool, id: Uuid) -> Result<PersonResponse, AppError> {
    println!("=== GET_PERSON DEBUG ===");
    println!("Fetching person with ID: {}", id);
    
    let row = sqlx::query_as::<_, PersonWithRelations>(
        "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email, p.type, 
                s.student_no, s.class_id, s.enrollment_date, s.status,
                t.employee_no, t.department_id, t.title, t.hire_date,
                pa.wechat_openid, pa.occupation,
                c.name as class_name, d.name as department_name
         FROM persons p
         LEFT JOIN students s ON p.id = s.person_id
         LEFT JOIN teachers t ON p.id = t.person_id
         LEFT JOIN parents pa ON p.id = pa.person_id
         LEFT JOIN classes c ON s.class_id = c.id
         LEFT JOIN departments d ON t.department_id = d.id
         WHERE p.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let row = match row {
        Some(row) => {
            println!("Person found in database");
            println!("Person data: {:?}", row);
            row
        },
        None => {
            println!("Person not found in database");
            return Err(AppError::NotFound);
        }
    };

    println!("Converting row to response...");
    let mut response = row.into_response();
    
    // 如果是老师，获取其关联的多个班级信息
    if let PersonResponse::Teacher(ref mut teacher_response) = response {
        println!("Fetching classes for teacher...");
        let classes: Vec<_> = sqlx::query!(
            "SELECT tc.class_id, c.name as class_name, tc.is_main_teacher
             FROM teacher_class tc
             LEFT JOIN classes c ON tc.class_id = c.id
             WHERE tc.teacher_id = $1",
            id
        )
        .fetch_all(pool)
        .await?;
        
        println!("Found {} classes for teacher", classes.len());
        
        teacher_response.classes = classes.into_iter().map(|c| {
            crate::models::person::TeacherClassInfo {
                class_id: c.class_id,
                class_name: c.class_name,
                is_main_teacher: c.is_main_teacher.unwrap_or(false)
            }
        }).collect();
    }
    
    println!("Response created successfully");
    Ok(response)
}

async fn create_person(
    pool: &sqlx::PgPool,
    payload: PersonCreate,
) -> Result<PersonResponse, AppError> {
    let mut tx = pool.begin().await?;

    // 打印payload.name的值，检查是否正确
    println!("Received payload.name: '{}'", payload.name);
    println!("Payload.name type: {:?}", std::any::type_name::<String>());

    let person_id = Uuid::new_v4();

    // 转换日期字符串为NaiveDate
    let birthday = payload.birthday.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
    let enrollment_date = payload.enrollment_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
    let hire_date = payload.hire_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());

    // 根据人员类型确定username
    let username = match payload.type_.as_str() {
        "student" => {
            let student_no = payload.student_no.as_ref().ok_or_else(|| {
                AppError::InvalidInput("student_no is required for student".to_string())
            })?;
            if student_no.trim().is_empty() {
                return Err(AppError::InvalidInput("student_no cannot be empty".to_string()));
            }
            student_no.clone()
        }
        "teacher" => {
            let employee_no = payload.employee_no.as_ref().ok_or_else(|| {
                AppError::InvalidInput("employee_no is required for teacher".to_string())
            })?;
            if employee_no.trim().is_empty() {
                return Err(AppError::InvalidInput("employee_no cannot be empty".to_string()));
            }
            employee_no.clone()
        }
        "parent" => {
            // 家长使用手机号作为username，如果没有手机号则使用UUID
            payload.phone.clone().unwrap_or_else(|| person_id.to_string())
        }
        _ => {
            return Err(AppError::InvalidInput("Invalid person type".to_string()));
        }
    };

    // 生成密码哈希：如果提供了密码则使用提供的密码，否则使用默认密码123456
    let password_to_hash = payload.password.as_deref().unwrap_or("123456");
    let password_hash = hash_password(password_to_hash)
        .map_err(|_| AppError::Internal)?;

    sqlx::query(
        "INSERT INTO persons (id, name, username, password_hash, gender, birthday, phone, email, type) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
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
    .execute(&mut *tx)
    .await?;

    match payload.type_.as_str() {
        "student" => {
            let student_no = payload.student_no.ok_or_else(|| {
                AppError::InvalidInput("student_no is required for student".to_string())
            })?;
            sqlx::query(
                "INSERT INTO students (person_id, student_no, class_id, enrollment_date, status)
                 VALUES ($1, $2, $3, $4, 'enrolled')",
            )
            .bind(person_id)
            .bind(student_no)
            .bind(payload.class_id)
            .bind(enrollment_date)
            .execute(&mut *tx)
            .await?;
        }
        "teacher" => {
            let employee_no = payload.employee_no.ok_or_else(|| {
                AppError::InvalidInput("employee_no is required for teacher".to_string())
            })?;
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
                    
                    // 如果是班主任，更新classes表的teacher_id字段
                    if class.is_main_teacher {
                        // 更新该班级的班主任
                        sqlx::query(
                            "UPDATE classes SET teacher_id = $1 WHERE id = $2",
                        )
                        .bind(person_id)
                        .bind(class.class_id)
                        .execute(&mut *tx)
                        .await?;
                        
                        // 清除该班级其他老师的班主任标志
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
        _ => {
            return Err(AppError::InvalidInput("Invalid person type".to_string()));
        }
    }

    tx.commit().await?;

    get_person(pool, person_id).await
}

async fn update_person(
    pool: &sqlx::PgPool,
    id: Uuid,
    payload: PersonUpdate,
) -> Result<PersonResponse, AppError> {
    println!("=== UPDATE PERSON DEBUG ===");
    println!("Person ID: {}", id);
    println!("Payload: {:?}", payload);
    
    // TODO: 添加权限检查逻辑
    // 1. 从请求中获取当前用户信息
    // 2. 检查用户角色是否为管理员
    // 3. 如果不是管理员，检查用户是否为班主任
    // 4. 如果既不是管理员也不是班主任，返回权限错误
    
    let mut tx = match pool.begin().await {
        Ok(tx) => {
            println!("Transaction started successfully");
            tx
        },
        Err(e) => {
            println!("Failed to start transaction: {:?}", e);
            return Err(AppError::Database(e));
        }
    };

    println!("Fetching person data...");
    let person = match sqlx::query_as::<_, Person>("SELECT * FROM persons WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await {
        Ok(Some(p)) => {
            println!("Person found: {:?}", p);
            p
        },
        Ok(None) => {
            println!("Person not found");
            return Err(AppError::NotFound);
        },
        Err(e) => {
            println!("Failed to fetch person: {:?}", e);
            return Err(AppError::Database(e));
        }
    };

    if let Some(name) = payload.name {
        println!("Updating name to: {}", name);
        match sqlx::query("UPDATE persons SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(&mut *tx)
            .await {
            Ok(result) => println!("Name update successful, rows affected: {}", result.rows_affected()),
            Err(e) => {
                println!("Name update failed: {:?}", e);
                return Err(AppError::Database(e));
            }
        };
    }
    if let Some(gender) = payload.gender {
        println!("Updating gender to: {} (converted to i16: {})
", gender, gender as i16);
        match sqlx::query("UPDATE persons SET gender = $1 WHERE id = $2")
            .bind(gender as i16)
            .bind(id)
            .execute(&mut *tx)
            .await {
            Ok(result) => println!("Gender update successful, rows affected: {}", result.rows_affected()),
            Err(e) => {
                println!("Gender update failed: {:?}", e);
                return Err(AppError::Database(e));
            }
        };
    }
    if let Some(birthday_str) = payload.birthday {
        if !birthday_str.is_empty() {
            if let Ok(birthday) = chrono::NaiveDate::parse_from_str(&birthday_str, "%Y-%m-%d") {
                sqlx::query("UPDATE persons SET birthday = $1 WHERE id = $2")
                    .bind(birthday)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        } else {
            sqlx::query("UPDATE persons SET birthday = NULL WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }
    if payload.phone.is_some() {
        sqlx::query("UPDATE persons SET phone = $1 WHERE id = $2")
            .bind(payload.phone)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    if payload.email.is_some() {
        sqlx::query("UPDATE persons SET email = $1 WHERE id = $2")
            .bind(payload.email)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    // 更新密码（如果提供了）
    if let Some(password) = payload.password.as_ref() {
        if !password.is_empty() {
            let password_hash = hash_password(password)
                .map_err(|_| AppError::Internal)?;
            sqlx::query("UPDATE persons SET password_hash = $1 WHERE id = $2")
                .bind(password_hash)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }

    match person.type_.as_str() {
        "student" => {
            println!("Processing student updates...");
            if let Some(student_no) = payload.student_no.as_ref() {
                println!("Updating student_no to: {:?}", student_no);
                // 同时更新students表和persons表的username
                match sqlx::query("UPDATE students SET student_no = $1 WHERE person_id = $2")
                    .bind(student_no)
                    .bind(id)
                    .execute(&mut *tx)
                    .await {
                    Ok(result) => println!("Student_no update successful, rows affected: {}", result.rows_affected()),
                    Err(e) => {
                        println!("Student_no update failed: {:?}", e);
                        return Err(AppError::Database(e));
                    }
                }
                // 同步更新persons表的username
                match sqlx::query("UPDATE persons SET username = $1 WHERE id = $2")
                    .bind(student_no)
                    .bind(id)
                    .execute(&mut *tx)
                    .await {
                    Ok(result) => println!("Username update successful, rows affected: {}", result.rows_affected()),
                    Err(e) => {
                        println!("Username update failed: {:?}", e);
                        return Err(AppError::Database(e));
                    }
                }
            }
            if payload.class_id.is_some() {
                println!("Updating class_id to: {:?}", payload.class_id);
                match sqlx::query("UPDATE students SET class_id = $1 WHERE person_id = $2")
                    .bind(payload.class_id)
                    .bind(id)
                    .execute(&mut *tx)
                    .await {
                    Ok(result) => println!("Class_id update successful, rows affected: {}", result.rows_affected()),
                    Err(e) => {
                        println!("Class_id update failed: {:?}", e);
                        return Err(AppError::Database(e));
                    }
                }
            }
            if let Some(enrollment_date_str) = payload.enrollment_date {
                println!("Processing enrollment_date: {:?}", enrollment_date_str);
                if !enrollment_date_str.is_empty() {
                    println!("Parsing enrollment_date string: {}", enrollment_date_str);
                    match chrono::NaiveDate::parse_from_str(&enrollment_date_str, "%Y-%m-%d") {
                        Ok(enrollment_date) => {
                            println!("Parsed enrollment_date: {}", enrollment_date);
                            match sqlx::query("UPDATE students SET enrollment_date = $1 WHERE person_id = $2")
                                .bind(enrollment_date)
                                .bind(id)
                                .execute(&mut *tx)
                                .await {
                                Ok(result) => println!("Enrollment_date update successful, rows affected: {}", result.rows_affected()),
                                Err(e) => {
                                    println!("Enrollment_date update failed: {:?}", e);
                                    return Err(AppError::Database(e));
                                }
                            }
                        },
                        Err(e) => println!("Failed to parse enrollment_date string: {:?}", e),
                    }
                } else {
                    println!("Setting enrollment_date to NULL");
                    match sqlx::query("UPDATE students SET enrollment_date = NULL WHERE person_id = $1")
                        .bind(id)
                        .execute(&mut *tx)
                        .await {
                        Ok(result) => println!("Enrollment_date set to NULL successful, rows affected: {}", result.rows_affected()),
                        Err(e) => {
                            println!("Enrollment_date NULL update failed: {:?}", e);
                            return Err(AppError::Database(e));
                        }
                    }
                }
            }
        }
        "teacher" => {
            if let Some(employee_no) = payload.employee_no.as_ref() {
                // 同时更新teachers表和persons表的username
                sqlx::query("UPDATE teachers SET employee_no = $1 WHERE person_id = $2")
                    .bind(employee_no)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                // 同步更新persons表的username
                sqlx::query("UPDATE persons SET username = $1 WHERE id = $2")
                    .bind(employee_no)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
            if payload.department_id.is_some() {
                sqlx::query("UPDATE teachers SET department_id = $1 WHERE person_id = $2")
                    .bind(payload.department_id)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
            if payload.title.is_some() {
                sqlx::query("UPDATE teachers SET title = $1 WHERE person_id = $2")
                    .bind(payload.title)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(hire_date_str) = payload.hire_date {
                if !hire_date_str.is_empty() {
                    if let Ok(hire_date) = chrono::NaiveDate::parse_from_str(&hire_date_str, "%Y-%m-%d") {
                        sqlx::query("UPDATE teachers SET hire_date = $1 WHERE person_id = $2")
                            .bind(hire_date)
                            .bind(id)
                            .execute(&mut *tx)
                            .await?;
                    }
                } else {
                    sqlx::query("UPDATE teachers SET hire_date = NULL WHERE person_id = $1")
                        .bind(id)
                        .execute(&mut *tx)
                        .await?;
                }
            }
            
            // 处理老师与班级的关联
            if let Some(classes) = payload.classes {
                // 先删除现有的关联关系
                sqlx::query("DELETE FROM teacher_class WHERE teacher_id = $1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                
                // 插入新的关联关系
                for class in classes {
                    sqlx::query(
                        "INSERT INTO teacher_class (teacher_id, class_id, is_main_teacher)
                         VALUES ($1, $2, $3)",
                    )
                    .bind(id)
                    .bind(class.class_id)
                    .bind(class.is_main_teacher)
                    .execute(&mut *tx)
                    .await?;
                    
                    // 如果是班主任，更新classes表的teacher_id字段
                    if class.is_main_teacher {
                        // 更新该班级的班主任
                        sqlx::query(
                            "UPDATE classes SET teacher_id = $1 WHERE id = $2",
                        )
                        .bind(id)
                        .bind(class.class_id)
                        .execute(&mut *tx)
                        .await?;
                        
                        // 清除该班级其他老师的班主任标志
                        sqlx::query(
                            "UPDATE teacher_class SET is_main_teacher = false 
                             WHERE class_id = $1 AND teacher_id != $2",
                        )
                        .bind(class.class_id)
                        .bind(id)
                        .execute(&mut *tx)
                        .await?;
                    }
                }
            }
        }
        "parent" => {
            if payload.wechat_openid.is_some() {
                sqlx::query("UPDATE parents SET wechat_openid = $1 WHERE person_id = $2")
                    .bind(payload.wechat_openid)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
            if payload.occupation.is_some() {
                sqlx::query("UPDATE parents SET occupation = $1 WHERE person_id = $2")
                    .bind(payload.occupation)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        _ => {}
    }

    println!("About to commit transaction...");
    match tx.commit().await {
        Ok(_) => {
            println!("Transaction committed successfully");
        },
        Err(e) => {
            println!("Failed to commit transaction: {:?}", e);
            return Err(AppError::Database(e));
        }
    }

    println!("Fetching updated person data...");
    match get_person(pool, id).await {
        Ok(person) => {
            println!("Update completed successfully, returning person data");
            Ok(person)
        },
        Err(e) => {
            println!("Failed to fetch updated person data: {:?}", e);
            Err(e)
        }
    }
}

async fn delete_person(pool: &sqlx::PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM persons WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct PersonWithRelations {
    id: Uuid,
    name: String,
    gender: i16,
    birthday: Option<chrono::NaiveDate>,
    phone: Option<String>,
    email: Option<String>,
    #[sqlx(rename = "type")]
    type_: String,
    student_no: Option<String>,
    class_id: Option<Uuid>,
    enrollment_date: Option<chrono::NaiveDate>,
    status: Option<String>,
    employee_no: Option<String>,
    department_id: Option<Uuid>,
    title: Option<String>,
    hire_date: Option<chrono::NaiveDate>,
    wechat_openid: Option<String>,
    occupation: Option<String>,
    class_name: Option<String>,
    department_name: Option<String>,
}

impl PersonWithRelations {
    fn into_response(self) -> PersonResponse {
        println!("=== INTO_RESPONSE DEBUG ===");
        println!("Person type: {}", self.type_);
        println!("Student no: {:?}", self.student_no);
        println!("Status: {:?}", self.status);
        
        match self.type_.as_str() {
            "student" => PersonResponse::Student(StudentResponse {
                id: self.id,
                name: self.name,
                gender: self.gender,
                birthday: self.birthday,
                phone: self.phone,
                email: self.email,
                student_no: self.student_no.unwrap_or_default(),
                class_id: self.class_id,
                class_name: self.class_name,
                enrollment_date: self.enrollment_date,
                status: self.status.unwrap_or_else(|| "enrolled".to_string()),
            }),
            "teacher" => PersonResponse::Teacher(TeacherResponse {
                id: self.id,
                name: self.name,
                gender: self.gender,
                birthday: self.birthday,
                phone: self.phone,
                email: self.email,
                employee_no: self.employee_no.unwrap_or_default(),
                department_id: self.department_id,
                department_name: self.department_name,
                classes: Vec::new(), // 暂时返回空数组，需要在get_person函数中填充
                title: self.title,
                hire_date: self.hire_date,
            }),
            "parent" => PersonResponse::Parent(ParentResponse {
                id: self.id,
                name: self.name,
                gender: self.gender,
                birthday: self.birthday,
                phone: self.phone,
                email: self.email,
                wechat_openid: self.wechat_openid,
                occupation: self.occupation,
            }),
            _ => PersonResponse::Teacher(TeacherResponse {
                id: self.id,
                name: self.name,
                gender: self.gender,
                birthday: self.birthday,
                phone: self.phone,
                email: self.email,
                employee_no: String::new(),
                department_id: None,
                department_name: None,
                classes: Vec::new(),
                title: None,
                hire_date: None,
            }),
        }
    }
}
