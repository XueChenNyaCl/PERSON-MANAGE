use axum::{
    extract::{Extension, Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::chat_scope::{ensure_conversation_member, get_chat_targets};
use crate::core::error::AppError;
use crate::models::chat::{
    ChatConversationResponse, ChatMessageResponse, ChatSendMessageRequest,
};

pub async fn list_conversations(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<ChatConversationResponse>>, AppError> {
    let pool = state.pool.ok_or(AppError::Internal)?;
    let user_id = parse_user_id(&claims)?;

    ensure_default_conversations(&pool, user_id, &claims.role).await?;
    let conversations = load_user_conversations(&pool, user_id).await?;

    Ok(Json(conversations))
}

pub async fn list_messages(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Vec<ChatMessageResponse>>, AppError> {
    let pool = state.pool.ok_or(AppError::Internal)?;
    let user_id = parse_user_id(&claims)?;

    ensure_conversation_member(&pool, conversation_id, user_id).await?;
    Ok(Json(load_conversation_messages(&pool, conversation_id, user_id).await?))
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<ChatSendMessageRequest>,
) -> Result<Json<ChatMessageResponse>, AppError> {
    let pool = state.pool.ok_or(AppError::Internal)?;
    let user_id = parse_user_id(&claims)?;

    ensure_conversation_member(&pool, conversation_id, user_id).await?;

    let content = payload.content.trim();
    if content.is_empty() {
        return Err(AppError::InvalidInput("消息内容不能为空".to_string()));
    }

    let message_type = payload.message_type.unwrap_or_else(|| "text".to_string());

    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO chat_messages (conversation_id, sender_id, content, message_type)
         VALUES ($1, $2, $3, $4)
         RETURNING id"
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(content)
    .bind(&message_type)
    .fetch_one(&pool)
    .await?;

    sqlx::query("UPDATE chat_conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(&pool)
        .await?;

    Ok(Json(load_message_by_id(&pool, message_id, user_id).await?))
}

pub async fn mark_read(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<ReadReceiptResponse>, AppError> {
    let pool = state.pool.ok_or(AppError::Internal)?;
    let user_id = parse_user_id(&claims)?;

    ensure_conversation_member(&pool, conversation_id, user_id).await?;

    sqlx::query(
        "UPDATE chat_conversation_members SET last_read_at = NOW() WHERE conversation_id = $1 AND user_id = $2"
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(&pool)
    .await?;

    Ok(Json(ReadReceiptResponse {
        conversation_id,
        read_at: Utc::now(),
    }))
}

#[derive(Debug, Deserialize, sqlx::FromRow)]
struct ConversationRow {
    id: Uuid,
    conversation_type: String,
    updated_at: DateTime<Utc>,
    peer_user_id: Uuid,
    peer_name: String,
    peer_role: String,
    last_message: Option<String>,
    last_message_at: Option<DateTime<Utc>>,
    unread_count: Option<i64>,
}

#[derive(Debug, Deserialize, sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    conversation_id: Uuid,
    sender_id: Uuid,
    sender_name: String,
    content: String,
    message_type: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReadReceiptResponse {
    pub conversation_id: Uuid,
    pub read_at: DateTime<Utc>,
}

fn parse_user_id(claims: &Claims) -> Result<Uuid, AppError> {
    Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))
}

async fn ensure_default_conversations(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    role: &str,
) -> Result<(), AppError> {
    let targets = get_chat_targets(pool, user_id, role).await?;

    for target_id in targets {
        get_or_create_direct_conversation(pool, user_id, target_id).await?;
    }

    Ok(())
}

async fn get_or_create_direct_conversation(
    pool: &sqlx::PgPool,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<Uuid, AppError> {
    let pair_key = build_pair_key(user_a, user_b);

    let conversation_id: Uuid = if let Some(id) = sqlx::query_scalar(
        "SELECT id FROM chat_conversations WHERE pair_key = $1"
    )
    .bind(&pair_key)
    .fetch_optional(pool)
    .await?
    {
        id
    } else {
        sqlx::query_scalar(
            "INSERT INTO chat_conversations (conversation_type, pair_key)
             VALUES ('direct', $1)
             RETURNING id"
        )
        .bind(&pair_key)
        .fetch_one(pool)
        .await?
    };

    sqlx::query(
        "INSERT INTO chat_conversation_members (conversation_id, user_id)
         VALUES ($1, $2)
         ON CONFLICT (conversation_id, user_id) DO NOTHING"
    )
    .bind(conversation_id)
    .bind(user_a)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO chat_conversation_members (conversation_id, user_id)
         VALUES ($1, $2)
         ON CONFLICT (conversation_id, user_id) DO NOTHING"
    )
    .bind(conversation_id)
    .bind(user_b)
    .execute(pool)
    .await?;

    Ok(conversation_id)
}

fn build_pair_key(user_a: Uuid, user_b: Uuid) -> String {
    let mut ids = [user_a.to_string(), user_b.to_string()];
    ids.sort();
    format!("direct:{}:{}", ids[0], ids[1])
}

async fn load_user_conversations(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<ChatConversationResponse>, AppError> {
    let rows: Vec<ConversationRow> = sqlx::query_as(
        "SELECT c.id,
                c.conversation_type,
                c.updated_at,
                peer.id AS peer_user_id,
                peer.name AS peer_name,
                COALESCE(peer.role, peer.type) AS peer_role,
                last_msg.content AS last_message,
                last_msg.created_at AS last_message_at,
                unread.unread_count AS unread_count
         FROM chat_conversation_members self_member
         JOIN chat_conversations c ON c.id = self_member.conversation_id
         JOIN chat_conversation_members peer_member
           ON peer_member.conversation_id = c.id AND peer_member.user_id <> self_member.user_id
         JOIN persons peer ON peer.id = peer_member.user_id
         LEFT JOIN LATERAL (
             SELECT m.content, m.created_at
             FROM chat_messages m
             WHERE m.conversation_id = c.id
             ORDER BY m.created_at DESC
             LIMIT 1
         ) last_msg ON TRUE
         LEFT JOIN LATERAL (
             SELECT COUNT(*)::BIGINT AS unread_count
             FROM chat_messages m
             WHERE m.conversation_id = c.id
               AND m.sender_id <> $1
               AND (
                   self_member.last_read_at IS NULL
                   OR m.created_at > self_member.last_read_at
               )
         ) unread ON TRUE
         WHERE self_member.user_id = $1
         ORDER BY COALESCE(last_msg.created_at, c.updated_at) DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ChatConversationResponse {
            id: row.id,
            conversation_type: row.conversation_type,
            peer_user_id: row.peer_user_id,
            peer_name: row.peer_name,
            peer_role: row.peer_role,
            last_message: row.last_message,
            last_message_at: row.last_message_at,
            unread_count: row.unread_count.unwrap_or(0),
            updated_at: row.updated_at,
        })
        .collect())
}

async fn load_conversation_messages(
    pool: &sqlx::PgPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<ChatMessageResponse>, AppError> {
    let rows: Vec<MessageRow> = sqlx::query_as(
        "SELECT m.id,
                m.conversation_id,
                m.sender_id,
                p.name AS sender_name,
                m.content,
                m.message_type,
                m.created_at
         FROM chat_messages m
         JOIN persons p ON p.id = m.sender_id
         WHERE m.conversation_id = $1
         ORDER BY m.created_at ASC"
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ChatMessageResponse {
            id: row.id,
            conversation_id: row.conversation_id,
            sender_id: row.sender_id,
            sender_name: row.sender_name,
            content: row.content,
            message_type: row.message_type,
            created_at: row.created_at,
            is_self: row.sender_id == user_id,
        })
        .collect())
}

async fn load_message_by_id(
    pool: &sqlx::PgPool,
    message_id: Uuid,
    user_id: Uuid,
) -> Result<ChatMessageResponse, AppError> {
    let row: Option<MessageRow> = sqlx::query_as(
        "SELECT m.id,
                m.conversation_id,
                m.sender_id,
                p.name AS sender_name,
                m.content,
                m.message_type,
                m.created_at
         FROM chat_messages m
         JOIN persons p ON p.id = m.sender_id
         WHERE m.id = $1"
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or(AppError::NotFound)?;

    Ok(ChatMessageResponse {
        id: row.id,
        conversation_id: row.conversation_id,
        sender_id: row.sender_id,
        sender_name: row.sender_name,
        content: row.content,
        message_type: row.message_type,
        created_at: row.created_at,
        is_self: row.sender_id == user_id,
    })
}