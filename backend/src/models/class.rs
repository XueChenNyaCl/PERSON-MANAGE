use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Class {
    pub id: Uuid,
    pub name: String,
    pub grade: i16,               // 年级，如 1 表示一年级
    pub teacher_id: Option<Uuid>, // 班主任
    pub academic_year: String,    // 学年，如 "2025-2026"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassCreate {
    pub name: String,
    pub grade: i32,
    pub teacher_id: Option<String>,
    pub academic_year: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassUpdate {
    pub name: Option<String>,
    pub grade: Option<i32>,
    pub teacher_id: Option<String>,
    pub academic_year: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassResponse {
    pub id: Uuid,
    pub name: String,
    pub grade: i16,
    pub teacher_id: Option<Uuid>,
    pub teacher_name: Option<String>, // 班主任姓名
    pub academic_year: String,
    pub created_at: DateTime<Utc>,
}

impl From<Class> for ClassResponse {
    fn from(class: Class) -> Self {
        Self {
            id: class.id,
            name: class.name,
            grade: class.grade,
            teacher_id: class.teacher_id,
            teacher_name: None, // 需要额外查询
            academic_year: class.academic_year,
            created_at: class.created_at,
        }
    }
}
