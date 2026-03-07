use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Local, NaiveDate, NaiveTime};
use tracing::{info, warn};

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;

// ========== AI 操作请求/响应结构 ==========

/// AI 操作请求
#[derive(Debug, Serialize, Deserialize)]
pub struct AIActionRequest {
    /// 操作类型
    pub action_type: String,
    /// 操作参数
    #[serde(default = "default_action_params")]
    pub params: serde_json::Value,
    /// 操作原因/说明
    pub reason: String,
    /// 是否批量（兼容AI返回）
    #[serde(default)]
    pub batch: bool,
    /// 批量项目（兼容AI返回顶层items）
    #[serde(default)]
    pub items: Vec<serde_json::Value>,
}

fn default_action_params() -> serde_json::Value {
    serde_json::json!({})
}

/// AI 操作响应
#[derive(Debug, Serialize)]
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
    pub need_confirmation: bool,
    /// 候选项（用于重名情况）
    pub candidates: Option<Vec<NameCandidate>>,
}

/// 名称候选项（用于重名选择）
#[derive(Debug, Serialize)]
pub struct NameCandidate {
    pub id: String,
    pub name: String,
    pub info: String, // 额外信息，如班级、学号等
}

/// 创建公告参数
#[derive(Debug, Deserialize)]
pub struct CreateNoticeParams {
    pub title: String,
    pub content: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub is_important: Option<bool>,
}

/// 创建小组参数
#[derive(Debug, Deserialize)]
pub struct CreateGroupParams {
    pub class_id: String,
    pub name: String,
    pub description: Option<String>,
}

/// 更新小组积分参数
#[derive(Debug, Deserialize)]
pub struct UpdateGroupScoreParams {
    #[serde(alias = "group_name", alias = "group")]
    pub group_id: String,
    pub score_change: i32,
    pub reason: String,
}

/// 添加小组成员参数
#[derive(Debug, Deserialize)]
pub struct AddGroupMemberParams {
    pub group_id: String,
    pub person_id: String,
}

/// 移除小组成员参数
#[derive(Debug, Deserialize)]
pub struct RemoveGroupMemberParams {
    pub group_id: String,
    pub person_id: String,
}

/// 创建考勤记录参数
#[derive(Debug, Deserialize)]
pub struct CreateAttendanceParams {
    pub person_id: String,
    pub date: String,
    pub status: String,
    pub time: Option<String>,
    pub remark: Option<String>,
}

/// 创建个人积分记录参数
#[derive(Debug, Deserialize)]
pub struct CreateScoreParams {
    #[serde(alias = "person_id", alias = "person", alias = "person_name", alias = "name", alias = "student")]
    pub student_id: String,
    pub reason: String,
    pub value: i32,
}

/// 创建人员参数
#[derive(Debug, Deserialize)]
pub struct CreatePersonParams {
    pub name: String,
    #[serde(default, alias = "type", alias = "personType", alias = "person_kind", alias = "role")]
    pub person_type: String,  // student, teacher, parent
    #[serde(deserialize_with = "deserialize_gender")]
    pub gender: i16,  // 0: 未知, 1: 男, 2: 女
    pub phone: Option<String>,
    pub email: Option<String>,
    pub birthday: Option<String>,  // YYYY-MM-DD
    // 学生特有
    pub student_no: Option<String>,
    pub class_id: Option<String>,  // 可以是班级名称或UUID
    pub enrollment_date: Option<String>,  // YYYY-MM-DD
    // 教师特有
    pub employee_no: Option<String>,
    pub department_id: Option<String>,  // 可以是部门名称或UUID
    pub title: Option<String>,
    pub hire_date: Option<String>,  // YYYY-MM-DD
}

/// 自定义反序列化函数，支持字符串和数字类型的gender
fn deserialize_gender<'de, D>(deserializer: D) -> Result<i16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    
    match value {
        serde_json::Value::Number(n) => {
            n.as_i64()
                .map(|v| v as i16)
                .ok_or_else(|| D::Error::custom("无效的gender数值"))
        }
        serde_json::Value::String(s) => {
            match s.to_lowercase().as_str() {
                "male" | "男" | "m" | "1" => Ok(1),
                "female" | "女" | "f" | "2" => Ok(2),
                "unknown" | "未知" | "u" | "0" | "" => Ok(0),
                _ => {
                    // 尝试解析数字字符串
                    s.parse::<i16>()
                        .map_err(|_| D::Error::custom(format!("未知的gender值: {}", s)))
                }
            }
        }
        _ => Err(D::Error::custom("gender必须是字符串或数字")),
    }
}

/// 批量创建人员参数
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreatePersonsBatchParams {
    pub items: Vec<CreatePersonParams>,
}

/// 批量操作结果项
#[derive(Debug, Serialize)]
pub struct BatchItemResult {
    pub success: bool,
    pub index: usize,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// ========== 名称解析服务 ==========

pub struct NameResolver;

impl NameResolver {
    /// 解析人员名称，返回人员ID
    /// 如果有多个人员同名，返回候选项
    pub async fn resolve_person(
        pool: &PgPool,
        name: &str,
    ) -> Result<ResolutionResult, AppError> {
        let keyword = name.trim();
        let fuzzy_keyword = format!("%{}%", keyword);

        // 首先尝试直接作为UUID解析
        if let Ok(uuid) = Uuid::parse_str(keyword) {
            // 检查该UUID是否存在
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM persons WHERE id = $1)"
            )
            .bind(uuid)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
            
            if exists {
                return Ok(ResolutionResult::Single(uuid.to_string()));
            }
        }
        
        // 按名称搜索人员
        let persons: Vec<PersonInfo> = sqlx::query_as(
            r#"SELECT 
                p.id, 
                p.name, 
                p.gender,
                COALESCE(s.student_no, t.employee_no, '-') as number,
                COALESCE(c.name, d.name, '无') as belong_info,
                CASE 
                    WHEN s.person_id IS NOT NULL THEN '学生'
                    WHEN t.person_id IS NOT NULL THEN '教师'
                    ELSE '其他'
                END as person_type
            FROM persons p
            LEFT JOIN students s ON p.id = s.person_id
            LEFT JOIN teachers t ON p.id = t.person_id
            LEFT JOIN classes c ON s.class_id = c.id
            LEFT JOIN departments d ON t.department_id = d.id
            WHERE p.name = $1
               OR p.name ILIKE $2
               OR s.student_no = $1
               OR t.employee_no = $1
               OR p.username = $1
               OR p.phone = $1
            ORDER BY p.name"#,
        )
        .bind(keyword)
        .bind(&fuzzy_keyword)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        
        if persons.is_empty() {
            return Ok(ResolutionResult::NotFound(format!("未找到人员: '{}'（可用姓名/学号/工号/用户名/手机号）", keyword)));
        }
        
        if persons.len() == 1 {
            return Ok(ResolutionResult::Single(persons[0].id.to_string()));
        }
        
        // 有多个同名人员，返回候选项
        let candidates: Vec<NameCandidate> = persons
            .iter()
            .map(|p| NameCandidate {
                id: p.id.to_string(),
                name: p.name.clone(),
                info: format!("{} - {} - {}", p.person_type, p.number, p.belong_info),
            })
            .collect();
        
        Ok(ResolutionResult::Multiple(candidates))
    }
    
    /// 解析小组名称，返回小组ID
    pub async fn resolve_group(
        pool: &PgPool,
        name: &str,
    ) -> Result<ResolutionResult, AppError> {
        // 首先尝试直接作为UUID解析
        if let Ok(uuid) = Uuid::parse_str(name) {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM class_groups WHERE id = $1)"
            )
            .bind(uuid)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
            
            if exists {
                return Ok(ResolutionResult::Single(uuid.to_string()));
            }
        }
        
        // 按名称搜索小组
        let groups: Vec<GroupInfo> = sqlx::query_as(
            r#"SELECT 
                cg.id, 
                cg.name, 
                c.name as class_name,
                cg.score
            FROM class_groups cg
            JOIN classes c ON cg.class_id = c.id
            WHERE cg.name = $1 OR cg.name ILIKE $1
            ORDER BY cg.name"#,
        )
        .bind(name)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        
        if groups.is_empty() {
            return Ok(ResolutionResult::NotFound(format!("未找到名为 '{}' 的小组", name)));
        }
        
        if groups.len() == 1 {
            return Ok(ResolutionResult::Single(groups[0].id.to_string()));
        }
        
        // 有多个同名小组，返回候选项
        let candidates: Vec<NameCandidate> = groups
            .iter()
            .map(|g| NameCandidate {
                id: g.id.to_string(),
                name: g.name.clone(),
                info: format!("班级: {} | 积分: {}", g.class_name, g.score),
            })
            .collect();
        
        Ok(ResolutionResult::Multiple(candidates))
    }
    
    /// 解析班级名称，返回班级ID
    pub async fn resolve_class(
        pool: &PgPool,
        name: &str,
    ) -> Result<ResolutionResult, AppError> {
        // 首先尝试直接作为UUID解析
        if let Ok(uuid) = Uuid::parse_str(name) {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM classes WHERE id = $1)"
            )
            .bind(uuid)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
            
            if exists {
                return Ok(ResolutionResult::Single(uuid.to_string()));
            }
        }
        
        // 按名称搜索班级
        let classes: Vec<ClassInfo> = sqlx::query_as(
            "SELECT id, name, grade FROM classes WHERE name = $1 OR name ILIKE $1 ORDER BY name"
        )
        .bind(name)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        
        if classes.is_empty() {
            return Ok(ResolutionResult::NotFound(format!("未找到名为 '{}' 的班级", name)));
        }
        
        if classes.len() == 1 {
            return Ok(ResolutionResult::Single(classes[0].id.to_string()));
        }
        
        // 有多个同名班级，返回候选项
        let candidates: Vec<NameCandidate> = classes
            .iter()
            .map(|c| NameCandidate {
                id: c.id.to_string(),
                name: c.name.clone(),
                info: format!("年级: {}", c.grade),
            })
            .collect();
        
        Ok(ResolutionResult::Multiple(candidates))
    }
    
    /// 解析部门名称，返回部门ID
    pub async fn resolve_department(
        pool: &PgPool,
        name: &str,
    ) -> Result<ResolutionResult, AppError> {
        // 首先尝试直接作为UUID解析
        if let Ok(uuid) = Uuid::parse_str(name) {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM departments WHERE id = $1)"
            )
            .bind(uuid)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
            
            if exists {
                return Ok(ResolutionResult::Single(uuid.to_string()));
            }
        }
        
        // 按名称搜索部门
        let departments: Vec<DepartmentInfo> = sqlx::query_as(
            "SELECT id, name, description FROM departments WHERE name = $1 OR name ILIKE $1 ORDER BY name"
        )
        .bind(name)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        
        if departments.is_empty() {
            return Ok(ResolutionResult::NotFound(format!("未找到名为 '{}' 的部门", name)));
        }
        
        if departments.len() == 1 {
            return Ok(ResolutionResult::Single(departments[0].id.to_string()));
        }
        
        // 有多个同名部门，返回候选项
        let candidates: Vec<NameCandidate> = departments
            .iter()
            .map(|d| NameCandidate {
                id: d.id.to_string(),
                name: d.name.clone(),
                info: d.description.clone().unwrap_or_else(|| "无描述".to_string()),
            })
            .collect();
        
        Ok(ResolutionResult::Multiple(candidates))
    }
}

/// 名称解析结果
pub enum ResolutionResult {
    /// 解析成功，返回单个ID
    Single(String),
    /// 有多个候选项，需要用户选择
    Multiple(Vec<NameCandidate>),
    /// 未找到
    NotFound(String),
}

/// 人员信息
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct PersonInfo {
    id: Uuid,
    name: String,
    gender: i16,
    number: String,
    belong_info: String,
    person_type: String,
}

/// 小组信息
#[derive(sqlx::FromRow)]
struct GroupInfo {
    id: Uuid,
    name: String,
    class_name: String,
    score: i32,
}

/// 班级信息
#[derive(sqlx::FromRow)]
struct ClassInfo {
    id: Uuid,
    name: String,
    grade: String,
}

/// 部门信息
#[derive(sqlx::FromRow)]
struct DepartmentInfo {
    id: Uuid,
    name: String,
    description: Option<String>,
}

// ========== 智能参数补全服务 ==========

pub struct ParamAutoCompleter;

impl ParamAutoCompleter {
    /// 自动补全日期参数
    /// 支持：今天、明天、昨天、本周、本月、YYYY-MM-DD
    pub fn complete_date(date_str: &str) -> Option<String> {
        let today = Local::now().date_naive();
        
        match date_str.trim() {
            "今天" | "今日" | "today" | "now" => {
                Some(today.format("%Y-%m-%d").to_string())
            }
            "明天" | "明日" | "tomorrow" => {
                Some((today + chrono::Duration::days(1)).format("%Y-%m-%d").to_string())
            }
            "昨天" | "昨日" | "yesterday" => {
                Some((today - chrono::Duration::days(1)).format("%Y-%m-%d").to_string())
            }
            s if s.parse::<NaiveDate>().is_ok() => {
                // 已经是标准格式
                Some(s.to_string())
            }
            _ => None
        }
    }
    
    /// 自动补全时间参数
    /// 支持：现在、上午8点、下午3点、8点30分、8点半、HH:MM格式
    pub fn complete_time(time_str: &str) -> Option<String> {
        let now = Local::now();
        let s = time_str.trim();
        
        // 处理"现在"、"当前"、"now"
        if s == "现在" || s == "当前" || s == "now" {
            return Some(now.format("%H:%M").to_string());
        }
        
        // 判断是上午还是下午
        let is_am = s.contains("上午") || s.contains("早上") || s.contains("早");
        let is_pm = s.contains("下午") || s.contains("晚上") || s.contains("晚");
        
        // 移除中文描述词，只保留数字和分隔符
        let cleaned = s.replace("上午", "")
            .replace("早上", "")
            .replace("早", "")
            .replace("下午", "")
            .replace("晚上", "")
            .replace("晚", "")
            .replace("点", ":")
            .replace("时", ":")
            .replace("分", "")
            .replace("半", "30")
            .replace("一刻", "15")
            .replace("三刻", "45");
        
        // 提取数字和冒号
        let hour;
        let mut minute = 0;
        
        // 查找第一个数字序列作为小时
        let hour_start = cleaned.find(|c: char| c.is_ascii_digit());
        if let Some(start) = hour_start {
            let hour_end = cleaned[start..].find(|c: char| !c.is_ascii_digit()).unwrap_or(cleaned.len() - start);
            let hour_str = &cleaned[start..start + hour_end];
            hour = hour_str.parse().unwrap_or(0);
            
            // 查找冒号后面的分钟
            if let Some(colon_pos) = cleaned.find(':') {
                let minute_start = colon_pos + 1;
                if minute_start < cleaned.len() {
                    let minute_end = cleaned[minute_start..].find(|c: char| !c.is_ascii_digit()).unwrap_or(cleaned.len() - minute_start);
                    let minute_str = &cleaned[minute_start..minute_start + minute_end];
                    minute = minute_str.parse().unwrap_or(0);
                }
            }
        } else {
            // 没有找到数字，尝试直接解析
            if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%H:%M") {
                return Some(naive_time.format("%H:%M").to_string());
            }
            if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%H:%M:%S") {
                return Some(naive_time.format("%H:%M").to_string());
            }
            return None;
        }
        
        // 处理12小时制转换
        let mut final_hour = hour;
        if is_pm && hour < 12 {
            final_hour = hour + 12;
        } else if is_am && hour == 12 {
            final_hour = 0; // 上午12点就是0点
        }
        
        // 处理特殊情况：下午12点应该是12点，不是24点
        if is_pm && hour == 12 {
            final_hour = 12;
        }
        
        Some(format!("{:02}:{:02}", final_hour, minute))
    }
    
    /// 自动补全考勤状态
    pub fn complete_attendance_status(status: &str) -> Option<String> {
        let status_map = [
            ("出勤", "present"),
            ("正常", "present"),
            ("到", "present"),
            ("迟到", "late"),
            ("晚", "late"),
            ("缺勤", "absent"),
            ("缺", "absent"),
            ("旷课", "absent"),
            ("早退", "early_leave"),
            ("请假", "excused"),
            ("病假", "excused"),
            ("事假", "excused"),
        ];
        
        for (key, value) in &status_map {
            if status.contains(key) {
                return Some(value.to_string());
            }
        }
        
        // 检查是否已经是英文状态
        let valid_statuses = ["present", "late", "absent", "early_leave", "excused"];
        if valid_statuses.contains(&status) {
            return Some(status.to_string());
        }
        
        None
    }
}

// ========== 权限检查函数 ==========

/// 检查用户是否有指定权限
#[allow(dead_code)]
async fn check_permission(
    pool: &PgPool,
    user_id: Uuid,
    permission: &str,
) -> Result<bool, AppError> {
    let permission_manager = PermissionManager::new(pool.clone());
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;
    
    // 检查具体权限或通配符权限
    let has_permission = user_permissions.iter().any(|p| {
        p == permission || 
        p == &format!("{}.*", permission.split('.').next().unwrap_or("")) ||
        p == "*"
    });
    
    Ok(has_permission)
}

/// 获取用户权限列表
async fn get_user_permissions(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<String>, AppError> {
    let permission_manager = PermissionManager::new(pool.clone());
    permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)
}

// ========== 操作执行器 ==========

pub struct AIActionExecutor;

impl AIActionExecutor {
    const MAX_BATCH_ITEMS: usize = 100;
    const MAX_ACTION_TYPE_LEN: usize = 64;
    const MAX_REASON_LEN: usize = 300;
    const MAX_PARAM_JSON_BYTES: usize = 64 * 1024;
    const MAX_NOTICE_TITLE_LEN: usize = 120;
    const MAX_NOTICE_CONTENT_LEN: usize = 4000;
    const MAX_GROUP_NAME_LEN: usize = 64;
    const MAX_GROUP_DESCRIPTION_LEN: usize = 500;
    const MAX_PERSON_NAME_LEN: usize = 64;
    const MAX_REMARK_LEN: usize = 500;

    fn invalid_input_response(user_permissions: &[String], message: impl Into<String>) -> AIActionResponse {
        AIActionResponse {
            success: false,
            message: message.into(),
            data: None,
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        }
    }

    fn exceeds_char_limit(value: &str, max_len: usize) -> bool {
        value.chars().count() > max_len
    }

    fn audit_action(
        user_id: Uuid,
        action_type: &str,
        batch: bool,
        items_len: usize,
        result: &Result<AIActionResponse, AppError>,
    ) {
        match result {
            Ok(resp) => info!(
                target: "ai_action_audit",
                user_id = %user_id,
                action_type = %action_type,
                batch,
                items_len,
                success = resp.success,
                message = %resp.message,
                "ai action executed"
            ),
            Err(err) => warn!(
                target: "ai_action_audit",
                user_id = %user_id,
                action_type = %action_type,
                batch,
                items_len,
                error = %err,
                "ai action failed"
            ),
        }
    }

    fn normalize_create_person_value(value: &serde_json::Value) -> serde_json::Value {
        let mut normalized = match value {
            serde_json::Value::Object(map) => map.clone(),
            _ => return value.clone(),
        };

        // 兼容部分模型返回的嵌套 params 结构
        let nested_params = normalized
            .get("params")
            .and_then(|v| v.as_object())
            .cloned();

        if let Some(inner) = nested_params {
            for (k, v) in inner {
                if normalized.get(k.as_str()).is_none() {
                    normalized.insert(k, v);
                }
            }
        }

        // 统一人员类型字段
        if !normalized.contains_key("person_type") {
            for alias in ["type", "personType", "person_kind", "role"] {
                if let Some(v) = normalized.get(alias).cloned() {
                    normalized.insert("person_type".to_string(), v);
                    break;
                }
            }
        }

        // 统一性别字段
        if !normalized.contains_key("gender") {
            if let Some(v) = normalized.get("sex").cloned() {
                normalized.insert("gender".to_string(), v);
            }
        }

        serde_json::Value::Object(normalized)
    }

    fn normalize_create_score_value(value: &serde_json::Value) -> serde_json::Value {
        let mut normalized = match value {
            serde_json::Value::Object(map) => map.clone(),
            _ => return value.clone(),
        };

        let nested_params = normalized
            .get("params")
            .and_then(|v| v.as_object())
            .cloned();

        if let Some(inner) = nested_params {
            for (k, v) in inner {
                if normalized.get(k.as_str()).is_none() {
                    normalized.insert(k, v);
                }
            }
        }

        if !normalized.contains_key("student_id") {
            for alias in ["person_id", "person", "person_name", "name", "student"] {
                if let Some(v) = normalized.get(alias).cloned() {
                    normalized.insert("student_id".to_string(), v);
                    break;
                }
            }
        }

        if !normalized.contains_key("group_id") {
            for alias in ["group_name", "group"] {
                if let Some(v) = normalized.get(alias).cloned() {
                    normalized.insert("group_id".to_string(), v);
                    break;
                }
            }
        }

        serde_json::Value::Object(normalized)
    }

    fn canonical_person_type(raw: &str) -> Option<&'static str> {
        let normalized = raw.trim().to_lowercase();
        match normalized.as_str() {
            "student" | "学生" | "stu" => Some("student"),
            "teacher" | "教师" | "老师" => Some("teacher"),
            "parent" | "家长" | "guardian" => Some("parent"),
            _ => None,
        }
    }

    fn infer_person_type(params: &CreatePersonParams) -> Option<&'static str> {
        let has_student_hints = params.student_no.as_ref().is_some_and(|v| !v.trim().is_empty())
            || params.class_id.as_ref().is_some_and(|v| !v.trim().is_empty())
            || params.enrollment_date.as_ref().is_some_and(|v| !v.trim().is_empty());

        if has_student_hints {
            return Some("student");
        }

        let has_teacher_hints = params.employee_no.as_ref().is_some_and(|v| !v.trim().is_empty())
            || params.department_id.as_ref().is_some_and(|v| !v.trim().is_empty())
            || params.title.as_ref().is_some_and(|v| !v.trim().is_empty())
            || params.hire_date.as_ref().is_some_and(|v| !v.trim().is_empty());

        if has_teacher_hints {
            return Some("teacher");
        }

        None
    }

    fn normalize_person_type(params: &mut CreatePersonParams) {
        if let Some(canonical) = Self::canonical_person_type(&params.person_type) {
            params.person_type = canonical.to_string();
            return;
        }

        if params.person_type.trim().is_empty() {
            if let Some(inferred) = Self::infer_person_type(params) {
                params.person_type = inferred.to_string();
            }
        }
    }

    /// 执行AI请求的操作
    pub async fn execute(
        pool: &PgPool,
        action_req: &AIActionRequest,
        user_id: Uuid,
        _user_name: &str,
    ) -> Result<AIActionResponse, AppError> {
        // 获取用户权限
        let user_permissions = get_user_permissions(pool, user_id).await?;

        if action_req.action_type.trim().is_empty() {
            return Ok(Self::invalid_input_response(&user_permissions, "操作类型不能为空"));
        }
        if Self::exceeds_char_limit(&action_req.action_type, Self::MAX_ACTION_TYPE_LEN) {
            return Ok(Self::invalid_input_response(
                &user_permissions,
                format!("操作类型长度不能超过 {} 个字符", Self::MAX_ACTION_TYPE_LEN),
            ));
        }
        if Self::exceeds_char_limit(&action_req.reason, Self::MAX_REASON_LEN) {
            return Ok(Self::invalid_input_response(
                &user_permissions,
                format!("操作原因长度不能超过 {} 个字符", Self::MAX_REASON_LEN),
            ));
        }
        let params_size = serde_json::to_vec(&action_req.params).map_or(0, |v| v.len());
        if params_size > Self::MAX_PARAM_JSON_BYTES {
            return Ok(Self::invalid_input_response(
                &user_permissions,
                format!("参数体积过大，不能超过 {} 字节", Self::MAX_PARAM_JSON_BYTES),
            ));
        }
        
        // 根据操作类型执行相应操作
        let result = match action_req.action_type.as_str() {
            "create_notice" => {
                Self::execute_create_notice(pool, &action_req.params, user_id, &user_permissions).await
            }
            "create_group" => {
                Self::execute_create_group(pool, &action_req.params, &user_permissions).await
            }
            "update_group_score" => {
                Self::execute_update_group_score(pool, &action_req.params, user_id, &user_permissions).await
            }
            "add_group_member" => {
                Self::execute_add_group_member(pool, &action_req.params, &user_permissions).await
            }
            "remove_group_member" => {
                Self::execute_remove_group_member(pool, &action_req.params, &user_permissions).await
            }
            "create_attendance" => {
                Self::execute_create_attendance(pool, &action_req.params, &user_permissions).await
            }
            "create_score" => {
                let mut normalized = Self::normalize_create_score_value(&action_req.params);
                if normalized.get("student_id").is_none() && normalized.get("group_id").is_some() {
                    if let Some(map) = normalized.as_object_mut() {
                        let score_change = map
                            .get("score_change")
                            .and_then(|v| v.as_i64())
                            .or_else(|| map.get("value").and_then(|v| v.as_i64()))
                            .unwrap_or(0);

                        map.insert("score_change".to_string(), serde_json::json!(score_change));

                        let reason = map
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .filter(|v| !v.trim().is_empty())
                            .unwrap_or(action_req.reason.as_str());
                        map.insert("reason".to_string(), serde_json::json!(reason));
                    }
                    Self::execute_update_group_score(pool, &normalized, user_id, &user_permissions).await
                } else {
                    Self::execute_create_score(pool, &normalized, &user_permissions).await
                }
            }
            "create_attendances_batch" => {
                let items = if !action_req.items.is_empty() {
                    action_req.items.clone()
                } else {
                    action_req.params.get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                };
                Self::execute_create_attendances_batch(pool, &items, &user_permissions).await
            }
            "create_scores_batch" => {
                let items = if !action_req.items.is_empty() {
                    action_req.items.clone()
                } else {
                    action_req.params.get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                };
                Self::execute_create_scores_batch(pool, &items, &user_permissions).await
            }
            "create_person" => {
                Self::execute_create_person(pool, &action_req.params, &user_permissions).await
            }
            "create_persons_batch" => {
                let mut items = if !action_req.items.is_empty() {
                    action_req.items.clone()
                } else {
                    action_req.params.get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                };

                if let Some(base) = action_req.params.as_object() {
                    let mut base_defaults = base.clone();
                    base_defaults.remove("items");
                    base_defaults.remove("params");

                    if let Some(nested) = base.get("params").and_then(|v| v.as_object()) {
                        for (k, v) in nested {
                            base_defaults.entry(k.clone()).or_insert_with(|| v.clone());
                        }
                    }

                    for item in &mut items {
                        if let Some(item_obj) = item.as_object() {
                            let mut merged = base_defaults.clone();
                            for (k, v) in item_obj {
                                merged.insert(k.clone(), v.clone());
                            }
                            *item = serde_json::Value::Object(merged);
                        }
                    }
                }

                Self::execute_create_persons_batch(pool, &items, &user_permissions).await
            }
            _ => {
                Ok(AIActionResponse {
                    success: false,
                    message: format!("未知的操作类型: {}", action_req.action_type),
                    data: None,
                    user_permissions,
                    need_confirmation: false,
                    candidates: None,
                })
            }
        };

        Self::audit_action(
            user_id,
            &action_req.action_type,
            action_req.batch,
            action_req.items.len(),
            &result,
        );

        result
    }
    
    /// 执行创建公告操作
    async fn execute_create_notice(
        pool: &PgPool,
        params: &serde_json::Value,
        user_id: Uuid,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "notice.create" || p == "notice.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有创建公告的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 解析参数
        let notice_params: CreateNoticeParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 验证必填字段
        if notice_params.title.trim().is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "公告标题不能为空".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if Self::exceeds_char_limit(&notice_params.title, Self::MAX_NOTICE_TITLE_LEN) {
            return Ok(Self::invalid_input_response(
                user_permissions,
                format!("公告标题长度不能超过 {} 个字符", Self::MAX_NOTICE_TITLE_LEN),
            ));
        }
        
        if notice_params.content.trim().is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "公告内容不能为空".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if Self::exceeds_char_limit(&notice_params.content, Self::MAX_NOTICE_CONTENT_LEN) {
            return Ok(Self::invalid_input_response(
                user_permissions,
                format!("公告内容长度不能超过 {} 个字符", Self::MAX_NOTICE_CONTENT_LEN),
            ));
        }
        
        // 执行创建
        let target_id = notice_params.target_id.as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());
        
        let row = sqlx::query_as::<_, NoticeRow>(
            "INSERT INTO notices (title, content, author_id, target_type, target_id, is_important) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id, title, content, author_id, 
             (SELECT name FROM persons WHERE id = $3) as author_name, 
             target_type, target_id, is_important, created_at"
        )
        .bind(&notice_params.title)
        .bind(&notice_params.content)
        .bind(user_id)
        .bind(&notice_params.target_type)
        .bind(target_id)
        .bind(notice_params.is_important.unwrap_or(false))
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(AIActionResponse {
            success: true,
            message: format!("公告 '{}' 创建成功", notice_params.title),
            data: Some(serde_json::json!({
                "id": row.id.to_string(),
                "title": row.title,
                "author_name": row.author_name,
                "created_at": row.created_at.to_rfc3339(),
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
    
    /// 执行创建小组操作
    async fn execute_create_group(
        pool: &PgPool,
        params: &serde_json::Value,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "group.create" || p == "group.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有创建小组的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 解析参数
        let group_params: CreateGroupParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 验证必填字段
        if group_params.name.trim().is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "小组名称不能为空".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if Self::exceeds_char_limit(&group_params.name, Self::MAX_GROUP_NAME_LEN) {
            return Ok(Self::invalid_input_response(
                user_permissions,
                format!("小组名称长度不能超过 {} 个字符", Self::MAX_GROUP_NAME_LEN),
            ));
        }

        if group_params
            .description
            .as_ref()
            .is_some_and(|v| Self::exceeds_char_limit(v, Self::MAX_GROUP_DESCRIPTION_LEN))
        {
            return Ok(Self::invalid_input_response(
                user_permissions,
                format!("小组描述长度不能超过 {} 个字符", Self::MAX_GROUP_DESCRIPTION_LEN),
            ));
        }
        
        // 解析班级ID（支持名称或UUID）
        let class_id = match NameResolver::resolve_class(pool, &group_params.class_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的班级，请选择", group_params.class_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 执行创建
        let row = sqlx::query_as::<_, GroupRow>(
            "INSERT INTO class_groups (class_id, name, description) 
             VALUES ($1, $2, $3) 
             RETURNING id, class_id, name, description, score, 
             (SELECT name FROM classes WHERE id = $1) as class_name,
             created_at, updated_at"
        )
        .bind(class_id)
        .bind(&group_params.name)
        .bind(group_params.description.as_deref().unwrap_or(""))
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(AIActionResponse {
            success: true,
            message: format!("小组 '{}' 创建成功", group_params.name),
            data: Some(serde_json::json!({
                "id": row.id.to_string(),
                "name": row.name,
                "class_name": row.class_name,
                "created_at": row.created_at.to_rfc3339(),
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
    
    /// 执行更新小组积分操作
    async fn execute_update_group_score(
        pool: &PgPool,
        params: &serde_json::Value,
        user_id: Uuid,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "group.update.score" || p == "group.update" || p == "group.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有更新小组积分的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 解析参数
        let score_params: UpdateGroupScoreParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 解析小组ID（支持名称或UUID）
        let group_id = match NameResolver::resolve_group(pool, &score_params.group_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的小组，请选择", score_params.group_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 检查小组是否存在
        let group_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM class_groups WHERE id = $1)"
        )
        .bind(group_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        if !group_exists {
            return Ok(AIActionResponse {
                success: false,
                message: "小组不存在".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 执行积分更新
        let row = sqlx::query_as::<_, ScoreRecordRow>(
            "INSERT INTO group_score_records (group_id, score_change, reason, created_by) 
             VALUES ($1, $2, $3, $4) 
             RETURNING id, score_change as score, reason, created_at,
             (SELECT name FROM persons WHERE id = $4) as operator_name"
        )
        .bind(group_id)
        .bind(score_params.score_change)
        .bind(&score_params.reason)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        // 更新小组总积分
        sqlx::query(
            "UPDATE class_groups SET score = score + $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2"
        )
        .bind(score_params.score_change)
        .bind(group_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
        
        let action = if score_params.score_change >= 0 { "增加" } else { "扣除" };
        
        Ok(AIActionResponse {
            success: true,
            message: format!("{}积分 {} 分", action, score_params.score_change.abs()),
            data: Some(serde_json::json!({
                "record_id": row.id.to_string(),
                "score_change": score_params.score_change,
                "reason": score_params.reason,
                "created_at": row.created_at.to_rfc3339(),
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
    
    /// 执行添加小组成员操作
    async fn execute_add_group_member(
        pool: &PgPool,
        params: &serde_json::Value,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "group.update.member" || p == "group.update" || p == "group.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有管理小组成员的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 解析参数
        let member_params: AddGroupMemberParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 解析小组ID（支持名称或UUID）
        let group_id = match NameResolver::resolve_group(pool, &member_params.group_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的小组，请选择", member_params.group_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 解析人员ID（支持名称或UUID）
        let person_id = match NameResolver::resolve_person(pool, &member_params.person_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的人员，请选择", member_params.person_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 检查人员是否已在小组中
        let member_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND person_id = $2)"
        )
        .bind(group_id)
        .bind(person_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        if member_exists {
            return Ok(AIActionResponse {
                success: false,
                message: "该成员已在小组中".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 获取人员信息
        let person_info: (String, String) = sqlx::query_as(
            "SELECT p.name, COALESCE(s.student_no, t.employee_no, '-') as number 
             FROM persons p 
             LEFT JOIN students s ON p.id = s.person_id 
             LEFT JOIN teachers t ON p.id = t.person_id 
             WHERE p.id = $1"
        )
        .bind(person_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        // 执行添加
        sqlx::query(
            "INSERT INTO group_members (group_id, person_id) VALUES ($1, $2)"
        )
        .bind(group_id)
        .bind(person_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(AIActionResponse {
            success: true,
            message: format!("成功添加成员 '{}' 到小组", person_info.0),
            data: Some(serde_json::json!({
                "person_name": person_info.0,
                "person_number": person_info.1,
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
    
    /// 执行移除小组成员操作
    async fn execute_remove_group_member(
        pool: &PgPool,
        params: &serde_json::Value,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "group.update.member" || p == "group.update" || p == "group.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有管理小组成员的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 解析参数
        let member_params: RemoveGroupMemberParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 解析小组ID（支持名称或UUID）
        let group_id = match NameResolver::resolve_group(pool, &member_params.group_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的小组，请选择", member_params.group_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 解析人员ID（支持名称或UUID）
        let person_id = match NameResolver::resolve_person(pool, &member_params.person_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的人员，请选择", member_params.person_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 获取人员信息
        let person_info: Option<(String,)> = sqlx::query_as(
            "SELECT p.name FROM persons p 
             JOIN group_members gm ON p.id = gm.person_id 
             WHERE gm.group_id = $1 AND gm.person_id = $2"
        )
        .bind(group_id)
        .bind(person_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;
        
        let person_name = match person_info {
            Some(info) => info.0,
            None => {
                return Ok(AIActionResponse {
                    success: false,
                    message: "该成员不在小组中".to_string(),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 执行移除
        let result = sqlx::query(
            "DELETE FROM group_members WHERE group_id = $1 AND person_id = $2"
        )
        .bind(group_id)
        .bind(person_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
        
        if result.rows_affected() == 0 {
            return Ok(AIActionResponse {
                success: false,
                message: "移除成员失败".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        Ok(AIActionResponse {
            success: true,
            message: format!("成功将 '{}' 从小组移除", person_name),
            data: None,
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
    
    /// 执行创建考勤记录操作
    async fn execute_create_attendance(
        pool: &PgPool,
        params: &serde_json::Value,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "attendance.create" || p == "attendance.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有创建考勤记录的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 解析参数
        let attendance_params: CreateAttendanceParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };

        if attendance_params
            .remark
            .as_ref()
            .is_some_and(|v| Self::exceeds_char_limit(v, Self::MAX_REMARK_LEN))
        {
            return Ok(Self::invalid_input_response(
                user_permissions,
                format!("考勤备注长度不能超过 {} 个字符", Self::MAX_REMARK_LEN),
            ));
        }
        
        // 解析人员ID（支持名称或UUID）
        let person_id = match NameResolver::resolve_person(pool, &attendance_params.person_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的人员，请选择", attendance_params.person_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 自动补全日期
        let date = ParamAutoCompleter::complete_date(&attendance_params.date)
            .unwrap_or_else(|| attendance_params.date.clone());
        
        // 验证日期格式并解析为 NaiveDate
        let naive_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("无效的日期格式: {}，请使用 YYYY-MM-DD 格式或相对日期（今天、明天等）", date),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 自动补全考勤状态
        let status = ParamAutoCompleter::complete_attendance_status(&attendance_params.status)
            .unwrap_or_else(|| attendance_params.status.clone());
        
        // 验证考勤状态
        let valid_statuses = ["present", "late", "absent", "early_leave", "excused"];
        if !valid_statuses.contains(&status.as_str()) {
            return Ok(AIActionResponse {
                success: false,
                message: format!("无效的考勤状态: {}，可选值: 出勤(present)、迟到(late)、缺勤(absent)、早退(early_leave)、请假(excused)", status),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 自动补全时间并解析为 NaiveTime
        let naive_time = if let Some(time_str) = attendance_params.time.as_ref() {
            let time_str = ParamAutoCompleter::complete_time(time_str)
                .unwrap_or_else(|| time_str.clone());
            
            // 尝试解析时间格式 HH:MM 或 HH:MM:SS
            if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%H:%M") {
                Some(time)
            } else if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%H:%M:%S") {
                Some(time)
            } else {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("无效的时间格式: {}，请使用 HH:MM 格式或描述（上午8点、下午3点、现在）", time_str),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        } else {
            None
        };
        
        // 获取人员信息
        let person_info: (String,) = sqlx::query_as(
            "SELECT name FROM persons WHERE id = $1"
        )
        .bind(person_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound)?;
        
        // 执行创建
        let row = sqlx::query_as::<_, AttendanceRow>(
            "INSERT INTO attendances (person_id, date, status, time, remark) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id, person_id, (SELECT name FROM persons WHERE id = $1) as person_name,
             date, status, time, remark, created_at"
        )
        .bind(person_id)
        .bind(naive_date)
        .bind(&status)
        .bind(naive_time)
        .bind(attendance_params.remark.as_deref())
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(AIActionResponse {
            success: true,
            message: format!("为 '{}' 创建考勤记录成功", person_info.0),
            data: Some(serde_json::json!({
                "id": row.id.to_string(),
                "person_name": row.person_name,
                "date": row.date,
                "status": row.status,
                "time": row.time,
                "created_at": row.created_at.to_rfc3339(),
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
    
    /// 执行创建成绩记录操作
    async fn execute_create_score(
        pool: &PgPool,
        params: &serde_json::Value,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "score.create" || p == "score.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有添加个人积分的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        // 解析参数
        let score_params: CreateScoreParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        
        // 解析学生ID（支持名称或UUID）
        let student_id = match NameResolver::resolve_person(pool, &score_params.student_id).await? {
            ResolutionResult::Single(id) => Uuid::parse_str(&id).unwrap(),
            ResolutionResult::Multiple(candidates) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("找到多个名为 '{}' 的学生，请选择", score_params.student_id),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: true,
                    candidates: Some(candidates),
                });
            }
            ResolutionResult::NotFound(msg) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: msg,
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };
        

        
        // 获取人员信息
        let person_info: (String,) = sqlx::query_as(
            "SELECT name FROM persons WHERE id = $1"
        )
        .bind(student_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound)?;
        
        // 执行创建
        let row = sqlx::query_as::<_, ScoreRow>(
            "INSERT INTO scores (person_id, score_type, value, reason, event_id, created_by) 
             VALUES ($1, $2, $3, $4, NULL, NULL) 
             RETURNING id, person_id, 
             (SELECT name FROM persons WHERE id = $1) as person_name,
             score_type, value, reason, created_at"
        )
        .bind(student_id)
        .bind("personal")  // score_type
        .bind(score_params.value)  // value
        .bind(&score_params.reason)  // reason
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(AIActionResponse {
            success: true,
            message: format!("为 '{}' 添加个人积分 {} 分成功，原因：{}", person_info.0, score_params.value, score_params.reason),
            data: Some(serde_json::json!({
                "id": row.id.to_string(),
                "person_name": row.person_name,
                "value": row.value,
                "reason": row.reason,
                "score_type": row.score_type,
                "created_at": row.created_at.to_rfc3339(),
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }

    /// 执行创建人员操作
    async fn execute_create_person(
        pool: &PgPool,
        params: &serde_json::Value,
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "person.create" || p == "person.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有创建人员的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        // 解析参数
        let normalized_params = Self::normalize_create_person_value(params);
        let mut person_params: CreatePersonParams = match serde_json::from_value(normalized_params) {
            Ok(p) => p,
            Err(e) => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("参数解析失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        };

        Self::normalize_person_type(&mut person_params);

        // 验证必填字段
        if person_params.name.trim().is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "人员姓名不能为空".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if Self::exceeds_char_limit(&person_params.name, Self::MAX_PERSON_NAME_LEN) {
            return Ok(Self::invalid_input_response(
                user_permissions,
                format!("人员姓名长度不能超过 {} 个字符", Self::MAX_PERSON_NAME_LEN),
            ));
        }

        if person_params.person_type.trim().is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "缺少人员类型，请提供 type 或 person_type（student/teacher/parent）".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        // 根据人员类型验证必填字段
        match person_params.person_type.as_str() {
            "student" => {
                if person_params.student_no.is_none() || person_params.student_no.as_ref().unwrap().trim().is_empty() {
                    return Ok(AIActionResponse {
                        success: false,
                        message: "创建学生时必须提供学号(student_no)".to_string(),
                        data: None,
                        user_permissions: user_permissions.to_vec(),
                        need_confirmation: false,
                        candidates: None,
                    });
                }
            }
            "teacher" => {
                if person_params.employee_no.is_none() || person_params.employee_no.as_ref().unwrap().trim().is_empty() {
                    return Ok(AIActionResponse {
                        success: false,
                        message: "创建教师时必须提供工号(employee_no)".to_string(),
                        data: None,
                        user_permissions: user_permissions.to_vec(),
                        need_confirmation: false,
                        candidates: None,
                    });
                }
            }
            "parent" => {
                // 家长没有特殊必填字段
            }
            _ => {
                return Ok(AIActionResponse {
                    success: false,
                    message: format!("未知的人员类型: {}，必须是 student、teacher 或 parent", person_params.person_type),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                });
            }
        }

        // 执行创建
        match Self::create_person_internal(pool, &person_params).await {
            Ok(person_id) => {
                Ok(AIActionResponse {
                    success: true,
                    message: format!("成功创建{} '{}'", 
                        match person_params.person_type.as_str() {
                            "student" => "学生",
                            "teacher" => "教师",
                            "parent" => "家长",
                            _ => "人员"
                        },
                        person_params.name
                    ),
                    data: Some(serde_json::json!({
                        "id": person_id.to_string(),
                        "name": person_params.name,
                        "person_type": person_params.person_type,
                    })),
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                })
            }
            Err(e) => {
                Ok(AIActionResponse {
                    success: false,
                    message: format!("创建人员失败: {}", e),
                    data: None,
                    user_permissions: user_permissions.to_vec(),
                    need_confirmation: false,
                    candidates: None,
                })
            }
        }
    }

    /// 内部方法：创建人员
    async fn create_person_internal(
        pool: &PgPool,
        params: &CreatePersonParams,
    ) -> Result<Uuid, AppError> {
        use crate::core::password::hash_password;

        let mut tx = pool.begin().await?;
        let person_id = Uuid::new_v4();

        // 解析日期
        let birthday = params.birthday.as_ref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let enrollment_date = params.enrollment_date.as_ref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let hire_date = params.hire_date.as_ref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

        // 根据人员类型确定username
        let username = match params.person_type.as_str() {
            "student" => {
                params.student_no.as_ref().cloned().unwrap_or_default()
            }
            "teacher" => {
                params.employee_no.as_ref().cloned().unwrap_or_default()
            }
            "parent" => {
                params.phone.clone().unwrap_or_else(|| person_id.to_string())
            }
            _ => person_id.to_string(),
        };

        // 生成密码哈希（默认密码123456）
        let password_hash = hash_password("123456")
            .map_err(|_| AppError::Internal)?;

        // 插入人员基本信息
        sqlx::query(
            "INSERT INTO persons (id, name, username, password_hash, gender, birthday, phone, email, type) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(person_id)
        .bind(&params.name)
        .bind(&username)
        .bind(&password_hash)
        .bind(params.gender)
        .bind(birthday)
        .bind(&params.phone)
        .bind(&params.email)
        .bind(&params.person_type)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        // 根据人员类型插入扩展信息
        match params.person_type.as_str() {
            "student" => {
                let student_no = params.student_no.as_ref().cloned().unwrap_or_default();
                
                // 解析班级ID（支持名称或UUID）
                let class_id = if let Some(class_id_str) = &params.class_id {
                    match NameResolver::resolve_class(pool, class_id_str).await? {
                        ResolutionResult::Single(id) => Some(Uuid::parse_str(&id).unwrap()),
                        ResolutionResult::Multiple(_) => None,
                        ResolutionResult::NotFound(_) => None,
                    }
                } else {
                    None
                };

                sqlx::query(
                    "INSERT INTO students (person_id, student_no, class_id, enrollment_date, status)
                     VALUES ($1, $2, $3, $4, 'enrolled')"
                )
                .bind(person_id)
                .bind(student_no)
                .bind(class_id)
                .bind(enrollment_date)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Database)?;
            }
            "teacher" => {
                let employee_no = params.employee_no.as_ref().cloned().unwrap_or_default();
                
                // 解析部门ID（支持名称或UUID）
                let department_id = if let Some(dept_id_str) = &params.department_id {
                    match NameResolver::resolve_department(pool, dept_id_str).await? {
                        ResolutionResult::Single(id) => Some(Uuid::parse_str(&id).unwrap()),
                        ResolutionResult::Multiple(_) => None,
                        ResolutionResult::NotFound(_) => None,
                    }
                } else {
                    None
                };

                sqlx::query(
                    "INSERT INTO teachers (person_id, employee_no, department_id, title, hire_date)
                     VALUES ($1, $2, $3, $4, $5)"
                )
                .bind(person_id)
                .bind(employee_no)
                .bind(department_id)
                .bind(&params.title)
                .bind(hire_date)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Database)?;
            }
            "parent" => {
                sqlx::query(
                    "INSERT INTO parents (person_id, occupation, address)
                     VALUES ($1, NULL, NULL)"
                )
                .bind(person_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Database)?;
            }
            _ => {}
        }

        tx.commit().await.map_err(AppError::Database)?;
        Ok(person_id)
    }

    /// 执行批量创建人员操作
    async fn execute_create_persons_batch(
        pool: &PgPool,
        items: &[serde_json::Value],
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "person.create" || p == "person.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有创建人员的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if items.is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "批量创建人员失败: items 不能为空".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if items.len() > Self::MAX_BATCH_ITEMS {
            return Ok(AIActionResponse {
                success: false,
                message: format!("批量创建人员失败: 单次最多支持 {} 条", Self::MAX_BATCH_ITEMS),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        let mut success_count = 0;
        let mut failure_count = 0;
        let mut item_results = Vec::new();

        for (index, item) in items.iter().enumerate() {
            let normalized_item = Self::normalize_create_person_value(item);
            let mut person_params: CreatePersonParams = match serde_json::from_value(normalized_item) {
                Ok(p) => p,
                Err(e) => {
                    failure_count += 1;
                    item_results.push(BatchItemResult {
                        success: false,
                        index,
                        data: None,
                        error: Some(format!("参数解析失败: {}", e)),
                    });
                    continue;
                }
            };

            Self::normalize_person_type(&mut person_params);

            if person_params.person_type.trim().is_empty() {
                failure_count += 1;
                item_results.push(BatchItemResult {
                    success: false,
                    index,
                    data: None,
                    error: Some("缺少人员类型，请提供 type 或 person_type（student/teacher/parent）".to_string()),
                });
                continue;
            }

            match Self::create_person_internal(pool, &person_params).await {
                Ok(person_id) => {
                    success_count += 1;
                    item_results.push(BatchItemResult {
                        success: true,
                        index,
                        data: Some(serde_json::json!({
                            "id": person_id.to_string(),
                            "name": person_params.name,
                        })),
                        error: None,
                    });
                }
                Err(e) => {
                    failure_count += 1;
                    item_results.push(BatchItemResult {
                        success: false,
                        index,
                        data: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        let total = items.len();
        Ok(AIActionResponse {
            success: success_count > 0,
            message: format!("批量创建人员完成: 成功 {} 个, 失败 {} 个, 总计 {} 个", 
                success_count, failure_count, total),
            data: Some(serde_json::json!({
                "batch_result": {
                    "total": total,
                    "success_count": success_count,
                    "failure_count": failure_count,
                    "items": item_results,
                }
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
}

// ========== 数据库行结构 ==========

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct NoticeRow {
    id: Uuid,
    title: String,
    content: String,
    author_id: Uuid,
    author_name: String,
    target_type: String,
    target_id: Option<Uuid>,
    is_important: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct GroupRow {
    id: Uuid,
    class_id: Uuid,
    name: String,
    description: Option<String>,
    score: i32,
    class_name: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct ScoreRecordRow {
    id: Uuid,
    score: i32,
    reason: String,
    created_at: chrono::DateTime<chrono::Utc>,
    operator_name: Option<String>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct AttendanceRow {
    id: Uuid,
    person_id: Uuid,
    person_name: String,
    date: NaiveDate,
    status: String,
    time: Option<NaiveTime>,
    remark: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct ScoreRow {
    id: Uuid,
    person_id: Uuid,
    person_name: String,
    score_type: String,
    value: i32,
    reason: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ========== API 处理函数 ==========

/// AI 操作执行 API
pub async fn execute_action(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AIActionRequest>,
) -> Result<Json<AIActionResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    
    // 获取用户名称
    let user_name: String = sqlx::query_scalar(
        "SELECT name FROM persons WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| AppError::NotFound)?;
    
    // 执行操作
    let response = AIActionExecutor::execute(&pool, &req, user_id, &user_name).await?;
    
    Ok(Json(response))
}

/// 获取用户可用操作列表 API
pub async fn get_available_actions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    
    let user_permissions = get_user_permissions(&pool, user_id).await?;
    
    // 根据权限构建可用操作列表
    let mut available_actions = vec![];
    
    // 公告相关操作
    if user_permissions.iter().any(|p| p == "notice.create" || p == "notice.*") {
        available_actions.push(serde_json::json!({
            "action_type": "create_notice",
            "name": "创建公告",
            "description": "创建新的学校公告",
            "required_params": ["title", "content", "target_type"],
            "optional_params": ["target_id", "is_important"]
        }));
    }
    
    // 小组相关操作
    if user_permissions.iter().any(|p| p == "group.create" || p == "group.*") {
        available_actions.push(serde_json::json!({
            "action_type": "create_group",
            "name": "创建小组",
            "description": "在指定班级创建新的小组",
            "required_params": ["class_id", "name"],
            "optional_params": ["description"],
            "param_tips": {
                "class_id": "可以使用班级名称或UUID"
            }
        }));
    }
    
    if user_permissions.iter().any(|p| p == "group.update.score" || p == "group.update" || p == "group.*") {
        available_actions.push(serde_json::json!({
            "action_type": "update_group_score",
            "name": "更新小组积分",
            "description": "增加或减少小组积分",
            "required_params": ["group_id", "score_change", "reason"],
            "optional_params": [],
            "param_tips": {
                "group_id": "可以使用小组名称或UUID"
            }
        }));
    }
    
    if user_permissions.iter().any(|p| p == "group.update.member" || p == "group.update" || p == "group.*") {
        available_actions.push(serde_json::json!({
            "action_type": "add_group_member",
            "name": "添加小组成员",
            "description": "向小组添加新成员",
            "required_params": ["group_id", "person_id"],
            "optional_params": [],
            "param_tips": {
                "group_id": "可以使用小组名称或UUID",
                "person_id": "可以使用姓名、学号、工号、用户名、手机号或UUID"
            }
        }));
        available_actions.push(serde_json::json!({
            "action_type": "remove_group_member",
            "name": "移除小组成员",
            "description": "从小组移除成员",
            "required_params": ["group_id", "person_id"],
            "optional_params": [],
            "param_tips": {
                "group_id": "可以使用小组名称或UUID",
                "person_id": "可以使用姓名、学号、工号、用户名、手机号或UUID"
            }
        }));
    }
    
    // 考勤相关操作
    if user_permissions.iter().any(|p| p == "attendance.create" || p == "attendance.*") {
        available_actions.push(serde_json::json!({
            "action_type": "create_attendance",
            "name": "创建考勤记录",
            "description": "为人员创建考勤记录",
            "required_params": ["person_id", "date", "status"],
            "optional_params": ["time", "remark"],
            "param_tips": {
                "person_id": "可以使用姓名、学号、工号、用户名、手机号或UUID",
                "date": "支持日期格式(YYYY-MM-DD)或相对日期(今天、明天、昨天)",
                "status": "支持中文(出勤、迟到、缺勤、早退、请假)或英文(present、late、absent、early_leave、excused)",
                "time": "支持时间格式(HH:MM)或描述(上午8点、下午3点、现在)"
            }
        }));
    }
    
    // 个人积分相关操作
    if user_permissions.iter().any(|p| p == "score.create" || p == "score.*") {
        available_actions.push(serde_json::json!({
            "action_type": "create_score",
            "name": "添加个人积分",
            "description": "为人员添加个人表现积分",
            "required_params": ["student_id", "reason", "value"],
            "optional_params": [],
            "param_tips": {
                "student_id": "可以使用姓名、学号、工号、用户名、手机号或UUID",
                "value": "积分值（整数），可正可负"
            }
        }));
    }
    
    // 批量考勤相关操作
    if user_permissions.iter().any(|p| p == "attendance.create" || p == "attendance.*") {
        available_actions.push(serde_json::json!({
            "action_type": "create_attendances_batch",
            "name": "批量创建考勤记录",
            "description": "为多人批量创建考勤记录",
            "required_params": ["items"],
            "optional_params": [],
            "param_tips": {
                "items": "考勤记录数组，每个记录包含person_id, date, status等"
            }
        }));
    }
    
    // 批量积分相关操作
    if user_permissions.iter().any(|p| p == "score.create" || p == "score.*") {
        available_actions.push(serde_json::json!({
            "action_type": "create_scores_batch",
            "name": "批量添加个人积分",
            "description": "为多人批量添加个人表现积分",
            "required_params": ["items"],
            "optional_params": [],
            "param_tips": {
                "items": "积分记录数组，每个记录包含student_id, reason, value等"
            }
        }));
    }
    
    Ok(Json(serde_json::json!({
        "available_actions": available_actions,
        "user_permissions": user_permissions,
    })))
}

// ========== 批量操作执行函数 ==========

impl AIActionExecutor {
    /// 执行批量创建考勤记录
    pub async fn execute_create_attendances_batch(
        pool: &PgPool,
        items: &[serde_json::Value],
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "attendance.create" || p == "attendance.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有创建考勤记录的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if items.is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "批量创建考勤记录失败: items 不能为空".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if items.len() > Self::MAX_BATCH_ITEMS {
            return Ok(AIActionResponse {
                success: false,
                message: format!("批量创建考勤记录失败: 单次最多支持 {} 条", Self::MAX_BATCH_ITEMS),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        let mut batch_results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for (index, item) in items.iter().enumerate() {
            // 解析参数
            let person_id_str = item.get("person_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            // 解析人员ID
            let person_id = match NameResolver::resolve_person(pool, person_id_str).await {
                Ok(ResolutionResult::Single(id)) => match Uuid::parse_str(&id) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        batch_results.push(serde_json::json!({
                            "success": false,
                            "index": index,
                            "error": format!("无效的人员ID: {}", person_id_str)
                        }));
                        failure_count += 1;
                        continue;
                    }
                },
                Ok(ResolutionResult::Multiple(_)) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": format!("找到多个名为 '{}' 的人员", person_id_str)
                    }));
                    failure_count += 1;
                    continue;
                }
                Ok(ResolutionResult::NotFound(msg)) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": msg
                    }));
                    failure_count += 1;
                    continue;
                }
                Err(e) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": e.to_string()
                    }));
                    failure_count += 1;
                    continue;
                }
            };
            
            // 解析日期
            let date_str = item.get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let date = ParamAutoCompleter::complete_date(date_str)
                .unwrap_or_else(|| date_str.to_string());
            
            let naive_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": format!("无效的日期格式: {}", date)
                    }));
                    failure_count += 1;
                    continue;
                }
            };
            
            // 解析状态
            let status_str = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let status = ParamAutoCompleter::complete_attendance_status(status_str)
                .unwrap_or_else(|| status_str.to_string());
            
            let valid_statuses = ["present", "late", "absent", "early_leave", "excused"];
            if !valid_statuses.contains(&status.as_str()) {
                batch_results.push(serde_json::json!({
                    "success": false,
                    "index": index,
                    "error": format!("无效的考勤状态: {}", status)
                }));
                failure_count += 1;
                continue;
            }
            
            // 解析时间
            let naive_time = if let Some(time_str) = item.get("time").and_then(|v| v.as_str()) {
                let time_str = ParamAutoCompleter::complete_time(time_str)
                    .unwrap_or_else(|| time_str.to_string());
                
                if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%H:%M") {
                    Some(time)
                } else {
                    NaiveTime::parse_from_str(&time_str, "%H:%M:%S").ok()
                }
            } else {
                None
            };
            
            let remark = item.get("remark").and_then(|v| v.as_str());
            
            // 执行创建
            match sqlx::query_as::<_, AttendanceRow>(
                "INSERT INTO attendances (person_id, date, status, time, remark) 
                 VALUES ($1, $2, $3, $4, $5) 
                 RETURNING id, person_id, (SELECT name FROM persons WHERE id = $1) as person_name,
                 date, status, time, remark, created_at"
            )
            .bind(person_id)
            .bind(naive_date)
            .bind(&status)
            .bind(naive_time)
            .bind(remark)
            .fetch_one(pool)
            .await
            {
                Ok(row) => {
                    batch_results.push(serde_json::json!({
                        "success": true,
                        "index": index,
                        "data": {
                            "id": row.id.to_string(),
                            "person_name": row.person_name,
                            "date": row.date,
                            "status": row.status,
                        }
                    }));
                    success_count += 1;
                }
                Err(e) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": e.to_string()
                    }));
                    failure_count += 1;
                }
            }
        }
        
        Ok(AIActionResponse {
            success: success_count > 0,
            message: format!("批量创建完成，成功{}个，失败{}个", success_count, failure_count),
            data: Some(serde_json::json!({
                "total": items.len(),
                "success_count": success_count,
                "failure_count": failure_count,
                "items": batch_results
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
    
    /// 执行批量创建成绩记录
    pub async fn execute_create_scores_batch(
        pool: &PgPool,
        items: &[serde_json::Value],
        user_permissions: &[String],
    ) -> Result<AIActionResponse, AppError> {
        // 检查权限
        if !user_permissions.iter().any(|p| p == "score.create" || p == "score.*") {
            return Ok(AIActionResponse {
                success: false,
                message: "没有添加个人积分的权限".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if items.is_empty() {
            return Ok(AIActionResponse {
                success: false,
                message: "批量添加积分失败: items 不能为空".to_string(),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }

        if items.len() > Self::MAX_BATCH_ITEMS {
            return Ok(AIActionResponse {
                success: false,
                message: format!("批量添加积分失败: 单次最多支持 {} 条", Self::MAX_BATCH_ITEMS),
                data: None,
                user_permissions: user_permissions.to_vec(),
                need_confirmation: false,
                candidates: None,
            });
        }
        
        let mut batch_results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for (index, item) in items.iter().enumerate() {
            // 解析学生ID
            let student_id_str = item.get("student_id")
                .or_else(|| item.get("person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let student_id = match NameResolver::resolve_person(pool, student_id_str).await {
                Ok(ResolutionResult::Single(id)) => match Uuid::parse_str(&id) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        batch_results.push(serde_json::json!({
                            "success": false,
                            "index": index,
                            "error": format!("无效的学生ID: {}", student_id_str)
                        }));
                        failure_count += 1;
                        continue;
                    }
                },
                Ok(ResolutionResult::Multiple(_)) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": format!("找到多个名为 '{}' 的学生", student_id_str)
                    }));
                    failure_count += 1;
                    continue;
                }
                Ok(ResolutionResult::NotFound(msg)) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": msg
                    }));
                    failure_count += 1;
                    continue;
                }
                Err(e) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": e.to_string()
                    }));
                    failure_count += 1;
                    continue;
                }
            };
            
            // 解析原因
            let reason = item.get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if reason.is_empty() {
                batch_results.push(serde_json::json!({
                    "success": false,
                    "index": index,
                    "error": "评分原因不能为空"
                }));
                failure_count += 1;
                continue;
            }
            
            // 解析积分值
            let value = item.get("value")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            
            // 执行创建
            match sqlx::query_as::<_, ScoreRow>(
                "INSERT INTO scores (person_id, score_type, value, reason, event_id, created_by) 
                 VALUES ($1, $2, $3, $4, NULL, NULL) 
                 RETURNING id, person_id, 
                 (SELECT name FROM persons WHERE id = $1) as person_name,
                 score_type, value, reason, created_at"
            )
            .bind(student_id)
            .bind("personal")
            .bind(value)
            .bind(reason)
            .fetch_one(pool)
            .await
            {
                Ok(row) => {
                    batch_results.push(serde_json::json!({
                        "success": true,
                        "index": index,
                        "data": {
                            "id": row.id.to_string(),
                            "person_name": row.person_name,
                            "value": row.value,
                            "reason": row.reason,
                        }
                    }));
                    success_count += 1;
                }
                Err(e) => {
                    batch_results.push(serde_json::json!({
                        "success": false,
                        "index": index,
                        "error": e.to_string()
                    }));
                    failure_count += 1;
                }
            }
        }
        
        Ok(AIActionResponse {
            success: success_count > 0,
            message: format!("批量添加完成，成功{}个，失败{}个", success_count, failure_count),
            data: Some(serde_json::json!({
                "total": items.len(),
                "success_count": success_count,
                "failure_count": failure_count,
                "items": batch_results
            })),
            user_permissions: user_permissions.to_vec(),
            need_confirmation: false,
            candidates: None,
        })
    }
}
