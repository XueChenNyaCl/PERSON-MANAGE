use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::api::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateScoreRequest {
    pub target_id: i32,
    pub target_type: String, // person or group
    pub score: i32,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ScoreResponse {
    pub id: i32,
    pub target_id: i32,
    pub target_name: String,
    pub target_type: String,
    pub score: i32,
    pub reason: String,
    pub create_time: String,
}

pub async fn list(
    State(_state): State<AppState>,
) -> Result<Json<Vec<ScoreResponse>>, (StatusCode, String)> {
    // 模拟数据
    let scores = vec![
        ScoreResponse {
            id: 1,
            target_id: 1,
            target_name: "张三".to_string(),
            target_type: "person".to_string(),
            score: 10,
            reason: "作业完成优秀".to_string(),
            create_time: "2023-09-01 10:00:00".to_string(),
        },
        ScoreResponse {
            id: 2,
            target_id: 2,
            target_name: "李四".to_string(),
            target_type: "person".to_string(),
            score: -5,
            reason: "迟到".to_string(),
            create_time: "2023-09-01 08:15:00".to_string(),
        },
        ScoreResponse {
            id: 3,
            target_id: 1,
            target_name: "第一小组".to_string(),
            target_type: "group".to_string(),
            score: 20,
            reason: "团队合作优秀".to_string(),
            create_time: "2023-09-01 15:00:00".to_string(),
        },
    ];

    Ok(Json(scores))
}

pub async fn create(
    State(_state): State<AppState>,
    Json(create_req): Json<CreateScoreRequest>,
) -> Result<Json<ScoreResponse>, (StatusCode, String)> {
    // 模拟创建
    let score = ScoreResponse {
        id: 4,
        target_id: create_req.target_id,
        target_name: "张三".to_string(),
        target_type: create_req.target_type,
        score: create_req.score,
        reason: create_req.reason,
        create_time: "2023-09-01 10:00:00".to_string(),
    };

    Ok(Json(score))
}
