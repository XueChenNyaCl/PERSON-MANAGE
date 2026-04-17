use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 特殊用户类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
pub enum SpecialUserType {
    System,   // 系统用户，不可登录
    IoT,      // 物联网设备用户
    Scerm,    // 大屏用户
    SysAI,    // 系统AI，暂留
    ChatAI,   // 聊天AI，记录用户操作
}

impl std::fmt::Display for SpecialUserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialUserType::System => write!(f, "system"),
            SpecialUserType::IoT => write!(f, "iot"),
            SpecialUserType::Scerm => write!(f, "scerm"),
            SpecialUserType::SysAI => write!(f, "sysai"),
            SpecialUserType::ChatAI => write!(f, "chatai"),
        }
    }
}

/// 特殊用户数据库模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SpecialUser {
    pub id: Uuid,
    pub user_type: String,
    pub identifier: String,
    pub linked_person_id: Option<Uuid>,
    pub api_key_hash: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 特殊用户响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialUserResponse {
    pub id: Uuid,
    pub user_type: String,
    pub identifier: String,
    pub linked_person_id: Option<Uuid>,
    pub linked_person_name: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<SpecialUser> for SpecialUserResponse {
    fn from(user: SpecialUser) -> Self {
        Self {
            id: user.id,
            user_type: user.user_type,
            identifier: user.identifier,
            linked_person_id: user.linked_person_id,
            linked_person_name: None, // 需要额外查询填充
            description: user.description,
            is_active: user.is_active,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
        }
    }
}

/// 创建特殊用户请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSpecialUserRequest {
    pub user_type: String,  // 'iot', 'scerm'
    pub identifier: String, // 如 'device001', 'screen01'
    pub description: Option<String>,
    pub api_key: Option<String>, // 初始API密钥，后端会哈希存储
}

/// 更新特殊用户请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSpecialUserRequest {
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// 关联人员请求
#[derive(Debug, Clone, Deserialize)]
pub struct LinkPersonRequest {
    pub person_id: Uuid,
}

/// 特殊用户登录请求（IoT/Scerm）
#[derive(Debug, Clone, Deserialize)]
pub struct SpecialUserLoginRequest {
    pub identifier: String,
    pub api_key: String,
}

/// 特殊用户登录响应
#[derive(Debug, Clone, Serialize)]
pub struct SpecialUserLoginResponse {
    pub token: String,
    pub user_type: String,
    pub identifier: String,
    pub expires_in: u64,
}

/// 操作日志数据库模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: Uuid,
    pub operator_id: Option<Uuid>,
    pub operator_type: String,
    pub operator_name: String,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 操作日志响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLogResponse {
    pub id: Uuid,
    pub operator_type: String,
    pub operator_name: String,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<OperationLog> for OperationLogResponse {
    fn from(log: OperationLog) -> Self {
        Self {
            id: log.id,
            operator_type: log.operator_type,
            operator_name: log.operator_name,
            action: log.action,
            resource_type: log.resource_type,
            resource_id: log.resource_id,
            details: log.details,
            created_at: log.created_at,
        }
    }
}

/// 操作者信息（用于日志记录）
#[derive(Debug, Clone)]
pub struct OperatorInfo {
    pub operator_id: Option<Uuid>,
    pub operator_type: String,  // 'system', 'admin', 'user', 'chatai', 'sysai'
    pub operator_name: String,  // 如 'admin:zhangsan', '501001', 'system'
}

impl OperatorInfo {
    /// 创建系统操作者
    pub fn system() -> Self {
        Self {
            operator_id: None,
            operator_type: "system".to_string(),
            operator_name: "system".to_string(),
        }
    }

    /// 创建普通用户操作者
    pub fn user(user_id: Uuid, identifier: impl Into<String>) -> Self {
        Self {
            operator_id: Some(user_id),
            operator_type: "user".to_string(),
            operator_name: identifier.into(),
        }
    }

    /// 创建admin用户操作者
    pub fn admin(user_id: Uuid, username: impl Into<String>) -> Self {
        Self {
            operator_id: Some(user_id),
            operator_type: "admin".to_string(),
            operator_name: format!("admin:{}", username.into()),
        }
    }

    /// 创建ChatAI操作者
    pub fn chatai(user_id: impl Into<String>) -> Self {
        Self {
            operator_id: None,
            operator_type: "chatai".to_string(),
            operator_name: format!("ChatAI:{}", user_id.into()),
        }
    }

    /// 创建SysAI操作者
    pub fn sysai() -> Self {
        Self {
            operator_id: None,
            operator_type: "sysai".to_string(),
            operator_name: "SysAI".to_string(),
        }
    }
}
