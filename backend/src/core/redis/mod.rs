pub mod ai_chat_storage;
pub mod buffer;
pub mod cache;
pub mod client;
pub mod error;
pub mod monitor;

use std::sync::Arc;

pub use ai_chat_storage::AIChatStorage;
pub use buffer::WriteBuffer;
pub use cache::CacheManager;
pub use client::RedisClient;
pub use error::RedisError;
pub use monitor::RedisMonitor;

use crate::core::config::Config;
use crate::core::config::RedisConfig;
use sqlx::PgPool;

#[allow(dead_code)]
pub struct RedisService {
    pub client: RedisClient,
    pub cache: CacheManager,
    pub buffer: Arc<WriteBuffer>,
    pub monitor: Arc<RedisMonitor>,
    pub ai_chat_storage: AIChatStorage,
}

impl RedisService {
    pub async fn new(config: &RedisConfig, pg_pool: Option<PgPool>) -> Result<Self, RedisError> {
        let client = RedisClient::new(config).await?;

        let cache = CacheManager::new(client.clone(), config.cache.clone());

        let buffer = Arc::new(WriteBuffer::new(
            client.clone(),
            pg_pool,
            config.buffer.clone(),
        ));

        let monitor = Arc::new(RedisMonitor::new(client.clone(), config.monitor.clone()));

        let ai_chat_storage =
            AIChatStorage::new(client.clone(), config.cache.ai_chat_ttl_secs as usize);

        Ok(Self {
            client,
            cache,
            buffer,
            monitor,
            ai_chat_storage,
        })
    }

    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        self.client.is_connected().await
    }
}

pub async fn init_redis(config: &Config, pg_pool: Option<PgPool>) -> Option<RedisService> {
    match RedisService::new(&config.redis, pg_pool).await {
        Ok(service) => {
            tracing::info!("Redis service initialized successfully");
            Some(service)
        }
        Err(e) => {
            tracing::warn!("Failed to initialize Redis service: {}", e);
            None
        }
    }
}
