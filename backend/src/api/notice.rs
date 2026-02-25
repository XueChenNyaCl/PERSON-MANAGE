use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::api::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateNoticeRequest {
    pub title: String,
    pub content: String,
    pub notice_type: String, // school or class
    pub author: String,
}

#[derive(Debug, Serialize)]
pub struct NoticeResponse {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub notice_type: String,
    pub author: String,
    pub create_time: String,
}

pub async fn list(
    State(_state): State<AppState>,
) -> Result<Json<Vec<NoticeResponse>>, (StatusCode, String)> {
    // 模拟数据
    let notices = vec![
        NoticeResponse {
            id: 1,
            title: "开学通知".to_string(),
            content: "新学期将于9月1日正式开始，请同学们做好准备。".to_string(),
            notice_type: "school".to_string(),
            author: "教务处".to_string(),
            create_time: "2023-08-15 10:00:00".to_string(),
        },
        NoticeResponse {
            id: 2,
            title: "家长会通知".to_string(),
            content: "高一(1)班将于9月10日召开家长会，请家长准时参加。".to_string(),
            notice_type: "class".to_string(),
            author: "班主任".to_string(),
            create_time: "2023-09-05 14:00:00".to_string(),
        },
        NoticeResponse {
            id: 3,
            title: "运动会安排".to_string(),
            content: "学校将于10月1日举办秋季运动会，具体安排如下。".to_string(),
            notice_type: "school".to_string(),
            author: "体育组".to_string(),
            create_time: "2023-09-20 09:00:00".to_string(),
        },
    ];

    Ok(Json(notices))
}

pub async fn create(
    State(_state): State<AppState>,
    Json(create_req): Json<CreateNoticeRequest>,
) -> Result<Json<NoticeResponse>, (StatusCode, String)> {
    // 模拟创建
    let notice = NoticeResponse {
        id: 4,
        title: create_req.title,
        content: create_req.content,
        notice_type: create_req.notice_type,
        author: create_req.author,
        create_time: "2023-09-01 10:00:00".to_string(),
    };

    Ok(Json(notice))
}
