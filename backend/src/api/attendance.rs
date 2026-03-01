use axum::{
    extract::{Query, State, Path, Extension},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::error::AppError;
use crate::core::auth::Claims;
use crate::core::permission::PermissionManager;

#[derive(Debug, Deserialize)]
pub struct AttendanceQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub person_id: Option<Uuid>,
    pub date: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAttendanceRequest {
    pub person_id: Uuid,
    pub date: String,
    pub status: String,
    pub time: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAttendanceRequest {
    pub status: Option<String>,
    pub time: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AttendanceResponse {
    pub id: Uuid,
    pub person_id: Uuid,
    pub person_name: String,
    pub date: String,
    pub status: String,
    pub time: Option<String>,
    pub remark: Option<String>,
    pub created_at: String,
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
    Query(query): Query<AttendanceQuery>,
) -> Result<Json<ListResponse<AttendanceResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;
    
    // 解析日期参数
    let parsed_date = if let Some(ref date_str) = query.date {
        Some(chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| AppError::InvalidInput("无效的日期格式".to_string()))?)
    } else {
        None
    };
    println!("DEBUG: query.date = {:?}, parsed_date = {:?}", query.date, parsed_date);

    // 构建查询条件
    let mut conditions = vec!["1=1".to_string()];
    let mut param_index = 1;
    
    if query.person_id.is_some() {
        conditions.push(format!("a.person_id = ${}", param_index));
        param_index += 1;
    }
    
    if parsed_date.is_some() {
        conditions.push(format!("a.date = ${}", param_index));
        param_index += 1;
    }
    
    if query.status.is_some() {
        conditions.push(format!("a.status = ${}", param_index));
        param_index += 1;
    }
    
    let where_clause = conditions.join(" AND ");
    
    // 查询总数
    let count_sql = format!("SELECT COUNT(*) FROM attendances a WHERE {}", where_clause);
    let mut count_query = sqlx::query_scalar(&count_sql);
    
    if let Some(ref person_id) = query.person_id {
        count_query = count_query.bind(person_id);
    }
    if let Some(ref date) = parsed_date {
        count_query = count_query.bind(date);
    }
    if let Some(ref status) = query.status {
        count_query = count_query.bind(status);
    }
    
    let total: i64 = count_query
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    // 查询数据
    let sql = format!(
        "SELECT a.id, a.person_id, p.name as person_name, a.date, a.status, a.time, a.remark, a.created_at 
         FROM attendances a 
         JOIN persons p ON a.person_id = p.id 
         WHERE {} 
         ORDER BY a.date DESC, a.created_at DESC 
         LIMIT ${} OFFSET ${}",
        where_clause,
        param_index,
        param_index + 1
    );
    
    let mut data_query = sqlx::query_as::<_, AttendanceRow>(&sql);
    
    if let Some(ref person_id) = query.person_id {
        data_query = data_query.bind(person_id);
    }
    if let Some(ref date) = parsed_date {
        data_query = data_query.bind(date);
    }
    if let Some(ref status) = query.status {
        data_query = data_query.bind(status);
    }
    
    let attendances = data_query
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    let items: Vec<AttendanceResponse> = attendances.into_iter().map(|row| row.into()).collect();
    
    Ok(Json(ListResponse {
        items,
        total,
        page,
        limit,
    }))
}

#[derive(sqlx::FromRow)]
struct AttendanceRow {
    id: Uuid,
    person_id: Uuid,
    person_name: String,
    date: chrono::NaiveDate,
    status: String,
    time: Option<chrono::NaiveTime>,
    remark: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AttendanceRow> for AttendanceResponse {
    fn from(row: AttendanceRow) -> Self {
        Self {
            id: row.id,
            person_id: row.person_id,
            person_name: row.person_name,
            date: row.date.to_string(),
            status: row.status,
            time: row.time.map(|t| t.to_string()),
            remark: row.remark,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateAttendanceRequest>,
) -> Result<Json<AttendanceResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "attendance.create").await?;
    
    // 解析日期
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d")
        .map_err(|_| AppError::InvalidInput("无效的日期格式".to_string()))?;
    
    // 解析时间
    let time = match req.time {
        Some(t) => Some(
            chrono::NaiveTime::parse_from_str(&t, "%H:%M:%S")
                .map_err(|_| AppError::InvalidInput("无效的时间格式".to_string()))?
        ),
        None => None,
    };
    
    // 插入数据
    let row = sqlx::query_as::<_, AttendanceRow>(
        "INSERT INTO attendances (person_id, date, status, time, remark, created_by) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, person_id, (SELECT name FROM persons WHERE id = $1) as person_name, 
         date, status, time, remark, created_at"
    )
    .bind(req.person_id)
    .bind(date)
    .bind(req.status)
    .bind(time)
    .bind(req.remark)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e))?;
    
    Ok(Json(row.into()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AttendanceResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let row = sqlx::query_as::<_, AttendanceRow>(
        "SELECT a.id, a.person_id, p.name as person_name, a.date, a.status, a.time, a.remark, a.created_at 
         FROM attendances a 
         JOIN persons p ON a.person_id = p.id 
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or(AppError::NotFound)?;
    
    Ok(Json(row.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAttendanceRequest>,
) -> Result<Json<AttendanceResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "attendance.update").await?;
    
    // 构建更新字段
    let mut updates = vec![];
    let mut param_index = 1;
    
    if req.status.is_some() {
        updates.push(format!("status = ${}", param_index));
        param_index += 1;
    }
    
    if req.time.is_some() {
        updates.push(format!("time = ${}", param_index));
        param_index += 1;
    }
    
    if req.remark.is_some() {
        updates.push(format!("remark = ${}", param_index));
        param_index += 1;
    }
    
    if updates.is_empty() {
        return Err(AppError::InvalidInput("没有要更新的字段".to_string()));
    }
    
    let sql = format!(
        "UPDATE attendances SET {} WHERE id = ${} 
         RETURNING id, person_id, (SELECT name FROM persons WHERE id = attendances.person_id) as person_name, 
         date, status, time, remark, created_at",
        updates.join(", "),
        param_index
    );
    
    let mut query = sqlx::query_as::<_, AttendanceRow>(&sql);
    
    if let Some(status) = req.status {
        query = query.bind(status);
    }
    
    if let Some(time_str) = req.time {
        let time = chrono::NaiveTime::parse_from_str(&time_str, "%H:%M:%S")
            .map_err(|_| AppError::InvalidInput("无效的时间格式".to_string()))?;
        query = query.bind(time);
    }
    
    if let Some(remark) = req.remark {
        query = query.bind(remark);
    }
    
    let row = query
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or(AppError::NotFound)?;
    
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "attendance.delete").await?;
    
    let result = sqlx::query("DELETE FROM attendances WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(axum::http::StatusCode::NO_CONTENT)
}
