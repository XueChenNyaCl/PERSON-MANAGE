use std::path::PathBuf;

use axum::{
    extract::State,
    http::{StatusCode, Uri},
    middleware,
    response::{Html, IntoResponse},
    routing::delete,
    routing::get,
    routing::post,
    routing::put,
    Json, Router,
};
use sqlx::PgPool;
use tower_http::services::ServeDir;

use std::sync::Arc;

use crate::api::{
    ai, ai_actions, ai_assistant, ai_context_provider, ai_data, ai_enhanced, attendance, auth,
    chat, class, debug, department, group, monitor, notice, permission, person, score,
    special_user,
};
use crate::core::database_service::DatabaseService;
use crate::core::middleware::auth_middleware;
use crate::core::plugin::PluginManager;
use crate::core::redis::RedisMonitor;

// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub pool: Option<PgPool>,
    pub db_service: Option<DatabaseService>,
    pub redis_monitor: Option<Arc<RedisMonitor>>,
    pub static_index_path: PathBuf,
    #[allow(dead_code)]
    pub plugin_manager: PluginManager,
}

pub fn create_router(
    pool: Option<PgPool>,
    db_service: Option<DatabaseService>,
    redis_monitor: Option<Arc<RedisMonitor>>,
    plugin_manager: PluginManager,
) -> Router {
    let static_dir = crate::core::app_paths::resolve_runtime_path("static");
    let static_index_path = static_dir.join("index.html");

    let state = AppState {
        pool,
        db_service,
        redis_monitor,
        static_index_path,
        plugin_manager,
    };

    // 公开路由（无需认证）
    let public_routes = Router::new()
        // 健康检查
        .route("/health", get(health_check))
        // 数据库连接状态检查
        .route("/api/db/status", get(db_status_check))
        // 调试路由
        .route("/api/debug/persons", get(debug::debug_persons))
        // 认证路由
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        // 特殊用户登录路由（公开）
        .route("/api/special-users/login", post(special_user::special_user_login))
        // 公开路由
        .route("/api/persons", get(person::list))
        .route("/api/persons/:id", get(person::get))
        .route("/api/departments", get(department::list))
        .route("/api/departments/:id", get(department::get))
        .route("/api/attendances", get(attendance::list))
        .route("/api/scores", get(score::list))
        .route("/api/notices", get(notice::list))
        .route(
            "/api/permission/teacher/classes",
            get(person::get_teacher_classes),
        )
        // WebSocket路由
        .route("/ws", get(crate::ws::handler::ws_handler));

    // 需要认证的路由
    let protected_routes = Router::new()
        .route("/api/persons", post(person::create))
        .route("/api/persons/:id", put(person::update))
        .route("/api/persons/:id", delete(person::delete))
        .route("/api/classes", get(class::list))
        .route("/api/classes", post(class::create))
        .route("/api/classes/:id", get(class::get))
        .route("/api/classes/:id", put(class::update))
        .route("/api/classes/:id", delete(class::delete))
        .route("/api/classes/:id/students", get(class::get_class_students))
        .route("/api/classes/:id/teachers", get(class::get_class_teachers))
        .route("/api/departments", post(department::create))
        .route("/api/departments/:id", put(department::update))
        .route("/api/departments/:id", delete(department::delete))
        .route("/api/attendances", post(attendance::create))
        .route("/api/attendances/:id", get(attendance::get))
        .route("/api/attendances/:id", put(attendance::update))
        .route("/api/attendances/:id", delete(attendance::delete))
        .route("/api/scores", post(score::create))
        .route("/api/scores/:id", get(score::get))
        .route("/api/scores/:id", put(score::update))
        .route("/api/scores/:id", delete(score::delete))
        .route("/api/notices", post(notice::create))
        .route("/api/notices/:id", get(notice::get))
        .route("/api/notices/:id", put(notice::update))
        .route("/api/notices/:id", delete(notice::delete))
        // 权限管理路由
        .route("/api/permissions", get(permission::list_role_permissions))
        .route("/api/permissions", post(permission::add_role_permission))
        .route(
            "/api/permissions",
            delete(permission::remove_role_permission),
        )
        .route("/api/permissions/check", post(permission::check_permission))
        .route(
            "/api/permissions/users/:user_id",
            get(permission::list_user_permissions),
        )
        .route(
            "/api/permissions/users/:user_id",
            post(permission::add_user_permission),
        )
        .route(
            "/api/permissions/users/:user_id",
            delete(permission::remove_user_permission),
        )
        // 新增权限管理路由
        .route(
            "/api/permissions/translations",
            post(permission::get_permission_translations),
        )
        .route(
            "/api/permissions/keys",
            get(permission::get_all_permission_keys),
        )
        .route(
            "/api/permissions/apply-yaml",
            post(permission::apply_yaml_template),
        )
        // 小组管理路由（需要认证）
        .route("/api/groups", get(group::list_all))
        .route("/api/groups", post(group::create))
        .route("/api/groups/class/:class_id", get(group::list))
        .route("/api/groups/:id", get(group::get))
        .route("/api/groups/:id", put(group::update))
        .route("/api/groups/:id", delete(group::delete))
        .route("/api/groups/:id/members", get(group::get_members))
        .route("/api/groups/:id/members", post(group::add_member))
        .route(
            "/api/groups/:id/members/:person_id",
            delete(group::remove_member),
        )
        .route(
            "/api/groups/:id/score-records",
            get(group::get_score_records),
        )
        .route("/api/groups/:id/score", post(group::update_score))
        .route("/api/chat/conversations", get(chat::list_conversations))
        .route(
            "/api/chat/conversations/:id/messages",
            get(chat::list_messages),
        )
        .route(
            "/api/chat/conversations/:id/messages",
            post(chat::send_message),
        )
        .route("/api/chat/conversations/:id/read", post(chat::mark_read))
        // AI 相关路由
        .route("/api/ai/chat", post(ai::chat))
        .route("/api/ai/identities", get(ai::list_identities))
        .route("/api/ai/identities", post(ai::create_identity))
        .route("/api/ai/identities/:id", put(ai::update_identity))
        .route("/api/ai/identities/:id", delete(ai::delete_identity))
        .route("/api/ai/settings", get(ai::get_settings))
        .route("/api/ai/settings", put(ai::update_settings))
        .route("/api/ai/context-data", get(ai::get_context_data))
        .route("/api/ai/query", post(ai_data::query_data))
        .route("/api/ai/enhanced-chat", post(ai_enhanced::enhanced_chat))
        .route(
            "/api/ai/context",
            post(ai_context_provider::get_page_context),
        )
        .route(
            "/api/ai/assistant/suggestion",
            post(ai_assistant::get_assistant_suggestion),
        )
        .route("/api/ai/actions", post(ai_actions::execute_action))
        .route(
            "/api/ai/actions/available",
            get(ai_actions::get_available_actions),
        )
        // 监控路由（需要管理员权限）
        .route("/api/monitor/status", get(monitor::get_monitor_status))
        .route("/api/monitor/buffer", get(monitor::get_buffer_status))
        .route("/api/monitor/buffer/flush", post(monitor::flush_buffer))
        // 特殊用户路由
        .route("/api/special-users", get(special_user::list_special_users))
        .route("/api/special-users", post(special_user::create_special_user))
        .route("/api/special-users/:id", put(special_user::update_special_user))
        .route("/api/special-users/:id", delete(special_user::delete_special_user))
        .route("/api/special-users/:id/link", post(special_user::link_person_to_special_user))
        // 操作日志路由
        .route("/api/operation-logs", get(special_user::list_operation_logs))
        .layer(middleware::from_fn(auth_middleware));

    // 合并路由
    public_routes
        .merge(protected_routes)
        .nest_service("/assets", ServeDir::new(static_dir.join("assets")))
        .fallback(spa_fallback)
        // 注入状态
        .with_state(state)
}

async fn spa_fallback(State(state): State<AppState>, uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    if path.starts_with("/api") || path.starts_with("/ws") {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    match tokio::fs::read_to_string(&state.static_index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "前端页面未构建，请先生成 static/index.html",
        )
            .into_response(),
    }
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
    let pg_status = match &state.pool {
        Some(pool) => match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => ("connected", "PostgreSQL connection is active".to_string()),
            Err(e) => ("error", format!("PostgreSQL query failed: {}", e)),
        },
        None => (
            "not_initialized",
            "PostgreSQL pool not initialized".to_string(),
        ),
    };

    let redis_status = match &state.db_service {
        Some(db_service) => {
            let status = db_service.status().await;
            if status.redis_connected {
                ("connected", "Redis connection is active".to_string())
            } else {
                ("disconnected", "Redis is not connected".to_string())
            }
        }
        None => (
            "not_initialized",
            "Database service not initialized".to_string(),
        ),
    };

    let overall_status = if pg_status.0 == "connected" {
        "ok"
    } else {
        "error"
    };

    Json(DbStatusResponse {
        status: overall_status.to_string(),
        message: format!("PostgreSQL: {}, Redis: {}", pg_status.1, redis_status.1),
        details: Some(format!(
            "PostgreSQL status: {}, Redis status: {}",
            pg_status.0, redis_status.0
        )),
    })
}
