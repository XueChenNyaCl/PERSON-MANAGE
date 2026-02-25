use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::warn;

pub async fn init_db(database_url: &str) -> Result<PgPool, anyhow::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    // 执行数据库迁移，如果失败只记录警告不阻止启动
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => tracing::info!("Database migrations applied successfully"),
        Err(e) => warn!("Database migrations failed: {}, continuing with existing schema", e),
    }

    Ok(pool)
}
