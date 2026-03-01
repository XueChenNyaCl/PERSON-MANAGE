use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Local, NaiveDate, NaiveTime};

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
    pub params: serde_json::Value,
    /// 操作原因/说明
    pub reason: String,
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
    pub student_id: String,
    pub reason: String,
    pub value: i32,
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
        // 首先尝试直接作为UUID解析
        if let Ok(uuid) = Uuid::parse_str(name) {
            // 检查该UUID是否存在
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM persons WHERE id = $1)"
            )
            .bind(uuid)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e))?;
            
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
            WHERE p.name = $1 OR p.name ILIKE $1
            ORDER BY p.name"#,
        )
        .bind(name)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        if persons.is_empty() {
            return Ok(ResolutionResult::NotFound(format!("未找到名为 '{}' 的人员", name)));
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
            .map_err(|e| AppError::Database(e))?;
            
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
        .map_err(|e| AppError::Database(e))?;
        
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
            .map_err(|e| AppError::Database(e))?;
            
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
        .map_err(|e| AppError::Database(e))?;
        
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
        let mut hour = 0;
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
    /// 执行AI请求的操作
    pub async fn execute(
        pool: &PgPool,
        action_req: &AIActionRequest,
        user_id: Uuid,
        user_name: &str,
    ) -> Result<AIActionResponse, AppError> {
        // 获取用户权限
        let user_permissions = get_user_permissions(pool, user_id).await?;
        
        // 根据操作类型执行相应操作
        match action_req.action_type.as_str() {
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
                Self::execute_create_score(pool, &action_req.params, &user_permissions).await
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
        }
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
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
        // 更新小组总积分
        sqlx::query(
            "UPDATE class_groups SET score = score + $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2"
        )
        .bind(score_params.score_change)
        .bind(group_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
        // 执行添加
        sqlx::query(
            "INSERT INTO group_members (group_id, person_id) VALUES ($1, $2)"
        )
        .bind(group_id)
        .bind(person_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
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
        let valid_statuses = vec!["present", "late", "absent", "early_leave", "excused"];
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
        .map_err(|e| AppError::Database(e))?;
        
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
        .map_err(|e| AppError::Database(e))?;
        
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
}

// ========== 数据库行结构 ==========

#[derive(sqlx::FromRow)]
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
struct ScoreRecordRow {
    id: Uuid,
    score: i32,
    reason: String,
    created_at: chrono::DateTime<chrono::Utc>,
    operator_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct AttendanceRow {
    id: Uuid,
    person_id: Uuid,
    person_name: String,
    date: String,
    status: String,
    time: Option<String>,
    remark: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
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
                "person_id": "可以使用人员姓名或UUID"
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
                "person_id": "可以使用人员姓名或UUID"
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
                "person_id": "可以使用人员姓名或UUID",
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
                "student_id": "可以使用人员姓名或UUID",
                "value": "积分值（整数），可正可负"
            }
        }));
    }
    
    Ok(Json(serde_json::json!({
        "available_actions": available_actions,
        "user_permissions": user_permissions,
    })))
}
