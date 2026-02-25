use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Department {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DepartmentCreate {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DepartmentUpdate {
    pub name: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DepartmentResponse {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub parent_name: Option<String>, // 上级部门名称
    pub created_at: DateTime<Utc>,
}

impl From<Department> for DepartmentResponse {
    fn from(department: Department) -> Self {
        Self {
            id: department.id,
            name: department.name,
            parent_id: department.parent_id,
            parent_name: None, // 需要额外查询
            created_at: department.created_at,
        }
    }
}
