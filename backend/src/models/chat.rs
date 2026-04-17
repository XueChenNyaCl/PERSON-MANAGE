use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct ChatConversation {
    pub id: Uuid,
    pub conversation_type: String,
    pub pair_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct ChatConversationMember {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatConversationResponse {
    pub id: Uuid,
    pub conversation_type: String,
    pub peer_user_id: Uuid,
    pub peer_name: String,
    pub peer_role: String,
    pub last_message: Option<String>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub unread_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub created_at: DateTime<Utc>,
    pub is_self: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatSendMessageRequest {
    pub content: String,
    pub message_type: Option<String>,
}
