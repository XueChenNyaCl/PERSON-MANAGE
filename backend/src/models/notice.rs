use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Notice {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub target_type: String, // school, class, department
    pub target_id: Option<Uuid>,
    pub attachments: Option<Vec<String>>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NoticeCreate {
    pub title: String,
    pub content: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NoticeUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NoticeResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub attachments: Option<Vec<String>>,
    pub created_at: chrono::NaiveDateTime,
}

impl From<Notice> for NoticeResponse {
    fn from(notice: Notice) -> Self {
        Self {
            id: notice.id,
            title: notice.title,
            content: notice.content,
            author_id: notice.author_id,
            target_type: notice.target_type,
            target_id: notice.target_id,
            attachments: notice.attachments,
            created_at: notice.created_at,
        }
    }
}
