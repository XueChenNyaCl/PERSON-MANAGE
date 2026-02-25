use axum::{extract::State, Json, routing::delete, routing::get, routing::post, routing::put, Router};
use sqlx::PgPool;

use crate::api::{attendance, auth, class, department, debug, notice, person, score};
use crate::core::plugin::PluginManager;

// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub pool: Option<PgPool>,
    pub plugin_manager: PluginManager,
}

pub fn create_router(pool: Option<PgPool>, plugin_manager: PluginManager) -> Router {
    let state = AppState {
        pool,
        plugin_manager,
    };

    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        // 数据库连接状态检查
        .route("/api/db/status", get(db_status_check))
        // 调试路由
        .route("/api/debug/persons", get(debug::debug_persons))
        // 认证路由
        .route("/api/auth/login", post(auth::login))
        // 公开路由
        .route("/api/persons", get(person::list))
        .route("/api/persons/:id", get(person::get))
        .route("/api/classes", get(class::list))
        .route("/api/classes/:id", get(class::get))
        .route("/api/classes/:id/students", get(class::get_class_students))
        .route("/api/classes/:id/teachers", get(class::get_class_teachers))
        .route("/api/departments", get(department::list))
        .route("/api/departments/:id", get(department::get))
        .route("/api/attendance", get(attendance::list))
        .route("/api/score", get(score::list))
        .route("/api/notice", get(notice::list))
        .route("/api/permission/teacher/classes", get(person::get_teacher_classes))
        // 需要认证的路由
        // TODO: 添加认证中间件
        .route("/api/persons", post(person::create))
        .route("/api/persons/:id", put(person::update))
        .route("/api/persons/:id", delete(person::delete))
        .route("/api/classes", post(class::create))
        .route("/api/classes/:id", put(class::update))
        .route("/api/classes/:id", delete(class::delete))
        .route("/api/departments", post(department::create))
        .route("/api/departments/:id", put(department::update))
        .route("/api/departments/:id", delete(department::delete))
        .route("/api/attendance", post(attendance::create))
        .route("/api/score", post(score::create))
        .route("/api/notice", post(notice::create))
        // WebSocket路由
        .route("/ws", get(crate::ws::handler::ws_handler))
        // 注入状态
        .with_state(state)
}

async fn health_check() -> &'static str {
    "Ok"
}

// 数据库状态检查响应
#[derive(Debug, serde::Serialize)]
struct DbStatusResponse {
    status: String,
    message: String,
    details: Option<String>,
}

async fn db_status_check(State(state): State<AppState>) -> Json<DbStatusResponse> {
    match &state.pool {
        Some(pool) => {
            // 尝试执行一个简单的SQL查询来测试连接
            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_) => Json(DbStatusResponse {
                    status: "ok".to_string(),
                    message: "Database connection is active".to_string(),
                    details: Some("Successfully executed test query".to_string()),
                }),
                Err(e) => Json(DbStatusResponse {
                    status: "error".to_string(),
                    message: "Database connection exists but query failed".to_string(),
                    details: Some(format!("Error: {}", e)),
                }),
            }
        }
        None => Json(DbStatusResponse {
            status: "error".to_string(),
            message: "Database connection pool not initialized".to_string(),
            details: Some("Check database configuration and ensure PostgreSQL is running".to_string()),
        }),
    }
}
