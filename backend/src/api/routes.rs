use axum::{extract::State, Json, middleware, routing::delete, routing::get, routing::post, routing::put, Router};
use sqlx::PgPool;

use crate::api::{ai, ai_actions, ai_data, ai_enhanced, attendance, auth, class, department, debug, group, notice, permission, person, score};
use crate::core::middleware::auth_middleware;
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
        // 公开路由
        .route("/api/persons", get(person::list))
        .route("/api/persons/:id", get(person::get))
        .route("/api/classes", get(class::list))
        .route("/api/classes/:id", get(class::get))
        .route("/api/classes/:id/students", get(class::get_class_students))
        .route("/api/classes/:id/teachers", get(class::get_class_teachers))
        .route("/api/departments", get(department::list))
        .route("/api/departments/:id", get(department::get))
        .route("/api/attendances", get(attendance::list))
        .route("/api/scores", get(score::list))
        .route("/api/notices", get(notice::list))
        .route("/api/permission/teacher/classes", get(person::get_teacher_classes))
        // 小组管理路由（公开查看）
        .route("/api/groups", get(group::list_all))
        .route("/api/groups/class/:class_id", get(group::list))
        .route("/api/groups/:id", get(group::get))
        .route("/api/groups/:id/members", get(group::get_members))
        .route("/api/groups/:id/score-records", get(group::get_score_records))
        // WebSocket路由
        .route("/ws", get(crate::ws::handler::ws_handler));

    // 需要认证的路由
    let protected_routes = Router::new()
        .route("/api/persons", post(person::create))
        .route("/api/persons/:id", put(person::update))
        .route("/api/persons/:id", delete(person::delete))
        .route("/api/classes", post(class::create))
        .route("/api/classes/:id", put(class::update))
        .route("/api/classes/:id", delete(class::delete))
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
        .route("/api/permissions", delete(permission::remove_role_permission))
        .route("/api/permissions/check", post(permission::check_permission))
        .route("/api/permissions/users/:user_id", get(permission::list_user_permissions))
        .route("/api/permissions/users/:user_id", post(permission::add_user_permission))
        .route("/api/permissions/users/:user_id", delete(permission::remove_user_permission))
        // 新增权限管理路由
        .route("/api/permissions/translations", post(permission::get_permission_translations))
        .route("/api/permissions/keys", get(permission::get_all_permission_keys))
        .route("/api/permissions/apply-yaml", post(permission::apply_yaml_template))
        // 小组管理路由（需要认证）
        .route("/api/groups", post(group::create))
        .route("/api/groups/:id", put(group::update))
        .route("/api/groups/:id", delete(group::delete))
        .route("/api/groups/:id/members", post(group::add_member))
        .route("/api/groups/:id/members/:person_id", delete(group::remove_member))
        .route("/api/groups/:id/score", post(group::update_score))
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
        .route("/api/ai/actions", post(ai_actions::execute_action))
        .route("/api/ai/actions/available", get(ai_actions::get_available_actions))
        .layer(middleware::from_fn(auth_middleware));

    // 合并路由
    public_routes
        .merge(protected_routes)
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
