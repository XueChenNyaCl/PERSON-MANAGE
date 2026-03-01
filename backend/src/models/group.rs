use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Group {
    pub id: Uuid,
    pub class_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub score: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupCreate {
    pub class_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub class_id: Uuid,
    pub class_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub score: i32,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct GroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub person_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupMemberAdd {
    pub person_id: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct GroupScoreRecord {
    pub id: Uuid,
    pub group_id: Uuid,
    pub score_change: i32,
    pub reason: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupScoreChange {
    pub score_change: i32,
    pub reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupMemberResponse {
    pub id: Uuid,
    pub name: String,
    pub gender: i16,
    pub birthday: Option<NaiveDate>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub student_no: String,
    pub class_id: Option<Uuid>,
    pub class_name: Option<String>,
    pub enrollment_date: Option<NaiveDate>,
    pub status: String,
}
