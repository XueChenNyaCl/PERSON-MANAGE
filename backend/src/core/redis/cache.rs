use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, trace, warn};

use super::client::RedisClient;
use super::error::Result;
use crate::core::config::CacheConfig;

#[derive(Clone)]
pub struct CacheManager {
    client: RedisClient,
    config: CacheConfig,
}

pub struct CacheKey;

impl CacheKey {
    pub fn person(id: &str) -> String {
        format!("cache:person:{}", id)
    }

    pub fn persons_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:persons:list:{}", hash),
            None => "cache:persons:list".to_string(),
        }
    }

    pub fn class(id: &str) -> String {
        format!("cache:class:{}", id)
    }

    pub fn classes_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:classes:list:{}", hash),
            None => "cache:classes:list".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn score(id: &str) -> String {
        format!("cache:score:{}", id)
    }

    #[allow(dead_code)]
    pub fn scores_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:scores:list:{}", hash),
            None => "cache:scores:list".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn attendance(id: &str) -> String {
        format!("cache:attendance:{}", id)
    }

    #[allow(dead_code)]
    pub fn attendances_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:attendances:list:{}", hash),
            None => "cache:attendances:list".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn department(id: &str) -> String {
        format!("cache:department:{}", id)
    }

    #[allow(dead_code)]
    pub fn departments_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:departments:list:{}", hash),
            None => "cache:departments:list".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn notice(id: &str) -> String {
        format!("cache:notice:{}", id)
    }

    #[allow(dead_code)]
    pub fn notices_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:notices:list:{}", hash),
            None => "cache:notices:list".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn group(id: &str) -> String {
        format!("cache:group:{}", id)
    }

    #[allow(dead_code)]
    pub fn groups_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:groups:list:{}", hash),
            None => "cache:groups:list".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn user(id: &str) -> String {
        format!("cache:user:{}", id)
    }

    #[allow(dead_code)]
    pub fn users_list(query_hash: Option<&str>) -> String {
        match query_hash {
            Some(hash) => format!("cache:users:list:{}", hash),
            None => "cache:users:list".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn entity_pattern(entity: &str) -> String {
        format!("cache:{}:*", entity)
    }

    // 聊天记录相关缓存键
    #[allow(dead_code)]
    pub fn chat_conversation(id: &str) -> String {
        format!("cache:chat:conversation:{}", id)
    }

    #[allow(dead_code)]
    pub fn chat_conversations_list(user_id: &str) -> String {
        format!("cache:chat:conversations:{}", user_id)
    }

    #[allow(dead_code)]
    pub fn chat_messages(conversation_id: &str) -> String {
        format!("cache:chat:messages:{}", conversation_id)
    }

    #[allow(dead_code)]
    pub fn chat_message(id: &str) -> String {
        format!("cache:chat:message:{}", id)
    }

    // AI对话记录相关缓存键 (仅Redis)
    #[allow(dead_code)]
    pub fn ai_conversation(user_id: &str) -> String {
        format!("ai:conversation:{}", user_id)
    }

    #[allow(dead_code)]
    pub fn ai_conversation_messages(user_id: &str, conversation_id: &str) -> String {
        format!("ai:conversation:{}:messages:{}", user_id, conversation_id)
    }

    #[allow(dead_code)]
    pub fn ai_message(id: &str) -> String {
        format!("ai:message:{}", id)
    }

    #[allow(dead_code)]
    pub fn ai_user_conversations(user_id: &str) -> String {
        format!("ai:conversations:{}", user_id)
    }
}

impl CacheManager {
    pub fn new(client: RedisClient, config: CacheConfig) -> Self {
        Self { client, config }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        trace!("Cache GET: {}", key);

        match self.client.get(key).await? {
            Some(value) => {
                let data: T = serde_json::from_str(&value)?;
                trace!("Cache HIT: {}", key);
                Ok(Some(data))
            }
            None => {
                trace!("Cache MISS: {}", key);
                Ok(None)
            }
        }
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<usize>,
    ) -> Result<()> {
        let json_value = serde_json::to_string(value)?;
        let ttl = ttl_secs.or(Some(self.config.default_ttl_secs as usize));

        debug!("Cache SET: {} (TTL: {:?})", key, ttl);
        self.client.set(key, &json_value, ttl).await
    }

    #[allow(dead_code)]
    pub async fn set_with_entity_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        entity_type: &str,
    ) -> Result<()> {
        let ttl = self.get_ttl_for_entity(entity_type);
        self.set(key, value, Some(ttl)).await
    }

    #[allow(dead_code)]
    pub async fn delete(&self, key: &str) -> Result<bool> {
        debug!("Cache DELETE: {}", key);
        self.client.delete(key).await
    }

    #[allow(dead_code)]
    pub async fn exists(&self, key: &str) -> Result<bool> {
        self.client.exists(key).await
    }

    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<usize> {
        debug!("Cache INVALIDATE pattern: {}", pattern);

        let keys = self.client.keys(pattern).await?;
        let count = keys.len();

        for key in keys {
            if let Err(e) = self.client.delete(&key).await {
                warn!("Failed to delete cache key {}: {}", key, e);
            }
        }

        debug!("Invalidated {} keys matching pattern: {}", count, pattern);
        Ok(count)
    }

    pub async fn invalidate_entity(&self, entity_type: &str, id: Option<&str>) -> Result<usize> {
        let pattern = match id {
            Some(entity_id) => format!("cache:{}:{}", entity_type, entity_id),
            None => format!("cache:{}:*", entity_type),
        };
        self.invalidate_pattern(&pattern).await
    }

    pub async fn invalidate_list(&self, entity_type: &str) -> Result<usize> {
        let pattern = format!("cache:{}s:list*", entity_type);
        self.invalidate_pattern(&pattern).await
    }

    #[allow(dead_code)]
    fn get_ttl_for_entity(&self, entity_type: &str) -> usize {
        match entity_type {
            "person" => self.config.person_ttl_secs as usize,
            "class" => self.config.class_ttl_secs as usize,
            "score" => self.config.score_ttl_secs as usize,
            "list" => self.config.list_ttl_secs as usize,
            _ => self.config.default_ttl_secs as usize,
        }
    }

    #[allow(dead_code)]
    pub fn client(&self) -> &RedisClient {
        &self.client
    }

    pub async fn is_available(&self) -> bool {
        self.client.is_connected().await
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
        }
    }
}
