use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;
use crate::models::group::*;

// 小组列表（按班级）
pub async fn list(
    State(state): State<AppState>,
    Path(class_id): Path<Uuid>,
) -> Result<Json<Vec<GroupResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let groups = list_groups(&pool, class_id).await?;
    Ok(Json(groups))
}

// 所有小组列表
pub async fn list_all(
    State(state): State<AppState>,
) -> Result<Json<Vec<GroupResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let groups = list_all_groups(&pool).await?;
    Ok(Json(groups))
}

// 创建小组
pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<GroupCreate>,
) -> Result<Json<GroupResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    // 解析班级ID
    let class_id = Uuid::parse_str(&payload.class_id)
        .map_err(|_| AppError::InvalidInput("无效的班级ID".to_string()))?;
    
    // 检查班级特定权限：group.create.{class_suffix}
    let manager = PermissionManager::new(pool.clone());
    manager.require_class_permission(user_id, "group.create", class_id).await?;
    
    let group = create_group(&pool, payload).await?;
    Ok(Json(group))
}

// 获取小组详情
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<GroupResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let group = get_group(&pool, id).await?;
    Ok(Json(group))
}

// 更新小组
pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<GroupUpdate>,
) -> Result<Json<GroupResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    // 获取小组所属班级ID
    let group_row = sqlx::query_as::<_, GroupClassRow>("SELECT class_id FROM class_groups WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;
    
    // 检查班级特定权限：group.update.{class_suffix}
    let manager = PermissionManager::new(pool.clone());
    manager.require_class_permission(user_id, "group.update", group_row.class_id).await?;
    
    let updated_group = update_group(&pool, id, payload).await?;
    Ok(Json(updated_group))
}

// 删除小组
pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    // 获取小组所属班级ID
    let group_row = sqlx::query_as::<_, GroupClassRow>("SELECT class_id FROM class_groups WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;
    
    // 检查班级特定权限：group.delete.{class_suffix}
    let manager = PermissionManager::new(pool.clone());
    manager.require_class_permission(user_id, "group.delete", group_row.class_id).await?;
    
    delete_group(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// 获取小组成员
pub async fn get_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupMemberResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let members = get_group_members(&pool, id).await?;
    Ok(Json(members))
}

// 添加成员
pub async fn add_member(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<GroupMemberAdd>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    // 获取小组所属班级ID
    let group_row = sqlx::query_as::<_, GroupClassRow>("SELECT class_id FROM class_groups WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;
    
    // 检查班级特定权限：group.update.member.{class_suffix}
    let manager = PermissionManager::new(pool.clone());
    manager.require_class_permission(user_id, "group.update.member", group_row.class_id).await?;
    
    add_group_member(&pool, id, &payload.person_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// 移除成员
pub async fn remove_member(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, person_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    // 获取小组所属班级ID
    let group_row = sqlx::query_as::<_, GroupClassRow>("SELECT class_id FROM class_groups WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;
    
    // 检查班级特定权限：group.update.member.{class_suffix}
    let manager = PermissionManager::new(pool.clone());
    manager.require_class_permission(user_id, "group.update.member", group_row.class_id).await?;
    
    remove_group_member(&pool, id, person_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// 小组加分减分
pub async fn update_score(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<GroupScoreChange>,
) -> Result<Json<GroupScoreRecord>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    // 获取小组所属班级ID
    let group_row = sqlx::query_as::<_, GroupClassRow>("SELECT class_id FROM class_groups WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;
    
    // 检查班级特定权限：group.update.score.{class_suffix}
    let manager = PermissionManager::new(pool.clone());
    manager.require_class_permission(user_id, "group.update.score", group_row.class_id).await?;
    
    let record = update_group_score(&pool, id, user_id, payload).await?;
    Ok(Json(record))
}

// 获取小组积分记录
pub async fn get_score_records(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupScoreRecord>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let records = get_group_score_records(&pool, id).await?;
    Ok(Json(records))
}

// 用于查询小组班级ID的辅助结构体
#[derive(Debug, sqlx::FromRow)]
struct GroupClassRow {
    class_id: Uuid,
}

// 数据库操作函数

async fn list_groups(pool: &sqlx::PgPool, class_id: Uuid) -> Result<Vec<GroupResponse>, AppError> {
    let rows: Vec<GroupListRow> = sqlx::query_as(
        "SELECT g.id, g.class_id, g.name, g.description, g.score, g.created_at, g.updated_at, 
                c.name as class_name,
                COUNT(gm.id) as member_count
         FROM class_groups g
         LEFT JOIN classes c ON g.class_id = c.id
         LEFT JOIN group_members gm ON g.id = gm.group_id
         WHERE g.class_id = $1
         GROUP BY g.id, c.name
         ORDER BY g.created_at DESC"
    )
    .bind(class_id)
    .fetch_all(pool)
    .await?;
    
    let groups: Vec<GroupResponse> = rows.into_iter().map(|row| GroupResponse {
        id: row.id,
        class_id: row.class_id,
        class_name: row.class_name,
        name: row.name,
        description: row.description,
        score: row.score,
        member_count: row.member_count.unwrap_or(0),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }).collect();
    
    Ok(groups)
}

async fn list_all_groups(pool: &sqlx::PgPool) -> Result<Vec<GroupResponse>, AppError> {
    let rows: Vec<GroupListRow> = sqlx::query_as(
        "SELECT g.id, g.class_id, g.name, g.description, g.score, g.created_at, g.updated_at, 
                c.name as class_name,
                COUNT(gm.id) as member_count
         FROM class_groups g
         LEFT JOIN classes c ON g.class_id = c.id
         LEFT JOIN group_members gm ON g.id = gm.group_id
         GROUP BY g.id, c.name
         ORDER BY g.created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    
    let groups: Vec<GroupResponse> = rows.into_iter().map(|row| GroupResponse {
        id: row.id,
        class_id: row.class_id,
        class_name: row.class_name,
        name: row.name,
        description: row.description,
        score: row.score,
        member_count: row.member_count.unwrap_or(0),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }).collect();
    
    Ok(groups)
}

#[derive(Debug, sqlx::FromRow)]
struct GroupListRow {
    id: Uuid,
    class_id: Uuid,
    name: String,
    description: Option<String>,
    score: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    class_name: Option<String>,
    member_count: Option<i64>,
}

async fn create_group(pool: &sqlx::PgPool, payload: GroupCreate) -> Result<GroupResponse, AppError> {
    let class_id = Uuid::parse_str(&payload.class_id).map_err(|_| AppError::InvalidInput("无效的班级ID".to_string()))?;
    let id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO class_groups (id, class_id, name, description) 
         VALUES ($1, $2, $3, $4)"
    )
    .bind(id)
    .bind(class_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .execute(pool)
    .await?;
    
    get_group(pool, id).await
}

async fn get_group(pool: &sqlx::PgPool, id: Uuid) -> Result<GroupResponse, AppError> {
    let row: GroupListRow = sqlx::query_as(
        "SELECT g.id, g.class_id, g.name, g.description, g.score, g.created_at, g.updated_at, 
                c.name as class_name,
                COUNT(gm.id) as member_count
         FROM class_groups g
         LEFT JOIN classes c ON g.class_id = c.id
         LEFT JOIN group_members gm ON g.id = gm.group_id
         WHERE g.id = $1
         GROUP BY g.id, c.name"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    
    Ok(GroupResponse {
        id: row.id,
        class_id: row.class_id,
        class_name: row.class_name,
        name: row.name,
        description: row.description,
        score: row.score,
        member_count: row.member_count.unwrap_or(0),
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn update_group(pool: &sqlx::PgPool, id: Uuid, payload: GroupUpdate) -> Result<GroupResponse, AppError> {
    let mut tx = pool.begin().await?;
    
    if let Some(name) = payload.name {
        sqlx::query("UPDATE class_groups SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    
    if let Some(description) = payload.description {
        sqlx::query("UPDATE class_groups SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(description)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    
    tx.commit().await?;
    get_group(pool, id).await
}

async fn delete_group(pool: &sqlx::PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM class_groups WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(())
}

async fn get_group_members(pool: &sqlx::PgPool, group_id: Uuid) -> Result<Vec<GroupMemberResponse>, AppError> {
    let rows: Vec<GroupMemberRow> = sqlx::query_as(
        "SELECT p.id, p.name, p.gender, p.birthday, p.phone, p.email,
                s.student_no, s.class_id, s.enrollment_date, s.status,
                c.name as class_name
         FROM persons p
         JOIN students s ON p.id = s.person_id
         JOIN group_members gm ON p.id = gm.person_id
         LEFT JOIN classes c ON s.class_id = c.id
         WHERE gm.group_id = $1
         ORDER BY p.name"
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;
    
    let members: Vec<GroupMemberResponse> = rows.into_iter().map(|row| GroupMemberResponse {
        id: row.id,
        name: row.name,
        gender: row.gender,
        birthday: row.birthday,
        phone: row.phone,
        email: row.email,
        student_no: row.student_no,
        class_id: row.class_id,
        class_name: row.class_name,
        enrollment_date: row.enrollment_date,
        status: row.status,
    }).collect();
    
    Ok(members)
}

#[derive(Debug, sqlx::FromRow)]
struct GroupMemberRow {
    id: Uuid,
    name: String,
    gender: i16,
    birthday: Option<chrono::NaiveDate>,
    phone: Option<String>,
    email: Option<String>,
    student_no: String,
    class_id: Option<Uuid>,
    class_name: Option<String>,
    enrollment_date: Option<chrono::NaiveDate>,
    status: String,
}

async fn add_group_member(pool: &sqlx::PgPool, group_id: Uuid, person_id_str: &str) -> Result<(), AppError> {
    let person_id = Uuid::parse_str(person_id_str).map_err(|_| AppError::InvalidInput("无效的人员ID".to_string()))?;
    let id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO group_members (id, group_id, person_id)
         VALUES ($1, $2, $3)
         ON CONFLICT (group_id, person_id) DO NOTHING"
    )
    .bind(id)
    .bind(group_id)
    .bind(person_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

async fn remove_group_member(pool: &sqlx::PgPool, group_id: Uuid, person_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM group_members WHERE group_id = $1 AND person_id = $2"
    )
    .bind(group_id)
    .bind(person_id)
    .execute(pool)
    .await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(())
}

async fn update_group_score(pool: &sqlx::PgPool, group_id: Uuid, user_id: Uuid, payload: GroupScoreChange) -> Result<GroupScoreRecord, AppError> {
    let mut tx = pool.begin().await?;
    let record_id = Uuid::new_v4();
    
    // 插入积分记录
    sqlx::query(
        "INSERT INTO group_score_records (id, group_id, score_change, reason, created_by)
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(record_id)
    .bind(group_id)
    .bind(payload.score_change)
    .bind(&payload.reason)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    
    // 更新小组积分
    sqlx::query(
        "UPDATE class_groups SET score = score + $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2"
    )
    .bind(payload.score_change)
    .bind(group_id)
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    // 获取创建的记录
    let record = sqlx::query_as::<_, GroupScoreRecord>(
        "SELECT id, group_id, score_change, reason, created_by, created_at 
         FROM group_score_records WHERE id = $1"
    )
    .bind(record_id)
    .fetch_one(pool)
    .await?;
    
    Ok(record)
}

async fn get_group_score_records(pool: &sqlx::PgPool, group_id: Uuid) -> Result<Vec<GroupScoreRecord>, AppError> {
    let records = sqlx::query_as::<_, GroupScoreRecord>(
        "SELECT id, group_id, score_change, reason, created_by, created_at 
         FROM group_score_records 
         WHERE group_id = $1 
         ORDER BY created_at DESC"
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;
    
    Ok(records)
}
