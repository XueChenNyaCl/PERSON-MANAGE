use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

mod api;
mod core;
mod models;
mod plugins;
mod utils;
mod ws;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // 加载配置
    let config = core::config::load_config().expect("Failed to load config");

    // 初始化数据库连接
    let pool = match core::db::init_db(&config.database_url).await {
        Ok(pool) => {
            tracing::info!("Database connected successfully");
            Some(pool)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to initialize database: {}, starting server with limited functionality",
                e
            );
            // 数据库连接失败，返回 None，服务器仍然可以启动
            None
        }
    };

    // 初始化插件管理器
    let plugin_manager = core::plugin::PluginManager::new();

    // 构建路由
    let app = api::routes::create_router(pool, plugin_manager);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Failed to start server");
}
