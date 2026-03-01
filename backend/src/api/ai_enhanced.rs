use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use reqwest::Client;
use regex::Regex;

use crate::api::routes::AppState;
use crate::api::ai_actions::{AIActionRequest, AIActionExecutor, NameResolver, ResolutionResult};
use crate::api::ai_data::{
    ClassDataService, GroupDataService, DepartmentDataService,
    MarkdownFormatter
};
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;

// ========== AI 增强聊天请求/响应 ==========

#[derive(Debug, Deserialize)]
pub struct EnhancedChatRequest {
    pub message: String,
    pub conversation_history: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,  // user, assistant, system
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct EnhancedChatResponse {
    pub data: String,
    pub query_executed: bool,
    pub query_type: Option<String>,
}

// ========== AI 内部请求格式 ==========

/// AI 发送的数据库查询请求格式
#[derive(Debug, Serialize, Deserialize)]
pub struct AIQueryRequest {
    /// 查询类型
    pub query_type: String,
    /// 查询参数（如班级ID、部门ID等）
    pub params: Option<serde_json::Value>,
    /// 查询原因/说明
    pub reason: String,
}

/// AI 响应中的操作标记
#[derive(Debug, Serialize, Deserialize)]
pub struct AIAction {
    /// 操作类型: query_data, execute_action, final_answer, need_more_info
    pub action: String,
    /// 操作数据
    pub data: Option<serde_json::Value>,
}

// ========== 系统提示词模板 ==========

pub struct SystemPromptTemplate;

impl SystemPromptTemplate {
    /// 获取增强版系统提示词
    pub fn get_enhanced_prompt(user_permissions: &[String]) -> String {
        let permissions_str = user_permissions.join(", ");
        
        // 根据权限构建可用操作说明
        let mut action_descriptions = vec![];
        
        if user_permissions.iter().any(|p| p == "notice.create" || p == "notice.*") {
            action_descriptions.push(r#"**创建公告** (create_notice)
- 用途：创建新的学校公告
- 参数：title(标题), content(内容), target_type(目标类型: all/class/department/group), target_id(可选), is_important(是否重要)
- 示例：{"action_type": "create_notice", "params": {"title": "期中考试通知", "content": "下周一进行期中考试", "target_type": "all"}, "reason": "发布考试通知"}"#);
        }
        
        if user_permissions.iter().any(|p| p == "group.create" || p == "group.*") {
            action_descriptions.push(r#"**创建小组** (create_group)
- 用途：在指定班级创建新的小组
- 参数：class_id(班级ID或名称), name(小组名称), description(描述,可选)
- 示例：{"action_type": "create_group", "params": {"class_id": "一年级1班", "name": "学习小组1"}, "reason": "创建新学习小组"}"#);
        }
        
        if user_permissions.iter().any(|p| p == "group.update.score" || p == "group.update" || p == "group.*") {
            action_descriptions.push(r#"**更新小组积分** (update_group_score)
- 用途：增加或减少小组积分
- 参数：group_id(小组ID或名称), score_change(积分变化,正数增加负数减少), reason(原因)
- 示例：{"action_type": "update_group_score", "params": {"group_id": "香芋组", "score_change": 10, "reason": "课堂表现优秀"}, "reason": "奖励优秀表现"}"#);
        }
        
        if user_permissions.iter().any(|p| p == "group.update.member" || p == "group.update" || p == "group.*") {
            action_descriptions.push(r#"**添加小组成员** (add_group_member)
- 用途：向小组添加新成员
- 参数：group_id(小组ID或名称), person_id(人员姓名或ID)
- 示例：{"action_type": "add_group_member", "params": {"group_id": "香芋组", "person_id": "小明"}, "reason": "添加新成员"}"#);
            action_descriptions.push(r#"**移除小组成员** (remove_group_member)
- 用途：从小组移除成员
- 参数：group_id(小组ID或名称), person_id(人员姓名或ID)
- 示例：{"action_type": "remove_group_member", "params": {"group_id": "香芋组", "person_id": "小明"}, "reason": "移除成员"}"#);
        }
        
        if user_permissions.iter().any(|p| p == "attendance.create" || p == "attendance.*") {
            action_descriptions.push(r#"**创建考勤记录** (create_attendance)
- 用途：为人员创建考勤记录
- 参数：person_id(人员姓名或ID), date(日期,支持:今天/明天/昨天/YYYY-MM-DD), status(状态:出勤/迟到/缺勤/早退/请假或present/late/absent/early_leave/excused), time(时间,支持:现在/上午8点/下午3点/HH:MM), remark(备注,可选)
- 示例：{"action_type": "create_attendance", "params": {"person_id": "小绿", "date": "今天", "status": "出勤", "time": "上午8点"}, "reason": "记录出勤"}"#);
        }
        
        if user_permissions.iter().any(|p| p == "score.create" || p == "score.*") {
            action_descriptions.push(r#"**添加个人积分** (create_score)
- 用途：为人员添加个人表现积分
- 参数：student_id(人员姓名或ID), reason(原因), value(积分值,整数)
- 示例：{"action_type": "create_score", "params": {"student_id": "小绿", "reason": "大扫除优秀", "value": 10}, "reason": "奖励优秀表现"}"#);
        }
        
        let actions_str = if action_descriptions.is_empty() {
            "当前用户没有可执行的操作权限。".to_string()
        } else {
            action_descriptions.join("\n\n")
        };
        
        format!(r#"你是一个学校管理系统的AI助手。你的任务是帮助用户查询和分析学校数据，并根据用户权限执行相应操作。

## 你的权限
用户拥有以下权限: {}

## 重要规则

### 1. 数据查询规则
当用户询问需要数据库信息的问题时，你必须使用特定的JSON格式来请求数据。

**格式要求**:
```json
[AI_QUERY]
{{
    "query_type": "查询类型",
    "params": {{}},
    "reason": "查询原因"
}}
[/AI_QUERY]
```

**可用的查询类型**:
- `class_list` - 获取所有班级列表
- `class_detail` - 获取班级详情（需要params: {{"id": "班级UUID"}}）
- `group_list` - 获取所有小组列表
- `group_detail` - 获取小组详情（需要params: {{"id": "小组UUID"}}）
- `department_list` - 获取所有部门列表
- `department_detail` - 获取部门详情（需要params: {{"id": "部门UUID"}}）
- `overview` - 获取学校数据概览

### 2. 操作执行规则
当用户要求你执行某个操作时（如创建公告、创建小组等），你必须使用特定的JSON格式来请求执行操作。

**格式要求**:
```json
[AI_ACTION]
{{
    "action_type": "操作类型",
    "params": {{具体参数}},
    "reason": "操作原因"
}}
[/AI_ACTION]
```

**你可以执行的操作**:
{}

**重要提示**:
- 只有当用户明确要求执行某个操作时，才使用 [AI_ACTION] 格式
- 如果用户没有某个操作的权限，使用 [AI_ANSWER] 回复："抱歉，您没有执行此操作的权限。"
- 执行操作前，确保所有必需参数都已提供，否则询问用户补充
- **名称解析**：你可以直接使用人员姓名、小组名称、班级名称代替UUID，系统会自动解析。例如："小绿"、"香芋组"、"一年级1班"
- **日期自动补全**：可以使用"今天"、"明天"、"昨天"等相对日期，系统会自动转换为标准格式
- **时间自动补全**：可以使用"上午8点"、"下午3点"、"现在"等描述，系统会自动转换
- **考勤状态**：可以使用中文"出勤"、"迟到"、"缺勤"、"早退"、"请假"，系统会自动转换

### 2.1 重名处理规则
当系统返回需要确认的信息时（如找到多个同名人员），表示需要用户选择具体是哪一个：
- 系统会返回候选列表，包含每个人的额外信息（如学号、班级等）
- 你需要将这些选项呈现给用户，让用户选择
- 使用 [AI_NEED_INFO] 格式询问用户具体选择哪一个

### 3. 回答格式规则

**当你需要查询数据时**:
先输出 [AI_QUERY] JSON [/AI_QUERY] 格式的查询请求，系统会自动执行查询并返回结果。

**当你需要执行操作时**:
先输出 [AI_ACTION] JSON [/AI_ACTION] 格式的操作请求，系统会执行操作并返回结果。

**当你直接回答用户时**:
使用 [AI_ANSWER] 标记开始你的回答：
```
[AI_ANSWER]
你的回答内容...
```

**当你需要更多信息时**:
```
[AI_NEED_INFO]
我需要更多信息才能回答你的问题，请提供：...
```

### 4. 示例对话

**示例1 - 查询班级信息**:
用户: "帮我查看所有班级"
AI:
```
[AI_QUERY]
{{
    "query_type": "class_list",
    "params": {{}},
    "reason": "用户请求查看所有班级信息"
}}
[/AI_QUERY]
```

**示例2 - 创建公告**:
用户: "帮我创建一个公告，标题是期中考试通知"
AI:
```
[AI_ACTION]
{{
    "action_type": "create_notice",
    "params": {{"title": "期中考试通知", "content": "下周一进行期中考试，请同学们做好准备。", "target_type": "all"}},
    "reason": "用户要求创建期中考试通知公告"
}}
[/AI_ACTION]
```

**示例3 - 直接回答**:
用户: "你好"
AI:
```
[AI_ANSWER]
你好！我是学校管理系统的AI助手，可以帮你查询班级、小组、部门等信息，并根据你的权限执行相应操作。请问有什么可以帮你的？
```

### 5. 权限控制
- 你只能查询和执行用户权限范围内的操作
- 如果用户询问或请求执行没有权限的操作，使用 [AI_ANSWER] 标记回复："抱歉，您没有执行此操作的权限。"
- 不要编造数据，所有数据必须通过查询接口获取
- 不要越权执行操作，必须严格遵守权限限制

### 6. 数据分析
当获取到数据后，你应该：
1. 分析数据内容
2. 以清晰、易懂的方式呈现结果
3. 可以适当总结和计算（如平均值、总数等）
4. 使用中文回复用户

现在，请根据以上规则回答用户的问题。"#, permissions_str, actions_str)
    }
    
    /// 获取数据查询后的提示词
    pub fn get_data_analysis_prompt(query_result: &str, original_question: &str) -> String {
        format!(r#"以下是通过数据库查询获取的原始数据（Markdown格式）：

```
{}
```

用户的问题是："{}"

请分析以上数据，并以清晰、友好的方式回答用户的问题。注意：
1. 使用 [AI_ANSWER] 标记开始你的回答
2. 数据已经过权限验证，用户有权查看
3. 可以适当进行计算和总结
4. 使用中文回复
5. 如果数据为空，说明没有找到相关数据"#, query_result, original_question)
    }
}

// ========== AI 响应解析器 ==========

pub struct AIResponseParser;

impl AIResponseParser {
    /// 解析AI响应，提取操作标记
    pub fn parse_response(content: &str) -> AIAction {
        // 检查是否有操作执行请求（优先级最高）
        if let Some(action) = Self::extract_action(content) {
            return AIAction {
                action: "execute_action".to_string(),
                data: Some(serde_json::to_value(action).unwrap_or_default()),
            };
        }
        
        // 检查是否有查询请求
        if let Some(query) = Self::extract_query(content) {
            return AIAction {
                action: "query_data".to_string(),
                data: Some(serde_json::to_value(query).unwrap_or_default()),
            };
        }
        
        // 检查是否需要更多信息
        if content.contains("[AI_NEED_INFO]") {
            let info = content.replace("[AI_NEED_INFO]", "").trim().to_string();
            return AIAction {
                action: "need_more_info".to_string(),
                data: Some(serde_json::json!({"message": info})),
            };
        }
        
        // 默认是直接回答
        let answer = content.replace("[AI_ANSWER]", "").trim().to_string();
        AIAction {
            action: "final_answer".to_string(),
            data: Some(serde_json::json!({"message": answer})),
        }
    }
    
    /// 提取查询请求
    fn extract_query(content: &str) -> Option<AIQueryRequest> {
        // 使用正则表达式匹配 [AI_QUERY]...[/AI_QUERY] 格式
        let re = Regex::new(r"\[AI_QUERY\]\s*(\{[\s\S]*?\})\s*\[/AI_QUERY\]").ok()?;
        
        if let Some(caps) = re.captures(content) {
            if let Some(json_str) = caps.get(1) {
                return serde_json::from_str(json_str.as_str()).ok();
            }
        }
        
        None
    }
    
    /// 提取操作执行请求
    fn extract_action(content: &str) -> Option<AIActionRequest> {
        // 使用正则表达式匹配 [AI_ACTION]...[/AI_ACTION] 格式
        let re = Regex::new(r"\[AI_ACTION\]\s*(\{[\s\S]*?\})\s*\[/AI_ACTION\]").ok()?;
        
        if let Some(caps) = re.captures(content) {
            if let Some(json_str) = caps.get(1) {
                return serde_json::from_str(json_str.as_str()).ok();
            }
        }
        
        None
    }
    
    /// 清理响应内容，移除标记
    pub fn clean_response(content: &str) -> String {
        content
            .replace("[AI_ANSWER]", "")
            .replace("[/AI_ANSWER]", "")
            .replace("[AI_QUERY]", "")
            .replace("[/AI_QUERY]", "")
            .replace("[AI_ACTION]", "")
            .replace("[/AI_ACTION]", "")
            .replace("[AI_NEED_INFO]", "")
            .replace("[/AI_NEED_INFO]", "")
            .trim()
            .to_string()
    }
}

// ========== 数据查询执行器 ==========

pub struct DataQueryExecutor;

impl DataQueryExecutor {
    /// 执行AI请求的查询
    pub async fn execute(
        pool: &PgPool,
        query_req: &AIQueryRequest,
        user_permissions: &[String],
    ) -> Result<String, AppError> {
        // 检查权限
        let has_permission = match query_req.query_type.as_str() {
            "class_list" | "class_detail" => {
                user_permissions.iter().any(|p| p == "class.view" || p == "class.*")
            }
            "group_list" | "group_detail" => {
                user_permissions.iter().any(|p| p == "group.view" || p == "group.*")
            }
            "department_list" | "department_detail" => {
                user_permissions.iter().any(|p| p == "department.view" || p == "department.*")
            }
            "overview" => {
                user_permissions.iter().any(|p| {
                    p == "class.view" || p == "group.view" || p == "department.view" ||
                    p == "class.*" || p == "group.*" || p == "department.*"
                })
            }
            _ => false,
        };
        
        if !has_permission {
            return Err(AppError::Auth(format!("没有权限执行查询: {}", query_req.query_type)));
        }
        
        // 执行查询
        match query_req.query_type.as_str() {
            "class_list" => {
                let service = ClassDataService::new(pool.clone());
                let classes = service.get_all_classes().await?;
                Ok(MarkdownFormatter::format_class_list(&classes))
            }
            "class_detail" => {
                let class_id = query_req.params
                    .as_ref()
                    .and_then(|p| p.get("id"))
                    .and_then(|id| id.as_str())
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .ok_or(AppError::InvalidInput("缺少班级ID".to_string()))?;
                
                let service = ClassDataService::new(pool.clone());
                let detail = service.get_class_detail(class_id).await?;
                Ok(MarkdownFormatter::format_class_detail(&detail))
            }
            "group_list" => {
                let service = GroupDataService::new(pool.clone());
                let groups = service.get_all_groups().await?;
                Ok(MarkdownFormatter::format_group_list(&groups))
            }
            "group_detail" => {
                let group_id_str = query_req.params
                    .as_ref()
                    .and_then(|p| p.get("id"))
                    .and_then(|id| id.as_str())
                    .ok_or(AppError::InvalidInput("缺少小组ID".to_string()))?;
                
                // 使用 NameResolver 解析小组名称
                match NameResolver::resolve_group(pool, group_id_str).await? {
                    ResolutionResult::Single(id) => {
                        let group_id = Uuid::parse_str(&id)
                            .map_err(|_| AppError::InvalidInput(format!("无效的小组ID: {}", id)))?;
                        let service = GroupDataService::new(pool.clone());
                        let detail = service.get_group_detail(group_id).await?;
                        Ok(MarkdownFormatter::format_group_detail(&detail))
                    }
                    ResolutionResult::Multiple(candidates) => {
                        // 有多个同名小组，返回错误信息让AI询问用户
                        let candidates_info: Vec<String> = candidates
                            .iter()
                            .map(|c| format!("{} ({})", c.name, c.info))
                            .collect();
                        Err(AppError::InvalidInput(format!(
                            "找到多个名为 '{}' 的小组: {}。请指定具体是哪一个（例如提供班级信息）",
                            group_id_str,
                            candidates_info.join(", ")
                        )))
                    }
                    ResolutionResult::NotFound(msg) => {
                        Err(AppError::InvalidInput(msg))
                    }
                }
            }
            "department_list" => {
                let service = DepartmentDataService::new(pool.clone());
                let departments = service.get_all_departments().await?;
                Ok(MarkdownFormatter::format_department_list(&departments))
            }
            "department_detail" => {
                let dept_id = query_req.params
                    .as_ref()
                    .and_then(|p| p.get("id"))
                    .and_then(|id| id.as_str())
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .ok_or(AppError::InvalidInput("缺少部门ID".to_string()))?;
                
                let service = DepartmentDataService::new(pool.clone());
                let detail = service.get_department_detail(dept_id).await?;
                Ok(MarkdownFormatter::format_department_detail(&detail))
            }
            "overview" => {
                let class_service = ClassDataService::new(pool.clone());
                let group_service = GroupDataService::new(pool.clone());
                let dept_service = DepartmentDataService::new(pool.clone());
                
                let classes = if user_permissions.iter().any(|p| p == "class.view" || p == "class.*") {
                    class_service.get_all_classes().await.unwrap_or_default()
                } else {
                    Vec::new()
                };
                
                let groups = if user_permissions.iter().any(|p| p == "group.view" || p == "group.*") {
                    group_service.get_all_groups().await.unwrap_or_default()
                } else {
                    Vec::new()
                };
                
                let departments = if user_permissions.iter().any(|p| p == "department.view" || p == "department.*") {
                    dept_service.get_all_departments().await.unwrap_or_default()
                } else {
                    Vec::new()
                };
                
                Ok(MarkdownFormatter::format_overview(&classes, &groups, &departments))
            }
            _ => Err(AppError::InvalidInput(format!("未知的查询类型: {}", query_req.query_type))),
        }
    }
}

// ========== 增强版AI聊天API ==========

pub async fn enhanced_chat(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<EnhancedChatRequest>,
) -> Result<Json<EnhancedChatResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 获取AI设置
    let settings = sqlx::query_as::<_, crate::api::ai::AISettings>(
        "SELECT api_key, api_base_url, model, default_prompt, temperature, max_tokens 
         FROM ai_settings 
         ORDER BY id DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    
    // 获取用户权限
    let permission_manager = PermissionManager::new(pool.clone());
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;
    
    // 检查AI聊天权限
    if !user_permissions.iter().any(|p| p == "ai.chat") {
        return Err(AppError::Auth("没有 AI 聊天权限".to_string()));
    }
    
    // 构建系统提示词
    let system_prompt = SystemPromptTemplate::get_enhanced_prompt(&user_permissions);
    
    // 构建消息列表
    let mut messages = vec![
        crate::api::ai::AIChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }
    ];
    
    // 添加历史消息
    for msg in req.conversation_history {
        messages.push(crate::api::ai::AIChatMessage {
            role: msg.role,
            content: msg.content,
        });
    }
    
    // 添加当前用户消息
    messages.push(crate::api::ai::AIChatMessage {
        role: "user".to_string(),
        content: req.message.clone(),
    });
    
    // 克隆需要的设置值
    let model = settings.model.clone();
    let temperature = settings.temperature;
    let max_tokens = settings.max_tokens;
    let api_key = settings.api_key.clone();
    
    // 调用AI API
    let api_request = crate::api::ai::AIChatRequest {
        model: model.clone(),
        messages,
        temperature,
        max_tokens,
        stream: false,
    };
    
    let client = Client::new();
    let mut api_url = settings.api_base_url.clone();
    if !api_url.ends_with("/v1/chat/completions") && !api_url.ends_with("/v1/chat/completions/") {
        if !api_url.ends_with('/') {
            api_url.push('/');
        }
        api_url.push_str("v1/chat/completions");
    }
    
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&api_request)
        .send()
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("AI API 请求失败: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误响应".to_string());
        return Err(AppError::InternalWithMessage(format!("AI API 返回错误: {} - {}", status, error_text)));
    }
    
    let api_response: crate::api::ai::AIChatResponse = response
        .json()
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("解析 AI API 响应失败: {}", e)))?;
    
    let ai_content = api_response.choices
        .first()
        .map(|choice| choice.message.content.clone())
        .unwrap_or_else(|| "AI 没有返回有效回复".to_string());
    
    // 解析AI响应
    let action = AIResponseParser::parse_response(&ai_content);
    
    // 根据操作类型处理
    match action.action.as_str() {
        "query_data" => {
            // AI请求查询数据
            if let Some(data) = action.data {
                if let Ok(query_req) = serde_json::from_value::<AIQueryRequest>(data) {
                    // 执行查询
                    match DataQueryExecutor::execute(&pool, &query_req, &user_permissions).await {
                        Ok(query_result) => {
                            // 构建第二次AI请求，让AI分析数据
                            let analysis_prompt = SystemPromptTemplate::get_data_analysis_prompt(
                                &query_result, 
                                &req.message
                            );
                            
                            let analysis_messages = vec![
                                crate::api::ai::AIChatMessage {
                                    role: "system".to_string(),
                                    content: analysis_prompt,
                                },
                                crate::api::ai::AIChatMessage {
                                    role: "user".to_string(),
                                    content: "请分析以上数据并回答我的问题".to_string(),
                                },
                            ];
                            
                            let analysis_request = crate::api::ai::AIChatRequest {
                                model: model.clone(),
                                messages: analysis_messages,
                                temperature,
                                max_tokens,
                                stream: false,
                            };
                            
                            let analysis_response = client
                                .post(&api_url)
                                .header("Content-Type", "application/json")
                                .header("Authorization", format!("Bearer {}", api_key))
                                .json(&analysis_request)
                                .send()
                                .await
                                .map_err(|e| AppError::InternalWithMessage(format!("AI API 请求失败: {}", e)))?;
                            
                            if analysis_response.status().is_success() {
                                let analysis_result: crate::api::ai::AIChatResponse = analysis_response
                                    .json()
                                    .await
                                    .map_err(|e| AppError::InternalWithMessage(format!("解析 AI API 响应失败: {}", e)))?;
                                
                                let final_content = analysis_result.choices
                                    .first()
                                    .map(|choice| AIResponseParser::clean_response(&choice.message.content))
                                    .unwrap_or_else(|| query_result);
                                
                                return Ok(Json(EnhancedChatResponse {
                                    data: final_content,
                                    query_executed: true,
                                    query_type: Some(query_req.query_type),
                                }));
                            }
                            
                            // 如果第二次请求失败，直接返回查询结果
                            Ok(Json(EnhancedChatResponse {
                                data: query_result,
                                query_executed: true,
                                query_type: Some(query_req.query_type),
                            }))
                        }
                        Err(e) => {
                            // 查询失败，返回错误信息
                            let error_msg = format!("查询数据失败: {}", e);
                            Ok(Json(EnhancedChatResponse {
                                data: error_msg,
                                query_executed: true,
                                query_type: Some(query_req.query_type),
                            }))
                        }
                    }
                } else {
                    // 解析查询请求失败
                    Ok(Json(EnhancedChatResponse {
                        data: "AI 请求的查询格式不正确".to_string(),
                        query_executed: false,
                        query_type: None,
                    }))
                }
            } else {
                Ok(Json(EnhancedChatResponse {
                    data: "AI 请求的数据为空".to_string(),
                    query_executed: false,
                    query_type: None,
                }))
            }
        }
        "execute_action" => {
            // AI请求执行操作
            if let Some(data) = action.data {
                if let Ok(action_req) = serde_json::from_value::<AIActionRequest>(data) {
                    // 获取用户名称
                    let user_name: String = sqlx::query_scalar(
                        "SELECT name FROM persons WHERE id = $1"
                    )
                    .bind(user_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap_or_else(|_| "未知用户".to_string());
                    
                    // 执行操作
                    match AIActionExecutor::execute(&pool, &action_req, user_id, &user_name).await {
                        Ok(action_result) => {
                            if action_result.success {
                                // 操作成功，构建成功提示
                                let success_prompt = format!(r#"你刚刚成功执行了一个操作。

操作结果：
{}

用户原始请求："{}"

请用友好、简洁的方式告知用户操作已成功完成，并简要说明操作结果。使用 [AI_ANSWER] 标记开始你的回复。"#, 
                                    serde_json::to_string_pretty(&action_result.data).unwrap_or_default(),
                                    req.message
                                );
                                
                                let success_messages = vec![
                                    crate::api::ai::AIChatMessage {
                                        role: "system".to_string(),
                                        content: success_prompt,
                                    },
                                    crate::api::ai::AIChatMessage {
                                        role: "user".to_string(),
                                        content: "请告诉我操作结果".to_string(),
                                    },
                                ];
                                
                                let success_request = crate::api::ai::AIChatRequest {
                                    model: model.clone(),
                                    messages: success_messages,
                                    temperature,
                                    max_tokens,
                                    stream: false,
                                };
                                
                                let success_response = client
                                    .post(&api_url)
                                    .header("Content-Type", "application/json")
                                    .header("Authorization", format!("Bearer {}", api_key))
                                    .json(&success_request)
                                    .send()
                                    .await
                                    .map_err(|e| AppError::InternalWithMessage(format!("AI API 请求失败: {}", e)))?;
                                
                                if success_response.status().is_success() {
                                    let success_result: crate::api::ai::AIChatResponse = success_response
                                        .json()
                                        .await
                                        .map_err(|e| AppError::InternalWithMessage(format!("解析 AI API 响应失败: {}", e)))?;
                                    
                                    let final_content = success_result.choices
                                        .first()
                                        .map(|choice| AIResponseParser::clean_response(&choice.message.content))
                                        .unwrap_or_else(|| action_result.message.clone());
                                    
                                    return Ok(Json(EnhancedChatResponse {
                                        data: final_content,
                                        query_executed: true,
                                        query_type: Some(action_req.action_type),
                                    }));
                                }
                                
                                // 如果第二次请求失败，直接返回操作结果
                                Ok(Json(EnhancedChatResponse {
                                    data: action_result.message,
                                    query_executed: true,
                                    query_type: Some(action_req.action_type),
                                }))
                            } else {
                                // 操作失败，返回错误信息
                                Ok(Json(EnhancedChatResponse {
                                    data: format!("操作失败：{}", action_result.message),
                                    query_executed: true,
                                    query_type: Some(action_req.action_type),
                                }))
                            }
                        }
                        Err(e) => {
                            // 执行操作失败，返回错误信息
                            let error_msg = format!("执行操作失败: {}", e);
                            Ok(Json(EnhancedChatResponse {
                                data: error_msg,
                                query_executed: true,
                                query_type: Some(action_req.action_type),
                            }))
                        }
                    }
                } else {
                    // 解析操作请求失败
                    Ok(Json(EnhancedChatResponse {
                        data: "AI 请求的操作格式不正确".to_string(),
                        query_executed: false,
                        query_type: None,
                    }))
                }
            } else {
                Ok(Json(EnhancedChatResponse {
                    data: "AI 请求的操作数据为空".to_string(),
                    query_executed: false,
                    query_type: None,
                }))
            }
        }
        "need_more_info" => {
            // AI需要更多信息
            let message = action.data
                .and_then(|d| d.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
                .unwrap_or_else(|| "我需要更多信息才能回答你的问题".to_string());
            
            Ok(Json(EnhancedChatResponse {
                data: message,
                query_executed: false,
                query_type: None,
            }))
        }
        _ => {
            // 直接回答
            let final_answer = action.data
                .and_then(|d| d.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
                .unwrap_or_else(|| AIResponseParser::clean_response(&ai_content));
            
            Ok(Json(EnhancedChatResponse {
                data: final_answer,
                query_executed: false,
                query_type: None,
            }))
        }
    }
}
