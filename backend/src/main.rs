use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

mod api;
mod core;
mod models;
mod plugins;
mod utils;
mod ws;

/// 自定义日志格式化器
/// 格式: [time][level][target] message
struct CustomFormatter;

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for CustomFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        use tracing::Level;

        // 格式化时间
        let time_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 获取日志级别
        let level = match *event.metadata().level() {
            Level::TRACE => "TRACE",
            Level::DEBUG => "DEBUG",
            Level::INFO => "INFO",
            Level::WARN => "WARNING",
            Level::ERROR => "ERROR",
        };

        // 获取 target
        let target = event.metadata().target();

        // 写入格式: [time][level][target]
        write!(writer, "[{}][{}][{}] ", time_str, level, target)?;

        // 写入消息
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

#[tokio::main]
async fn main() {
    // 初始化日志（如果没有设置 RUST_LOG，默认使用 info 级别）
    // 格式: [time][level][target] message
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .event_format(CustomFormatter)
        .init();

    tracing::info!("Starting school-management-backend...");

    // 加载配置
    let config = match core::config::load_config() {
        Ok(config) => {
            tracing::info!("Configuration loaded successfully");
            Arc::new(config)
        }
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // 初始化 PostgreSQL 数据库连接
    tracing::info!("Connecting to PostgreSQL database...");
    let pool = match core::db::init_db(&config.database_url).await {
        Ok(pool) => {
            tracing::info!("PostgreSQL database connected successfully");
            Some(pool)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to initialize PostgreSQL database: {}, starting server with limited functionality",
                e
            );
            None
        }
    };

    // 初始化 Redis 服务（传入 pg_pool 以便 WriteBuffer 可以写入 PostgreSQL）
    let redis_service = core::redis::init_redis(&config, pool.clone()).await;

    // 初始化全局操作日志记录器
    let redis_client = redis_service.as_ref().map(|s| Arc::new(s.client.clone()));
    core::operation_logger::init_global_logger(redis_client);
    tracing::info!("Global operation logger initialized");

    // 创建统一数据库服务层（复用 RedisService 中的组件）
    let db_service = match core::database_service::DatabaseService::new(
        pool.clone(),
        redis_service.as_ref().map(|s| s.client.clone()),
        redis_service.as_ref().map(|s| s.buffer.clone()),
        config.clone(),
    )
    .await
    {
        Ok(service) => {
            tracing::info!("Database service initialized successfully");
            Some(service)
        }
        Err(e) => {
            tracing::error!("Failed to initialize database service: {}", e);
            None
        }
    };

    // 启动 Redis 缓冲写入调度器
    if let Some(ref redis) = redis_service {
        let buffer = redis.buffer.clone();
        buffer.start_flush_scheduler();
        tracing::info!("Redis buffer flush scheduler started");
    }

    // 启动 Redis 监控
    if let Some(ref redis) = redis_service {
        let monitor = redis.monitor.clone();

        // 立即执行一次检查以获取初始数据
        match monitor.immediate_check().await {
            Ok(metrics) => {
                if metrics.connected {
                    tracing::info!(
                        "Redis initial check: memory={:.1}%, clients={}, hit_rate={:.1}%",
                        metrics.memory_usage_percent,
                        metrics.connected_clients,
                        metrics.hit_rate
                    );
                } else {
                    tracing::warn!("Redis initial check: not connected");
                }
            }
            Err(e) => {
                tracing::warn!("Redis initial check failed: {}", e);
            }
        }

        monitor.start_monitoring();
        tracing::info!("Redis monitoring started");
    }

    // 初始化插件管理器
    let plugin_manager = core::plugin::PluginManager::new();

    // 获取 Redis 监控器
    let redis_monitor = redis_service.as_ref().map(|s| s.monitor.clone());

    // 构建路由
    let app = api::routes::create_router(pool, db_service, redis_monitor, plugin_manager);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Server starting on http://{}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("Server successfully bound to port {}", config.server_port);
            listener
        }
        Err(e) => {
            tracing::error!("Failed to bind to port {}: {}", config.server_port, e);
            std::process::exit(1);
        }
    };

    tracing::info!("Server is now running and accepting connections");

    if let Err(e) = axum::serve(listener, app.into_make_service()).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
