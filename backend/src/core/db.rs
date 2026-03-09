use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};
use tracing::warn;

pub async fn init_db(database_url: &str) -> Result<PgPool, anyhow::Error> {
    loop {
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
        {
            Ok(pool) => {
                // 执行数据库迁移，如果失败只记录警告不阻止启动
                match sqlx::migrate!().run(&pool).await {
                    Ok(_) => tracing::info!("Database migrations applied successfully"),
                    Err(e) => warn!(
                        "Database migrations failed: {}, continuing with existing schema",
                        e
                    ),
                }

                return Ok(pool);
            }
            Err(e) => {
                warn!(
                    "Failed to connect to database: {}. Retrying in 10 seconds...",
                    e
                );
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}
