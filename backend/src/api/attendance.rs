use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::api::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateAttendanceRequest {
    pub person_id: i32,
    pub date: String,
    pub status: String,
    pub time: String,
    pub remark: String,
}

#[derive(Debug, Serialize)]
pub struct AttendanceResponse {
    pub id: i32,
    pub person_id: i32,
    pub name: String,
    pub date: String,
    pub status: String,
    pub time: String,
    pub remark: String,
}

pub async fn list(
    State(_state): State<AppState>,
) -> Result<Json<Vec<AttendanceResponse>>, (StatusCode, String)> {
    // 模拟数据
    let attendances = vec![
        AttendanceResponse {
            id: 1,
            person_id: 1,
            name: "张三".to_string(),
            date: "2023-09-01".to_string(),
            status: "正常".to_string(),
            time: "08:00:00".to_string(),
            remark: "".to_string(),
        },
        AttendanceResponse {
            id: 2,
            person_id: 2,
            name: "李四".to_string(),
            date: "2023-09-01".to_string(),
            status: "迟到".to_string(),
            time: "08:15:00".to_string(),
            remark: "交通堵塞".to_string(),
        },
        AttendanceResponse {
            id: 3,
            person_id: 3,
            name: "王五".to_string(),
            date: "2023-09-01".to_string(),
            status: "缺勤".to_string(),
            time: "".to_string(),
            remark: "请假".to_string(),
        },
    ];

    Ok(Json(attendances))
}

pub async fn create(
    State(_state): State<AppState>,
    Json(create_req): Json<CreateAttendanceRequest>,
) -> Result<Json<AttendanceResponse>, (StatusCode, String)> {
    // 模拟创建
    let attendance = AttendanceResponse {
        id: 4,
        person_id: create_req.person_id,
        name: "张三".to_string(),
        date: create_req.date,
        status: create_req.status,
        time: create_req.time,
        remark: create_req.remark,
    };

    Ok(Json(attendance))
}
