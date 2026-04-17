use std::sync::Arc;
use std::time::Duration;

use redis::{aio::MultiplexedConnection, Client, RedisResult};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use super::error::{RedisError, Result};
use crate::core::config::RedisConfig;

#[derive(Clone)]
pub struct RedisClient {
    client: Client,
    connection: Arc<RwLock<Option<MultiplexedConnection>>>,
    config: RedisConfig,
}

impl RedisClient {
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        let client = Client::open(config.url.as_str()).map_err(|e| {
            RedisError::ConnectionError(format!("Failed to create Redis client: {}", e))
        })?;

        let conn = Self {
            client,
            connection: Arc::new(RwLock::new(None)),
            config: config.clone(),
        };

        // 尝试连接，但失败时不阻止创建客户端
        // 连接将在后续操作时重试
        info!("Redis client created (connection will be established on first use)");

        Ok(conn)
    }

    #[allow(dead_code)]
    pub async fn connect(&mut self) -> Result<()> {
        let mut conn = timeout(
            Duration::from_secs(self.config.timeout_secs),
            self.client.get_multiplexed_tokio_connection(),
        )
        .await
        .map_err(|_| RedisError::TimeoutError)?
        .map_err(RedisError::from)?;

        // 如果配置了密码，进行认证
        if let Some(ref password) = self.config.password {
            let auth_result: RedisResult<String> = redis::cmd("AUTH")
                .arg(password)
                .query_async(&mut conn)
                .await;

            match auth_result {
                Ok(_) => info!("Redis authentication successful"),
                Err(e) => {
                    warn!("Redis authentication failed: {}", e);
                    return Err(RedisError::from(e));
                }
            }
        }

        let mut connection = self.connection.write().await;
        *connection = Some(conn);
        info!("Connected to Redis successfully");
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        // 先检查是否有连接，不持有锁
        let has_connection = {
            let connection = self.connection.read().await;
            connection.is_some()
        };

        if !has_connection {
            return false;
        }

        // 执行健康检查
        self.health_check().await.is_ok()
    }

    pub async fn health_check(&self) -> Result<()> {
        let mut connection = self.connection.write().await;

        if connection.is_none() {
            return Err(RedisError::NotAvailable);
        }

        let conn = connection.as_mut().unwrap();

        let result: RedisResult<String> = redis::cmd("PING").query_async(conn).await;

        match result {
            Ok(pong) if pong == "PONG" => {
                debug!("Redis health check passed");
                Ok(())
            }
            Ok(other) => {
                warn!("Redis health check returned unexpected response: {}", other);
                Err(RedisError::ConnectionError(
                    "Unexpected PING response".to_string(),
                ))
            }
            Err(e) => {
                error!("Redis health check failed: {}", e);
                *connection = None;
                Err(RedisError::from(e))
            }
        }
    }

    pub async fn get_connection(&self) -> Result<MultiplexedConnection> {
        // 先检查是否有现有连接
        {
            let connection = self.connection.read().await;
            if let Some(conn) = connection.as_ref() {
                return Ok(conn.clone());
            }
        }

        // 如果没有连接，尝试建立连接
        warn!("No Redis connection available, attempting to connect...");
        self.try_reconnect().await?;

        // 再次获取连接
        let connection = self.connection.read().await;
        if let Some(conn) = connection.as_ref() {
            return Ok(conn.clone());
        }

        Err(RedisError::NotAvailable)
    }

    pub async fn try_reconnect(&self) -> Result<()> {
        warn!("Attempting to connect to Redis...");

        let mut new_conn = timeout(
            Duration::from_secs(self.config.timeout_secs),
            self.client.get_multiplexed_tokio_connection(),
        )
        .await
        .map_err(|_| RedisError::TimeoutError)?
        .map_err(RedisError::from)?;

        // 如果配置了密码，进行认证
        if let Some(ref password) = self.config.password {
            let auth_result: RedisResult<String> = redis::cmd("AUTH")
                .arg(password)
                .query_async(&mut new_conn)
                .await;

            match auth_result {
                Ok(_) => debug!("Redis authentication successful"),
                Err(e) => {
                    warn!("Redis authentication failed: {}", e);
                    return Err(RedisError::from(e));
                }
            }
        }

        let mut connection = self.connection.write().await;
        *connection = Some(new_conn);

        info!("Connected to Redis successfully");
        Ok(())
    }

    pub async fn execute_with_retry<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn(
            MultiplexedConnection,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = RedisResult<T>> + Send>>,
    {
        // 获取连接，如果没有则尝试连接
        let conn = match self.get_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!("Redis not available: {}", e);
                return Err(e);
            }
        };

        match operation(conn).await {
            Ok(result) => Ok(result),
            Err(e) => {
                if e.is_io_error() || e.is_timeout() {
                    warn!("Redis operation failed, connection may be lost: {}", e);
                    // 清除连接，下次会重新连接
                    let mut connection = self.connection.write().await;
                    *connection = None;
                }
                Err(RedisError::from(e))
            }
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("GET")
                    .arg(&key)
                    .query_async::<_, Option<String>>(&mut conn)
                    .await
            })
        })
        .await
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: Option<usize>) -> Result<()> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let value = value.to_string();
            Box::pin(async move {
                if let Some(ttl) = ttl_secs {
                    redis::cmd("SETEX")
                        .arg(&key)
                        .arg(ttl)
                        .arg(&value)
                        .query_async::<_, ()>(&mut conn)
                        .await
                } else {
                    redis::cmd("SET")
                        .arg(&key)
                        .arg(&value)
                        .query_async::<_, ()>(&mut conn)
                        .await
                }
            })
        })
        .await
    }

    pub async fn delete(&self, key: &str) -> Result<bool> {
        let result: u32 = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                Box::pin(async move {
                    redis::cmd("DEL")
                        .arg(&key)
                        .query_async::<_, u32>(&mut conn)
                        .await
                })
            })
            .await?;

        Ok(result > 0)
    }

    #[allow(dead_code)]
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let result: bool = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                Box::pin(async move {
                    redis::cmd("EXISTS")
                        .arg(&key)
                        .query_async::<_, bool>(&mut conn)
                        .await
                })
            })
            .await?;

        Ok(result)
    }

    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        self.execute_with_retry(|mut conn| {
            let pattern = pattern.to_string();
            Box::pin(async move {
                redis::cmd("KEYS")
                    .arg(&pattern)
                    .query_async::<_, Vec<String>>(&mut conn)
                    .await
            })
        })
        .await
    }

    pub async fn lpush(&self, key: &str, value: &str) -> Result<usize> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let value = value.to_string();
            Box::pin(async move {
                redis::cmd("LPUSH")
                    .arg(&key)
                    .arg(&value)
                    .query_async::<_, usize>(&mut conn)
                    .await
            })
        })
        .await
    }

    pub async fn rpop(&self, key: &str) -> Result<Option<String>> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("RPOP")
                    .arg(&key)
                    .query_async::<_, Option<String>>(&mut conn)
                    .await
            })
        })
        .await
    }

    pub async fn llen(&self, key: &str) -> Result<usize> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("LLEN")
                    .arg(&key)
                    .query_async::<_, usize>(&mut conn)
                    .await
            })
        })
        .await
    }

    pub async fn info(&self, section: &str) -> Result<String> {
        self.execute_with_retry(|mut conn| {
            let section = section.to_string();
            Box::pin(async move {
                redis::cmd("INFO")
                    .arg(&section)
                    .query_async::<_, String>(&mut conn)
                    .await
            })
        })
        .await
    }

    // ========== List 操作 ==========

    #[allow(dead_code)]
    pub async fn rpush(&self, key: &str, value: &str) -> Result<usize> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let value = value.to_string();
            Box::pin(async move {
                redis::cmd("RPUSH")
                    .arg(&key)
                    .arg(&value)
                    .query_async::<_, usize>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn lrange(&self, key: &str, start: isize, end: isize) -> Result<Vec<String>> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("LRANGE")
                    .arg(&key)
                    .arg(start)
                    .arg(end)
                    .query_async::<_, Vec<String>>(&mut conn)
                    .await
            })
        })
        .await
    }

    // ========== Hash 操作 ==========

    #[allow(dead_code)]
    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<bool> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let field = field.to_string();
            let value = value.to_string();
            Box::pin(async move {
                redis::cmd("HSET")
                    .arg(&key)
                    .arg(&field)
                    .arg(&value)
                    .query_async::<_, bool>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<String>> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let field = field.to_string();
            Box::pin(async move {
                redis::cmd("HGET")
                    .arg(&key)
                    .arg(&field)
                    .query_async::<_, Option<String>>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn hdel(&self, key: &str, field: &str) -> Result<bool> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let field = field.to_string();
            Box::pin(async move {
                redis::cmd("HDEL")
                    .arg(&key)
                    .arg(&field)
                    .query_async::<_, bool>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn hexists(&self, key: &str, field: &str) -> Result<bool> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let field = field.to_string();
            Box::pin(async move {
                redis::cmd("HEXISTS")
                    .arg(&key)
                    .arg(&field)
                    .query_async::<_, bool>(&mut conn)
                    .await
            })
        })
        .await
    }

    // ========== Sorted Set 操作 ==========

    #[allow(dead_code)]
    pub async fn zadd(&self, key: &str, member: &str, score: i64) -> Result<bool> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let member = member.to_string();
            Box::pin(async move {
                redis::cmd("ZADD")
                    .arg(&key)
                    .arg(score)
                    .arg(&member)
                    .query_async::<_, bool>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn zrem(&self, key: &str, member: &str) -> Result<bool> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            let member = member.to_string();
            Box::pin(async move {
                redis::cmd("ZREM")
                    .arg(&key)
                    .arg(&member)
                    .query_async::<_, bool>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn zrange(&self, key: &str, start: isize, end: isize) -> Result<Vec<String>> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("ZRANGE")
                    .arg(&key)
                    .arg(start)
                    .arg(end)
                    .query_async::<_, Vec<String>>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn zrevrange(&self, key: &str, start: isize, end: isize) -> Result<Vec<String>> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("ZREVRANGE")
                    .arg(&key)
                    .arg(start)
                    .arg(end)
                    .query_async::<_, Vec<String>>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn zcard(&self, key: &str) -> Result<usize> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("ZCARD")
                    .arg(&key)
                    .query_async::<_, usize>(&mut conn)
                    .await
            })
        })
        .await
    }

    // ========== 过期时间操作 ==========

    #[allow(dead_code)]
    pub async fn expire(&self, key: &str, seconds: usize) -> Result<bool> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("EXPIRE")
                    .arg(&key)
                    .arg(seconds)
                    .query_async::<_, bool>(&mut conn)
                    .await
            })
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        self.execute_with_retry(|mut conn| {
            let key = key.to_string();
            Box::pin(async move {
                redis::cmd("TTL")
                    .arg(&key)
                    .query_async::<_, i64>(&mut conn)
                    .await
            })
        })
        .await
    }
}
