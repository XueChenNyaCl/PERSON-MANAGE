use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;

#[derive(Debug, Deserialize)]
pub struct PageContextRequest {
    pub page: String,
    pub path: String,
    pub params: Option<serde_json::Value>,
    pub query: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PageContextResponse {
    pub page: String,
    pub data: serde_json::Value,
}

pub async fn get_page_context(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PageContextRequest>,
) -> Result<Json<PageContextResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    let _ = (&req.path, &req.params, &req.query, user_id);

    let data = match req.page.as_str() {
        "dashboard" => get_dashboard_context(&pool).await?,
        "person" => get_person_context(&pool).await?,
        "attendance" => get_attendance_context(&pool).await?,
        "notice" => get_notice_context(&pool).await?,
        "class" => get_class_context(&pool).await?,
        "group" => get_group_context(&pool).await?,
        _ => serde_json::json!({ "page": req.page }),
    };

    Ok(Json(PageContextResponse {
        page: req.page,
        data,
    }))
}

async fn get_dashboard_context(pool: &sqlx::PgPool) -> Result<serde_json::Value, AppError> {
    let person_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM persons")
        .fetch_one(pool)
        .await?;
    let class_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM classes")
        .fetch_one(pool)
        .await?;
    let group_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM class_groups")
        .fetch_one(pool)
        .await?;
    let notice_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notices")
        .fetch_one(pool)
        .await?;

    Ok(serde_json::json!({
        "page": "dashboard",
        "summary": {
            "total_persons": person_count,
            "total_classes": class_count,
            "total_groups": group_count,
            "total_notices": notice_count
        }
    }))
}

async fn get_person_context(pool: &sqlx::PgPool) -> Result<serde_json::Value, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM persons")
        .fetch_one(pool)
        .await?;
    let no_phone: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM persons WHERE phone IS NULL")
        .fetch_one(pool)
        .await?;
    let no_email: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM persons WHERE email IS NULL")
        .fetch_one(pool)
        .await?;

    let classes = sqlx::query(
        "SELECT c.id, c.name,
                (SELECT COUNT(*) FROM students s WHERE s.class_id = c.id) AS student_count
         FROM classes c
         ORDER BY c.name
         LIMIT 5"
    )
    .fetch_all(pool)
    .await?;

    let classes_data = classes.into_iter().map(|row| {
        serde_json::json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "name": row.get::<String, _>("name"),
            "student_count": row.get::<Option<i64>, _>("student_count").unwrap_or(0)
        })
    }).collect::<Vec<_>>();

    Ok(serde_json::json!({
        "page": "person",
        "stats": {
            "total": total,
            "no_phone": no_phone,
            "no_email": no_email,
            "incomplete_info": no_phone + no_email
        },
        "classes": classes_data
    }))
}

async fn get_attendance_context(pool: &sqlx::PgPool) -> Result<serde_json::Value, AppError> {
    let today = chrono::Local::now().date_naive();

    let stats = sqlx::query(
        "SELECT
            COUNT(*)::bigint AS total,
            SUM(CASE WHEN status = 'present' THEN 1 ELSE 0 END)::bigint AS present,
            SUM(CASE WHEN status = 'absent' THEN 1 ELSE 0 END)::bigint AS absent,
            SUM(CASE WHEN status = 'late' THEN 1 ELSE 0 END)::bigint AS late,
            SUM(CASE WHEN status = 'early_leave' THEN 1 ELSE 0 END)::bigint AS early_leave,
            SUM(CASE WHEN status = 'excused' THEN 1 ELSE 0 END)::bigint AS excused
         FROM attendances
         WHERE date = $1"
    )
    .bind(today)
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "page": "attendance",
        "current_date": today.to_string(),
        "today_stats": {
            "total": stats.get::<Option<i64>, _>("total").unwrap_or(0),
            "present": stats.get::<Option<i64>, _>("present").unwrap_or(0),
            "absent": stats.get::<Option<i64>, _>("absent").unwrap_or(0),
            "late": stats.get::<Option<i64>, _>("late").unwrap_or(0),
            "early_leave": stats.get::<Option<i64>, _>("early_leave").unwrap_or(0),
            "excused": stats.get::<Option<i64>, _>("excused").unwrap_or(0)
        }
    }))
}

async fn get_notice_context(pool: &sqlx::PgPool) -> Result<serde_json::Value, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notices")
        .fetch_one(pool)
        .await?;

    let recent = sqlx::query(
        "SELECT id, title, created_at
         FROM notices
         ORDER BY created_at DESC
         LIMIT 3"
    )
    .fetch_all(pool)
    .await?;

    let recent_data = recent.into_iter().map(|row| {
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        serde_json::json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "title": row.get::<String, _>("title"),
            "created_at": created_at.to_rfc3339()
        })
    }).collect::<Vec<_>>();

    Ok(serde_json::json!({
        "page": "notice",
        "stats": { "total": total },
        "recent_notices": recent_data
    }))
}

async fn get_class_context(pool: &sqlx::PgPool) -> Result<serde_json::Value, AppError> {
    let total_classes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM classes")
        .fetch_one(pool)
        .await?;

    let classes = sqlx::query(
        "SELECT id, name, grade
         FROM classes
         ORDER BY grade, name
         LIMIT 5"
    )
    .fetch_all(pool)
    .await?;

    let classes_data = classes.into_iter().map(|row| {
        serde_json::json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "name": row.get::<String, _>("name"),
            "grade": row.get::<i16, _>("grade")
        })
    }).collect::<Vec<_>>();

    Ok(serde_json::json!({
        "page": "class",
        "classes": classes_data,
        "stats": { "total_classes": total_classes }
    }))
}

async fn get_group_context(pool: &sqlx::PgPool) -> Result<serde_json::Value, AppError> {
    let total_groups: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM class_groups")
        .fetch_one(pool)
        .await?;

    let groups = sqlx::query(
        "SELECT cg.id, cg.name, c.name AS class_name,
                (SELECT COUNT(*) FROM group_members gm WHERE gm.group_id = cg.id) AS member_count
         FROM class_groups cg
         LEFT JOIN classes c ON cg.class_id = c.id
         ORDER BY cg.name
         LIMIT 5"
    )
    .fetch_all(pool)
    .await?;

    let groups_data = groups.into_iter().map(|row| {
        serde_json::json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "name": row.get::<String, _>("name"),
            "class_name": row.get::<Option<String>, _>("class_name"),
            "member_count": row.get::<Option<i64>, _>("member_count").unwrap_or(0)
        })
    }).collect::<Vec<_>>();

    Ok(serde_json::json!({
        "page": "group",
        "groups": groups_data,
        "stats": { "total_groups": total_groups }
    }))
}

use sqlx::Row;