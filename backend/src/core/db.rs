use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};
use tracing::warn;

pub async fn init_db(database_url: &str) -> Result<PgPool, anyhow::Error> {
    // 尝试连接数据库，最多重试3次
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 1..=max_retries {
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
                    "Failed to connect to database (attempt {}/{}): {}. Retrying in 3 seconds...",
                    attempt, max_retries, e
                );
                last_error = Some(e);
                if attempt < max_retries {
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to connect to database after {} attempts: {}",
        max_retries,
        last_error.unwrap_or_else(|| sqlx::Error::RowNotFound)
    ))
}
