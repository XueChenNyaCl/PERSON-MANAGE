use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::error::AppError;
use crate::models::department::{
    Department, DepartmentCreate, DepartmentResponse, DepartmentUpdate,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
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
) -> Result<Json<ListResponse<DepartmentResponse>>, AppError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    if let Some(pool) = state.pool {
        let (items, total) = list_departments(&pool, query.search.as_deref(), page, limit).await?;
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
    Json(payload): Json<DepartmentCreate>,
) -> Result<Json<DepartmentResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    let department = create_department(&pool, payload).await?;
    Ok(Json(department))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DepartmentResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::NotFound)?;

    let department = get_department(&pool, id).await?;
    Ok(Json(department))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<DepartmentUpdate>,
) -> Result<Json<DepartmentResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    let department = update_department(&pool, id, payload).await?;
    Ok(Json(department))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    delete_department(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_departments(
    pool: &sqlx::PgPool,
    search: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<(Vec<DepartmentResponse>, i64), AppError> {
    let offset = (page - 1) * limit;

    let total = if let Some(s) = search {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM departments d WHERE d.name ILIKE $1")
            .bind(format!("%{}%", s))
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM departments d")
            .fetch_one(pool)
            .await?
    };

    let rows = if let Some(s) = search {
        sqlx::query_as::<_, DepartmentWithParent>(
            "SELECT d.id, d.name, d.parent_id, d.created_at,
                    p.name as parent_name
             FROM departments d
             LEFT JOIN departments p ON d.parent_id = p.id
             WHERE d.name ILIKE $1
             ORDER BY d.created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(format!("%{}%", s))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, DepartmentWithParent>(
            "SELECT d.id, d.name, d.parent_id, d.created_at,
                    p.name as parent_name
             FROM departments d
             LEFT JOIN departments p ON d.parent_id = p.id
             ORDER BY d.created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    let items: Vec<DepartmentResponse> = rows.into_iter().map(|row| row.into_response()).collect();

    Ok((items, total))
}

async fn get_department(pool: &sqlx::PgPool, id: Uuid) -> Result<DepartmentResponse, AppError> {
    let row = sqlx::query_as::<_, DepartmentWithParent>(
        "SELECT d.id, d.name, d.parent_id, d.created_at,
                p.name as parent_name
         FROM departments d
         LEFT JOIN departments p ON d.parent_id = p.id
         WHERE d.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(row.into_response())
}

async fn create_department(
    pool: &sqlx::PgPool,
    payload: DepartmentCreate,
) -> Result<DepartmentResponse, AppError> {
    let id = Uuid::new_v4();

    // Convert string parent_id to Uuid if provided
    let parent_id = payload.parent_id.and_then(|id_str| Uuid::parse_str(&id_str).ok());

    sqlx::query(
        "INSERT INTO departments (id, name, parent_id)
         VALUES ($1, $2, $3)",
    )
    .bind(id)
    .bind(&payload.name)
    .bind(parent_id)
    .execute(pool)
    .await?;

    get_department(pool, id).await
}

async fn update_department(
    pool: &sqlx::PgPool,
    id: Uuid,
    payload: DepartmentUpdate,
) -> Result<DepartmentResponse, AppError> {
    let _department = sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    if let Some(name) = payload.name {
        sqlx::query("UPDATE departments SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if payload.parent_id.is_some() {
        // Convert string parent_id to Uuid if provided
        let parent_id = payload.parent_id.as_ref().and_then(|id_str| Uuid::parse_str(id_str).ok());
        sqlx::query("UPDATE departments SET parent_id = $1 WHERE id = $2")
            .bind(parent_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    get_department(pool, id).await
}

async fn delete_department(pool: &sqlx::PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM departments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct DepartmentWithParent {
    id: Uuid,
    name: String,
    parent_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    parent_name: Option<String>,
}

impl DepartmentWithParent {
    fn into_response(self) -> DepartmentResponse {
        DepartmentResponse {
            id: self.id,
            name: self.name,
            parent_id: self.parent_id,
            parent_name: self.parent_name,
            created_at: self.created_at,
        }
    }
}
