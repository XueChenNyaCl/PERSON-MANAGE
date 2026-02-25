use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Attendance {
    pub id: Uuid,
    pub person_id: Uuid,
    pub date: chrono::NaiveDate,
    pub status: String, // present, absent, late, excused
    pub time: Option<chrono::NaiveTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttendanceCreate {
    pub person_id: Uuid,
    pub date: chrono::NaiveDate,
    pub status: String,
    pub time: Option<chrono::NaiveTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttendanceUpdate {
    pub status: Option<String>,
    pub time: Option<chrono::NaiveTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttendanceResponse {
    pub id: Uuid,
    pub person_id: Uuid,
    pub date: chrono::NaiveDate,
    pub status: String,
    pub time: Option<chrono::NaiveTime>,
}

impl From<Attendance> for AttendanceResponse {
    fn from(attendance: Attendance) -> Self {
        Self {
            id: attendance.id,
            person_id: attendance.person_id,
            date: attendance.date,
            status: attendance.status,
            time: attendance.time,
        }
    }
}
