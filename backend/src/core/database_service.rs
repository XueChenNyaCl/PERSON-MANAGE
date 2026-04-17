use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, trace, warn};

use super::config::Config;
use super::error::AppError;
use super::redis::{buffer::WriteBuffer, cache::CacheManager, client::RedisClient};

#[derive(Clone)]
pub struct DatabaseService {
    pg_pool: Option<PgPool>,
    cache_manager: Option<CacheManager>,
    write_buffer: Option<Arc<WriteBuffer>>,
    rate_limiter: Arc<RateLimiter>,
    config: Arc<Config>,
}

#[allow(dead_code)]
pub struct ServiceStatus {
    pub pg_connected: bool,
    pub redis_connected: bool,
    pub buffer_size: usize,
    pub rate_limited: bool,
}

struct RateLimiter {
    qps: u32,
    tokens: Mutex<u32>,
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    fn new(qps: u32) -> Self {
        Self {
            qps,
            tokens: Mutex::new(qps),
            last_refill: Mutex::new(Instant::now()),
        }
    }

    async fn acquire(&self) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);
        let refill_amount = (elapsed.as_secs_f64() * self.qps as f64) as u32;

        if refill_amount > 0 {
            *tokens = (*tokens + refill_amount).min(self.qps);
            *last_refill = now;
        }

        if *tokens > 0 {
            *tokens -= 1;
            true
        } else {
            false
        }
    }

    async fn is_rate_limited(&self) -> bool {
        !self.acquire().await
    }
}

impl DatabaseService {
    pub async fn new(
        pg_pool: Option<PgPool>,
        redis_client: Option<RedisClient>,
        write_buffer: Option<Arc<WriteBuffer>>,
        config: Arc<Config>,
    ) -> anyhow::Result<Self> {
        let cache_manager = redis_client
            .as_ref()
            .map(|client| CacheManager::new(client.clone(), config.redis.cache.clone()));

        let rate_limiter = Arc::new(RateLimiter::new(config.redis.rate_limit.qps));

        Ok(Self {
            pg_pool,
            cache_manager,
            write_buffer,
            rate_limiter,
            config,
        })
    }

    #[allow(dead_code)]
    pub async fn get<T>(&self, cache_key: &str, query_sql: &str) -> Result<Option<T>, AppError>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + Serialize + DeserializeOwned,
    {
        if let Some(cache) = &self.cache_manager {
            if cache.is_available().await {
                match cache.get::<T>(cache_key).await {
                    Ok(Some(data)) => {
                        trace!("Cache hit for key: {}", cache_key);
                        return Ok(Some(data));
                    }
                    Ok(None) => {
                        trace!("Cache miss for key: {}", cache_key);
                    }
                    Err(e) => {
                        warn!("Cache read error: {}, falling back to database", e);
                    }
                }
            }
        }

        if (self.cache_manager.is_none()
            || !self.cache_manager.as_ref().unwrap().is_available().await)
            && self.rate_limiter.is_rate_limited().await
        {
            return Err(AppError::InternalWithMessage(
                "Service temporarily unavailable due to high load, please try again later"
                    .to_string(),
            ));
        }

        let pool = self.pg_pool.as_ref().ok_or(AppError::Internal)?;

        let row: Option<T> = sqlx::query_as(query_sql)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?;

        if let Some(ref data) = row {
            if let Some(cache) = &self.cache_manager {
                let ttl = self.get_ttl_from_key(cache_key);
                if let Err(e) = cache.set(cache_key, data, Some(ttl)).await {
                    warn!("Failed to set cache: {}", e);
                }
            }
        }

        Ok(row)
    }

    #[allow(dead_code)]
    pub async fn list<T>(&self, cache_key: &str, query_sql: &str) -> Result<Vec<T>, AppError>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + Serialize + DeserializeOwned,
    {
        if let Some(cache) = &self.cache_manager {
            if cache.is_available().await {
                match cache.get::<Vec<T>>(cache_key).await {
                    Ok(Some(data)) => {
                        trace!("Cache hit for list key: {}", cache_key);
                        return Ok(data);
                    }
                    Ok(None) => {
                        trace!("Cache miss for list key: {}", cache_key);
                    }
                    Err(e) => {
                        warn!("Cache read error: {}, falling back to database", e);
                    }
                }
            }
        }

        if (self.cache_manager.is_none()
            || !self.cache_manager.as_ref().unwrap().is_available().await)
            && self.rate_limiter.is_rate_limited().await
        {
            return Err(AppError::InternalWithMessage(
                "Service temporarily unavailable due to high load, please try again later"
                    .to_string(),
            ));
        }

        let pool = self.pg_pool.as_ref().ok_or(AppError::Internal)?;

        let rows: Vec<T> = sqlx::query_as(query_sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        if let Some(cache) = &self.cache_manager {
            let ttl = self.config.redis.cache.list_ttl_secs as usize;
            if let Err(e) = cache.set(cache_key, &rows, Some(ttl)).await {
                warn!("Failed to set list cache: {}", e);
            }
        }

        Ok(rows)
    }

    #[allow(dead_code)]
    pub async fn insert(&self, table: &str, data: Value) -> Result<(), AppError> {
        if let Some(buffer) = &self.write_buffer {
            buffer
                .buffer_insert(table, data)
                .await
                .map_err(AppError::from)?;

            self.invalidate_entity_cache(table, None).await?;

            debug!("Insert buffered for table: {}", table);
            Ok(())
        } else {
            self.insert_immediate(table, data).await
        }
    }

    #[allow(dead_code)]
    pub async fn update(&self, table: &str, id: &str, data: Value) -> Result<(), AppError> {
        if let Some(buffer) = &self.write_buffer {
            buffer
                .buffer_update(table, id, data)
                .await
                .map_err(AppError::from)?;

            self.invalidate_entity_cache(table, Some(id)).await?;

            debug!("Update buffered for table: {} id: {}", table, id);
            Ok(())
        } else {
            self.update_immediate(table, id, data).await
        }
    }

    #[allow(dead_code)]
    pub async fn delete(&self, table: &str, id: &str) -> Result<(), AppError> {
        if let Some(buffer) = &self.write_buffer {
            buffer
                .buffer_delete(table, id)
                .await
                .map_err(AppError::from)?;

            self.invalidate_entity_cache(table, Some(id)).await?;

            debug!("Delete buffered for table: {} id: {}", table, id);
            Ok(())
        } else {
            self.delete_immediate(table, id).await
        }
    }

    #[allow(dead_code)]
    pub async fn insert_immediate(&self, table: &str, data: Value) -> Result<(), AppError> {
        let pool = self.pg_pool.as_ref().ok_or(AppError::Internal)?;

        let obj = data
            .as_object()
            .ok_or_else(|| AppError::InvalidInput("Data must be an object".to_string()))?;

        let columns: Vec<String> = obj.keys().cloned().collect();
        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", ")
        );

        let mut sql_query = sqlx::query(&query);
        for value in obj.values() {
            sql_query = sql_query.bind(value.to_string());
        }

        sql_query.execute(pool).await.map_err(AppError::Database)?;

        self.invalidate_entity_cache(table, None).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn update_immediate(
        &self,
        table: &str,
        id: &str,
        data: Value,
    ) -> Result<(), AppError> {
        let pool = self.pg_pool.as_ref().ok_or(AppError::Internal)?;

        let obj = data
            .as_object()
            .ok_or_else(|| AppError::InvalidInput("Data must be an object".to_string()))?;

        let sets: Vec<String> = obj
            .keys()
            .enumerate()
            .map(|(i, col)| format!("{} = ${}", col, i + 1))
            .collect();

        let query = format!(
            "UPDATE {} SET {} WHERE id = ${}",
            table,
            sets.join(", "),
            sets.len() + 1
        );

        let mut sql_query = sqlx::query(&query);
        for value in obj.values() {
            sql_query = sql_query.bind(value.to_string());
        }
        sql_query = sql_query.bind(id);

        sql_query.execute(pool).await.map_err(AppError::Database)?;

        self.invalidate_entity_cache(table, Some(id)).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete_immediate(&self, table: &str, id: &str) -> Result<(), AppError> {
        let pool = self.pg_pool.as_ref().ok_or(AppError::Internal)?;

        let query = format!("DELETE FROM {} WHERE id = $1", table);
        sqlx::query(&query)
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        self.invalidate_entity_cache(table, Some(id)).await?;

        Ok(())
    }

    pub async fn invalidate_entity_cache(
        &self,
        table: &str,
        id: Option<&str>,
    ) -> Result<(), AppError> {
        if let Some(cache) = &self.cache_manager {
            let entity_type = table.trim_end_matches('s');

            if let Err(e) = cache.invalidate_entity(entity_type, id).await {
                warn!("Failed to invalidate entity cache: {}", e);
            }

            if let Err(e) = cache.invalidate_list(entity_type).await {
                warn!("Failed to invalidate list cache: {}", e);
            }
        }

        Ok(())
    }

    /// 从缓存查询数据（仅缓存，不查询数据库）
    pub async fn query_cached<T: DeserializeOwned>(
        &self,
        cache_key: &str,
    ) -> Result<Option<T>, AppError> {
        if let Some(cache) = &self.cache_manager {
            if cache.is_available().await {
                match cache.get::<T>(cache_key).await {
                    Ok(Some(data)) => {
                        trace!("Cache hit for key: {}", cache_key);
                        return Ok(Some(data));
                    }
                    Ok(None) => {
                        trace!("Cache miss for key: {}", cache_key);
                    }
                    Err(e) => {
                        warn!("Cache read error for key {}: {}", cache_key, e);
                    }
                }
            }
        }
        Ok(None)
    }

    /// 设置缓存数据
    pub async fn cache_set<T: Serialize>(
        &self,
        cache_key: &str,
        value: &T,
        ttl_secs: Option<usize>,
    ) -> Result<(), AppError> {
        if let Some(cache) = &self.cache_manager {
            let ttl = ttl_secs.unwrap_or_else(|| self.config.redis.cache.default_ttl_secs as usize);
            if let Err(e) = cache.set(cache_key, value, Some(ttl)).await {
                warn!("Failed to set cache for key {}: {}", cache_key, e);
                return Err(AppError::InternalWithMessage(format!(
                    "Cache set failed: {}",
                    e
                )));
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn query_raw<T>(&self, query_sql: &str) -> Result<Vec<T>, AppError>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let pool = self.pg_pool.as_ref().ok_or(AppError::Internal)?;

        sqlx::query_as(query_sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }

    #[allow(dead_code)]
    pub async fn execute_raw(&self, query_sql: &str) -> Result<u64, AppError> {
        let pool = self.pg_pool.as_ref().ok_or(AppError::Internal)?;

        let result = sqlx::query(query_sql)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    pub async fn status(&self) -> ServiceStatus {
        let pg_connected = self.pg_pool.is_some();

        let redis_connected = if let Some(cache) = &self.cache_manager {
            cache.is_available().await
        } else {
            false
        };

        let buffer_size = if let Some(buffer) = &self.write_buffer {
            buffer.get_queue_length().await.unwrap_or(0)
        } else {
            0
        };

        let rate_limited = self.rate_limiter.is_rate_limited().await;

        ServiceStatus {
            pg_connected,
            redis_connected,
            buffer_size,
            rate_limited,
        }
    }

    pub async fn flush_buffer(&self) -> Result<(), AppError> {
        if let Some(buffer) = &self.write_buffer {
            buffer.force_flush().await.map_err(AppError::from)?;
        }
        Ok(())
    }

    pub fn get_write_buffer(&self) -> Option<Arc<WriteBuffer>> {
        self.write_buffer.clone()
    }

    #[allow(dead_code)]
    fn get_ttl_from_key(&self, key: &str) -> usize {
        if key.contains(":list") {
            self.config.redis.cache.list_ttl_secs as usize
        } else if key.contains(":person") {
            self.config.redis.cache.person_ttl_secs as usize
        } else if key.contains(":class") {
            self.config.redis.cache.class_ttl_secs as usize
        } else if key.contains(":score") {
            self.config.redis.cache.score_ttl_secs as usize
        } else {
            self.config.redis.cache.default_ttl_secs as usize
        }
    }
}

#[allow(dead_code)]
pub async fn wait_for_rate_limiter(qps: u32) {
    let interval = Duration::from_secs_f64(1.0 / qps as f64);
    sleep(interval).await;
}
