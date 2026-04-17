use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AI操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum AIActionType {
    // 人员管理
    CreatePerson,
    CreatePersonsBatch,
    UpdatePerson,
    DeletePerson,
    GetPersons,

    // 考勤管理
    CreateAttendance,
    CreateAttendancesBatch,
    UpdateAttendance,
    DeleteAttendance,
    GetAttendances,

    // 成绩/积分管理
    CreateScore,
    CreateScoresBatch,
    UpdateScore,
    DeleteScore,
    GetScores,

    // 公告管理
    CreateNotice,
    UpdateNotice,
    DeleteNotice,
    GetNotices,

    // 班级管理
    GetClasses,
    GetClassDetail,

    // 小组管理
    GetGroups,
    GetGroupDetail,
    CreateGroup,
    UpdateGroupScore,
    AddGroupMember,
    RemoveGroupMember,

    // 部门管理
    GetDepartments,
    GetDepartmentDetail,
}

#[allow(dead_code)]
impl AIActionType {
    /// 获取操作所需的权限
    pub fn required_permission(&self) -> &'static str {
        match self {
            // 人员管理
            AIActionType::CreatePerson | AIActionType::CreatePersonsBatch => "person.create",
            AIActionType::UpdatePerson => "person.update",
            AIActionType::DeletePerson => "person.delete",
            AIActionType::GetPersons => "person.view",

            // 考勤管理
            AIActionType::CreateAttendance | AIActionType::CreateAttendancesBatch => {
                "attendance.create"
            }
            AIActionType::UpdateAttendance => "attendance.update",
            AIActionType::DeleteAttendance => "attendance.delete",
            AIActionType::GetAttendances => "attendance.view",

            // 成绩/积分管理
            AIActionType::CreateScore | AIActionType::CreateScoresBatch => "score.create",
            AIActionType::UpdateScore => "score.update",
            AIActionType::DeleteScore => "score.delete",
            AIActionType::GetScores => "score.view",

            // 公告管理
            AIActionType::CreateNotice => "notice.create",
            AIActionType::UpdateNotice => "notice.update",
            AIActionType::DeleteNotice => "notice.delete",
            AIActionType::GetNotices => "notice.view",

            // 班级管理
            AIActionType::GetClasses | AIActionType::GetClassDetail => "class.view",

            // 小组管理
            AIActionType::GetGroups | AIActionType::GetGroupDetail => "group.view",
            AIActionType::CreateGroup => "group.create",
            AIActionType::UpdateGroupScore => "group.update.score",
            AIActionType::AddGroupMember | AIActionType::RemoveGroupMember => "group.update.member",

            // 部门管理
            AIActionType::GetDepartments | AIActionType::GetDepartmentDetail => "department.view",
        }
    }

    /// 是否是批量操作
    pub fn is_batch(&self) -> bool {
        matches!(
            self,
            AIActionType::CreatePersonsBatch
                | AIActionType::CreateAttendancesBatch
                | AIActionType::CreateScoresBatch
        )
    }

    /// 是否是查询操作
    pub fn is_query(&self) -> bool {
        matches!(
            self,
            AIActionType::GetPersons
                | AIActionType::GetAttendances
                | AIActionType::GetScores
                | AIActionType::GetNotices
                | AIActionType::GetClasses
                | AIActionType::GetClassDetail
                | AIActionType::GetGroups
                | AIActionType::GetGroupDetail
                | AIActionType::GetDepartments
                | AIActionType::GetDepartmentDetail
        )
    }
}

impl std::str::FromStr for AIActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create_person" => Ok(AIActionType::CreatePerson),
            "create_persons_batch" => Ok(AIActionType::CreatePersonsBatch),
            "update_person" => Ok(AIActionType::UpdatePerson),
            "delete_person" => Ok(AIActionType::DeletePerson),
            "get_persons" => Ok(AIActionType::GetPersons),

            "create_attendance" => Ok(AIActionType::CreateAttendance),
            "create_attendances_batch" => Ok(AIActionType::CreateAttendancesBatch),
            "update_attendance" => Ok(AIActionType::UpdateAttendance),
            "delete_attendance" => Ok(AIActionType::DeleteAttendance),
            "get_attendances" => Ok(AIActionType::GetAttendances),

            "create_score" => Ok(AIActionType::CreateScore),
            "create_scores_batch" => Ok(AIActionType::CreateScoresBatch),
            "update_score" => Ok(AIActionType::UpdateScore),
            "delete_score" => Ok(AIActionType::DeleteScore),
            "get_scores" => Ok(AIActionType::GetScores),

            "create_notice" => Ok(AIActionType::CreateNotice),
            "update_notice" => Ok(AIActionType::UpdateNotice),
            "delete_notice" => Ok(AIActionType::DeleteNotice),
            "get_notices" => Ok(AIActionType::GetNotices),

            "get_classes" => Ok(AIActionType::GetClasses),
            "get_class_detail" => Ok(AIActionType::GetClassDetail),

            "get_groups" => Ok(AIActionType::GetGroups),
            "get_group_detail" => Ok(AIActionType::GetGroupDetail),
            "create_group" => Ok(AIActionType::CreateGroup),
            "update_group_score" => Ok(AIActionType::UpdateGroupScore),
            "add_group_member" => Ok(AIActionType::AddGroupMember),
            "remove_group_member" => Ok(AIActionType::RemoveGroupMember),

            "get_departments" => Ok(AIActionType::GetDepartments),
            "get_department_detail" => Ok(AIActionType::GetDepartmentDetail),

            _ => Err(format!("未知的操作类型: {}", s)),
        }
    }
}

/// AI操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AIActionRequest {
    /// 操作类型
    pub action_type: AIActionType,
    /// 操作参数
    pub params: serde_json::Value,
    /// 操作原因/说明
    pub reason: String,
    /// 是否是批量操作
    #[serde(default)]
    pub batch: bool,
    /// 批量操作的items（仅用于批量操作）
    #[serde(default)]
    pub items: Vec<serde_json::Value>,
}

/// AI操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AIActionResponse {
    /// 是否成功
    pub success: bool,
    /// 响应消息
    pub message: String,
    /// 操作结果数据
    pub data: Option<serde_json::Value>,
    /// 用户权限列表
    pub user_permissions: Vec<String>,
    /// 是否需要用户确认（用于重名情况）
    #[serde(default)]
    pub need_confirmation: bool,
    /// 候选项（用于重名情况）
    pub candidates: Option<Vec<NameCandidate>>,
    /// 批量操作结果（仅用于批量操作）
    pub batch_result: Option<BatchOperationResult>,
}

/// 名称候选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NameCandidate {
    pub id: String,
    pub name: String,
    pub info: String,
}

/// 批量操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BatchOperationResult {
    /// 总数
    pub total: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub failure_count: usize,
    /// 每个项目的结果
    pub items: Vec<BatchItemResult>,
}

/// 批量操作单项结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BatchItemResult {
    /// 是否成功
    pub success: bool,
    /// 索引
    pub index: usize,
    /// 数据（成功时）
    pub data: Option<serde_json::Value>,
    /// 错误信息（失败时）
    pub error: Option<String>,
}

/// 多步骤操作会话
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MultiStepSession {
    /// 会话ID
    pub session_id: Uuid,
    /// 用户ID
    pub user_id: Uuid,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 状态
    pub status: SessionStatus,
    /// 步骤列表
    pub steps: Vec<StepInfo>,
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum SessionStatus {
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// 步骤信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct StepInfo {
    /// 步骤ID
    pub step_id: String,
    /// 步骤序号
    pub step_number: usize,
    /// 操作类型
    pub action_type: AIActionType,
    /// 状态
    pub status: StepStatus,
    /// 执行结果
    pub result: Option<serde_json::Value>,
    /// 依赖的步骤ID
    pub depends_on: Option<String>,
    /// 执行时间
    pub executed_at: Option<DateTime<Utc>>,
}

/// 步骤状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum StepStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    Skipped,
}

/// 多步骤操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MultiStepRequest {
    /// 会话ID（新会话为null）
    pub session_id: Option<Uuid>,
    /// 当前步骤
    pub current_step: StepRequest,
    /// 总步骤数
    pub total_steps: usize,
}

/// 步骤请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct StepRequest {
    /// 步骤ID
    pub step_id: String,
    /// 步骤序号
    pub step_number: usize,
    /// 操作请求
    pub action: AIActionRequest,
    /// 依赖的步骤ID
    pub depends_on: Option<String>,
}

/// 多步骤操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MultiStepResponse {
    /// 是否成功
    pub success: bool,
    /// 会话ID
    pub session_id: Uuid,
    /// 当前步骤结果
    pub current_step_result: serde_json::Value,
    /// 会话状态
    pub session_status: SessionStatus,
    /// 已完成步骤数
    pub completed_steps: usize,
    /// 剩余步骤数
    pub remaining_steps: usize,
    /// 下一步建议
    pub next_step_suggestions: Vec<String>,
}

/// 可用的操作信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AvailableAction {
    /// 操作类型
    pub action_type: String,
    /// 操作名称
    pub name: String,
    /// 操作描述
    pub description: String,
    /// 所需权限
    pub required_permission: String,
    /// 是否是批量操作
    pub supports_batch: bool,
    /// 必需参数
    pub required_params: Vec<String>,
    /// 可选参数
    pub optional_params: Vec<String>,
    /// 参数提示
    pub param_tips: Option<serde_json::Value>,
}

/// 页面上下文数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PageContextData {
    /// 页面名称
    pub page: String,
    /// 页面路径
    pub path: String,
    /// 页面数据
    pub data: serde_json::Value,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// AI助手建议请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AssistantSuggestionRequest {
    /// 页面上下文
    pub page_context: serde_json::Value,
    /// 页面路径
    pub path: Option<String>,
    /// 页面名称
    pub name: Option<String>,
}

/// AI助手建议响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AssistantSuggestionResponse {
    /// 建议内容
    pub suggestion: String,
}
