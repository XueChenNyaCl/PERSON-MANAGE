use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::cache::CacheKey;
use super::client::RedisClient;
use super::error::{RedisError, Result};

/// AI对话消息结构
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIChatMessage {
    pub id: String,
    pub role: String, // "user" 或 "assistant"
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>, // 可选的元数据（如token使用量等）
}

impl AIChatMessage {
    #[allow(dead_code)]
    pub fn new_user_message(content: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: "user".to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            metadata: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_assistant_message(content: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            metadata: None,
        }
    }
}

/// AI对话会话摘要
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConversationSummary {
    pub id: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}

/// AI对话存储服务
/// 注意：所有数据仅存储在Redis中，不写入PostgreSQL
#[allow(dead_code)]
#[derive(Clone)]
pub struct AIChatStorage {
    client: RedisClient,
    default_ttl_secs: usize,
}

impl AIChatStorage {
    #[allow(dead_code)]
    pub fn new(client: RedisClient, default_ttl_secs: usize) -> Self {
        Self {
            client,
            default_ttl_secs,
        }
    }

    /// 保存AI对话消息（仅Redis）
    /// 会自动将会话添加到用户的会话列表中
    #[allow(dead_code)]
    pub async fn save_message(
        &self,
        user_id: &str,
        conversation_id: &str,
        message: &AIChatMessage,
    ) -> Result<()> {
        let messages_key = CacheKey::ai_conversation_messages(user_id, conversation_id);
        let conversation_key = CacheKey::ai_conversation(user_id);
        let user_conversations_key = CacheKey::ai_user_conversations(user_id);

        // 将消息添加到会话消息列表（使用Redis List，右侧插入）
        let message_json =
            serde_json::to_string(message).map_err(RedisError::SerializationError)?;

        self.client.rpush(&messages_key, &message_json).await?;

        // 设置消息列表的过期时间
        self.client
            .expire(&messages_key, self.default_ttl_secs)
            .await?;

        // 更新会话信息
        let now = Utc::now();
        let conversation_info = serde_json::json!({
            "id": conversation_id,
            "updated_at": now,
        });

        self.client
            .hset(
                &conversation_key,
                conversation_id,
                &conversation_info.to_string(),
            )
            .await?;

        self.client
            .expire(&conversation_key, self.default_ttl_secs)
            .await?;

        // 将会话添加到用户的会话列表（使用Sorted Set，按时间排序）
        let score = now.timestamp();
        self.client
            .zadd(&user_conversations_key, conversation_id, score)
            .await?;
        self.client
            .expire(&user_conversations_key, self.default_ttl_secs)
            .await?;

        debug!(
            "AI chat message saved to Redis: {} (user: {}, conversation: {})",
            message.id, user_id, conversation_id
        );

        Ok(())
    }

    /// 批量保存AI对话消息
    #[allow(dead_code)]
    pub async fn save_messages(
        &self,
        user_id: &str,
        conversation_id: &str,
        messages: &[AIChatMessage],
    ) -> Result<()> {
        for message in messages {
            self.save_message(user_id, conversation_id, message).await?;
        }
        Ok(())
    }

    /// 获取用户的AI对话历史
    /// 支持分页，使用page和page_size参数
    #[allow(dead_code)]
    pub async fn get_conversation_history(
        &self,
        user_id: &str,
        conversation_id: &str,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<AIChatMessage>> {
        let messages_key = CacheKey::ai_conversation_messages(user_id, conversation_id);

        // 检查是否存在
        let exists = self.client.exists(&messages_key).await?;
        if !exists {
            return Ok(Vec::new());
        }

        // 计算范围
        let start = ((page - 1) * page_size) as isize;
        let end = start + page_size as isize - 1;

        // 获取消息列表（使用LRANGE）
        let message_jsons = self.client.lrange(&messages_key, start, end).await?;

        let mut messages = Vec::new();
        for json_str in message_jsons {
            match serde_json::from_str::<AIChatMessage>(&json_str) {
                Ok(msg) => messages.push(msg),
                Err(e) => {
                    warn!("Failed to deserialize AI chat message: {}", e);
                }
            }
        }

        debug!(
            "Retrieved {} AI chat messages from Redis (user: {}, conversation: {})",
            messages.len(),
            user_id,
            conversation_id
        );

        Ok(messages)
    }

    /// 获取用户的所有AI对话会话
    /// 返回按更新时间排序的会话列表
    #[allow(dead_code)]
    pub async fn get_user_conversations(
        &self,
        user_id: &str,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<AIConversationSummary>> {
        let user_conversations_key = CacheKey::ai_user_conversations(user_id);
        let conversation_key = CacheKey::ai_conversation(user_id);

        // 检查是否存在
        let exists = self.client.exists(&user_conversations_key).await?;
        if !exists {
            return Ok(Vec::new());
        }

        // 获取会话ID列表（使用ZREVRANGE，按时间倒序）
        let start = ((page - 1) * page_size) as isize;
        let end = start + page_size as isize - 1;

        let conversation_ids = self
            .client
            .zrevrange(&user_conversations_key, start, end)
            .await?;

        let mut summaries = Vec::new();
        for conversation_id in conversation_ids {
            // 获取会话信息
            if let Ok(Some(info_json)) = self.client.hget(&conversation_key, &conversation_id).await
            {
                if let Ok(info) = serde_json::from_str::<serde_json::Value>(&info_json) {
                    // 获取消息数量
                    let messages_key =
                        CacheKey::ai_conversation_messages(user_id, &conversation_id);
                    let message_count = self.client.llen(&messages_key).await.unwrap_or(0);

                    let summary = AIConversationSummary {
                        id: conversation_id.clone(),
                        title: info
                            .get("title")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        created_at: info
                            .get("created_at")
                            .and_then(|v| v.as_str())
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now),
                        updated_at: info
                            .get("updated_at")
                            .and_then(|v| v.as_str())
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now),
                        message_count,
                    };
                    summaries.push(summary);
                }
            }
        }

        debug!(
            "Retrieved {} AI conversations from Redis (user: {})",
            summaries.len(),
            user_id
        );

        Ok(summaries)
    }

    /// 创建新的AI对话会话
    #[allow(dead_code)]
    pub async fn create_conversation(&self, user_id: &str, title: Option<&str>) -> Result<String> {
        let conversation_id = Uuid::new_v4().to_string();
        let conversation_key = CacheKey::ai_conversation(user_id);
        let user_conversations_key = CacheKey::ai_user_conversations(user_id);

        let now = Utc::now();
        let conversation_info = serde_json::json!({
            "id": &conversation_id,
            "title": title,
            "created_at": now,
            "updated_at": now,
        });

        // 保存会话信息
        self.client
            .hset(
                &conversation_key,
                &conversation_id,
                &conversation_info.to_string(),
            )
            .await?;

        self.client
            .expire(&conversation_key, self.default_ttl_secs)
            .await?;

        // 添加到用户会话列表
        let score = now.timestamp();
        self.client
            .zadd(&user_conversations_key, &conversation_id, score)
            .await?;
        self.client
            .expire(&user_conversations_key, self.default_ttl_secs)
            .await?;

        info!(
            "Created new AI conversation: {} (user: {})",
            conversation_id, user_id
        );

        Ok(conversation_id)
    }

    /// 更新AI对话会话标题
    #[allow(dead_code)]
    pub async fn update_conversation_title(
        &self,
        user_id: &str,
        conversation_id: &str,
        title: &str,
    ) -> Result<()> {
        let conversation_key = CacheKey::ai_conversation(user_id);

        // 获取现有信息
        if let Ok(Some(info_json)) = self.client.hget(&conversation_key, conversation_id).await {
            if let Ok(mut info) = serde_json::from_str::<serde_json::Value>(&info_json) {
                // 更新标题
                if let Some(obj) = info.as_object_mut() {
                    obj.insert("title".to_string(), serde_json::json!(title));
                    obj.insert("updated_at".to_string(), serde_json::json!(Utc::now()));
                }

                // 保存更新后的信息
                self.client
                    .hset(&conversation_key, conversation_id, &info.to_string())
                    .await?;

                debug!(
                    "Updated AI conversation title: {} (user: {})",
                    conversation_id, user_id
                );
            }
        }

        Ok(())
    }

    /// 删除AI对话会话
    #[allow(dead_code)]
    pub async fn delete_conversation(&self, user_id: &str, conversation_id: &str) -> Result<()> {
        let messages_key = CacheKey::ai_conversation_messages(user_id, conversation_id);
        let conversation_key = CacheKey::ai_conversation(user_id);
        let user_conversations_key = CacheKey::ai_user_conversations(user_id);

        // 删除消息列表
        self.client.delete(&messages_key).await?;

        // 从会话Hash中删除
        self.client.hdel(&conversation_key, conversation_id).await?;

        // 从用户会话列表中删除
        self.client
            .zrem(&user_conversations_key, conversation_id)
            .await?;

        info!(
            "Deleted AI conversation: {} (user: {})",
            conversation_id, user_id
        );

        Ok(())
    }

    /// 删除用户的所有AI对话记录
    #[allow(dead_code)]
    pub async fn delete_all_user_conversations(&self, user_id: &str) -> Result<usize> {
        let user_conversations_key = CacheKey::ai_user_conversations(user_id);
        let conversation_key = CacheKey::ai_conversation(user_id);

        // 获取所有会话ID
        let conversation_ids = self.client.zrange(&user_conversations_key, 0, -1).await?;
        let count = conversation_ids.len();

        // 删除每个会话的消息
        for conversation_id in &conversation_ids {
            let messages_key = CacheKey::ai_conversation_messages(user_id, conversation_id);
            if let Err(e) = self.client.delete(&messages_key).await {
                warn!(
                    "Failed to delete messages for conversation {}: {}",
                    conversation_id, e
                );
            }
        }

        // 删除会话Hash
        self.client.delete(&conversation_key).await?;

        // 删除用户会话列表
        self.client.delete(&user_conversations_key).await?;

        info!(
            "Deleted all AI conversations for user: {} (count: {})",
            user_id, count
        );

        Ok(count)
    }

    /// 清理过期的AI对话记录
    /// 注意：由于我们设置了TTL，Redis会自动清理过期的键
    /// 这个方法用于手动清理或处理特殊情况
    #[allow(dead_code)]
    pub async fn cleanup_expired_conversations(&self, _max_age_days: i64) -> Result<usize> {
        // Redis会自动处理过期键，这里可以添加额外的清理逻辑
        // 例如扫描并删除孤立的会话数据
        info!("AI conversation cleanup completed (Redis handles TTL automatically)");
        Ok(0)
    }

    /// 获取用户的会话数量
    #[allow(dead_code)]
    pub async fn get_user_conversation_count(&self, user_id: &str) -> Result<usize> {
        let user_conversations_key = CacheKey::ai_user_conversations(user_id);
        self.client.zcard(&user_conversations_key).await
    }

    /// 检查会话是否存在
    #[allow(dead_code)]
    pub async fn conversation_exists(&self, user_id: &str, conversation_id: &str) -> Result<bool> {
        let conversation_key = CacheKey::ai_conversation(user_id);
        self.client
            .hexists(&conversation_key, conversation_id)
            .await
    }

    /// 获取会话的消息数量
    #[allow(dead_code)]
    pub async fn get_conversation_message_count(
        &self,
        user_id: &str,
        conversation_id: &str,
    ) -> Result<usize> {
        let messages_key = CacheKey::ai_conversation_messages(user_id, conversation_id);
        self.client.llen(&messages_key).await
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AIChatStats {
    pub total_conversations: usize,
    pub total_messages: usize,
    pub storage_size_bytes: usize,
}
