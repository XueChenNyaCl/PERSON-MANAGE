# AI功能重构开发文档

## 1. 项目概述

本文档详细描述了项目AI功能的重构方案，包括：
- AI数据库操作能力（新增考勤、新增人员等）
- 批量操作支持
- 页面顶部AI助手提示功能
- JSON请求/响应格式规范
- 提示词设计
- 多步骤操作处理
- 页面信息获取策略

## 2. 文件目录结构

### 2.1 后端文件

```
backend/src/
├── api/
│   ├── ai.rs                  # AI基础聊天接口（保持）
│   ├── ai_actions.rs          # AI操作执行模块（重构）
│   ├── ai_data.rs             # AI数据查询模块（重构）
│   ├── ai_enhanced.rs         # 增强版AI聊天模块（重构）
│   ├── ai_assistant.rs        # AI助手提示模块（新增）
│   └── ai_context_provider.rs # AI上下文数据提供者（新增）
├── models/
│   └── ai_action.rs           # AI操作数据模型（新增）
└── core/
    ├── ai_action_validator.rs # AI操作验证器（新增）
    └── ai_orchestrator.rs     # AI编排器（新增，处理多步骤）
```

### 2.2 前端文件

```
frontend/src/
├── api/
│   └── ai.ts                  # AI API接口（更新）
├── components/
│   ├── AIAssistant.vue        # AI助手提示组件（新增）
│   └── AIActionExecutor.vue   # AI操作执行器（新增）
├── store/
│   └── ai.ts                  # AI状态管理（新增）
├── styles/
│   ├── ai-assistant.css       # AI助手提示样式（新增）
│   └── ai-view.css            # AI视图样式（更新）
├── views/
│   └── AIView.vue             # AI对话页面（更新）
└── composables/
    └── usePageContext.ts      # 页面上下文获取（新增）
```

## 3. 数据库操作设计

### 3.1 AI可执行的操作列表

| 操作类型 | 操作名称 | 所需权限 | 描述 |
|---------|---------|---------|------|
| 人员管理 | create_person | person.create | 创建单个人员 |
| 人员管理 | create_persons_batch | person.create | 批量创建人员 |
| 考勤管理 | create_attendance | attendance.create | 创建单条考勤记录 |
| 考勤管理 | create_attendances_batch | attendance.create | 批量创建考勤记录 |
| 成绩管理 | create_score | score.create | 创建单条成绩记录 |
| 成绩管理 | create_scores_batch | score.create | 批量创建成绩记录 |
| 公告管理 | create_notice | notice.create | 创建公告 |
| 数据查询 | get_persons | person.view | 查询人员列表 |
| 数据查询 | get_attendances | attendance.view | 查询考勤记录 |
| 数据查询 | get_classes | class.view | 查询班级列表 |
| 数据查询 | get_groups | group.view | 查询小组列表 |
| 数据查询 | get_departments | department.view | 查询部门列表 |
| 数据查询 | get_notices | notice.view | 查询公告列表 |

### 3.2 JSON请求格式规范

#### 3.2.1 AI操作请求头（支持多步骤）

```json
{
  "request_id": "uuid-string",
  "timestamp": "2026-03-07T10:00:00Z",
  "user_id": "uuid-string",
  "operation_type": "ai_action",
  "batch": false,
  "multi_step": {
    "enabled": true,
    "step_id": "step-1",
    "total_steps": 3,
    "depends_on": null,
    "session_id": "session-uuid"
  }
}
```

**多步骤操作说明：**
- `multi_step.enabled`: 是否启用多步骤操作
- `multi_step.step_id`: 当前步骤ID
- `multi_step.total_steps`: 总步骤数
- `multi_step.depends_on`: 依赖的上一步骤ID（null表示第一步）
- `multi_step.session_id`: 会话ID，用于关联同一任务的多个步骤

#### 3.2.2 单条操作请求格式

```json
{
  "action_type": "create_person",
  "params": {
    "name": "张三",
    "gender": 1,
    "type": "student",
    "student_no": "2026001",
    "class_id": "uuid-string",
    "phone": "13800138000",
    "email": "zhangsan@example.com"
  },
  "reason": "用户要求创建学生张三"
}
```

#### 3.2.3 批量操作请求格式

```json
{
  "action_type": "create_persons_batch",
  "batch": true,
  "items": [
    {
      "name": "张三",
      "gender": 1,
      "type": "student",
      "student_no": "2026001",
      "class_id": "uuid-string"
    },
    {
      "name": "李四",
      "gender": 2,
      "type": "student",
      "student_no": "2026002",
      "class_id": "uuid-string"
    }
  ],
  "reason": "用户要求批量创建学生"
}
```

#### 3.2.4 多步骤操作示例

**步骤1：查询班级列表**
```json
{
  "action_type": "get_classes",
  "params": {},
  "reason": "需要获取班级列表以创建学生"
}
```

**步骤2：创建学生（依赖步骤1的结果）**
```json
{
  "action_type": "create_person",
  "params": {
    "name": "张三",
    "gender": 1,
    "type": "student",
    "student_no": "2026001",
    "class_id": "{{step-1.data.classes[0].id}}",
    "phone": "13800138000"
  },
  "reason": "使用步骤1获取的第一个班级创建学生"
}
```

### 3.3 具体操作参数说明

#### 3.3.1 创建人员 (create_person)

```json
{
  "action_type": "create_person",
  "params": {
    "name": "张三",
    "gender": 1,
    "type": "student",
    "birthday": "2010-01-01",
    "phone": "13800138000",
    "email": "zhangsan@example.com",
    "student_no": "2026001",
    "class_id": "uuid-string",
    "enrollment_date": "2026-09-01"
  }
}
```

**参数说明：**
- `name`: 姓名（必填）
- `gender`: 性别，0=未知，1=男，2=女（必填）
- `type`: 人员类型，'student'|'teacher'|'parent'（必填）
- `birthday`: 出生日期，格式YYYY-MM-DD（可选）
- `phone`: 电话号码（可选）
- `email`: 邮箱地址（可选）
- `student_no`: 学号（student类型必填）
- `class_id`: 班级ID（student类型可选）
- `enrollment_date`: 入学日期（student类型可选）
- `employee_no`: 工号（teacher类型必填）
- `department_id`: 部门ID（teacher类型可选）
- `title`: 职称（teacher类型可选）
- `hire_date`: 入职日期（teacher类型可选）
- `wechat_openid`: 微信OpenID（parent类型可选）
- `occupation`: 职业（parent类型可选）

#### 3.3.2 批量创建人员 (create_persons_batch)

```json
{
  "action_type": "create_persons_batch",
  "batch": true,
  "items": [
    {
      "name": "张三",
      "gender": 1,
      "type": "student",
      "student_no": "2026001",
      "class_id": "uuid-string"
    },
    {
      "name": "李四",
      "gender": 2,
      "type": "student",
      "student_no": "2026002",
      "class_id": "uuid-string"
    }
  ]
}
```

#### 3.3.3 创建考勤 (create_attendance)

```json
{
  "action_type": "create_attendance",
  "params": {
    "person_id": "uuid-string",
    "date": "2026-03-07",
    "status": "present",
    "time": "08:00:00",
    "remark": "正常出勤"
  }
}
```

**参数说明：**
- `person_id`: 人员ID（必填）
- `date`: 考勤日期，格式YYYY-MM-DD（必填）
- `status`: 考勤状态，'present'|'absent'|'late'|'early_leave'|'excused'（必填）
- `time`: 考勤时间，格式HH:MM:SS（可选）
- `remark`: 备注（可选）

#### 3.3.4 批量创建考勤 (create_attendances_batch)

```json
{
  "action_type": "create_attendances_batch",
  "batch": true,
  "items": [
    {
      "person_id": "uuid-string-1",
      "date": "2026-03-07",
      "status": "present",
      "time": "08:00:00"
    },
    {
      "person_id": "uuid-string-2",
      "date": "2026-03-07",
      "status": "late",
      "time": "08:15:00",
      "remark": "迟到15分钟"
    }
  ]
}
```

#### 3.3.5 创建成绩 (create_score)

```json
{
  "action_type": "create_score",
  "params": {
    "person_id": "uuid-string",
    "score_type": "personal",
    "value": 95,
    "reason": "期中考试数学成绩"
  }
}
```

**参数说明：**
- `person_id`: 人员ID（必填）
- `score_type`: 分数类型，'personal'|'group'|'class'|'dormitory'（必填）
- `value`: 分数值，0-100（必填）
- `reason`: 评分原因（必填）
- `group_id`: 关联组ID（可选）
- `event_id`: 关联事件ID（可选）

#### 3.3.6 查询公告 (get_notices)

```json
{
  "action_type": "get_notices",
  "params": {
    "limit": 5,
    "sort_by": "created_at",
    "order": "desc"
  }
}
```

**参数说明：**
- `limit`: 返回数量限制（可选，默认10）
- `sort_by`: 排序字段（可选，默认created_at）
- `order`: 排序方向，'asc'|'desc'（可选，默认desc）

### 3.4 JSON响应格式规范

#### 3.4.1 成功响应

```json
{
  "success": true,
  "request_id": "uuid-string",
  "action_type": "create_person",
  "message": "人员创建成功",
  "data": {
    "id": "uuid-string",
    "name": "张三",
    "type": "student",
    "created_at": "2026-03-07T10:00:00Z"
  },
  "user_permissions": ["person.create", "person.view"],
  "next_step": {
    "suggested": true,
    "options": ["create_attendance", "add_to_group"]
  }
}
```

#### 3.4.2 批量操作成功响应

```json
{
  "success": true,
  "request_id": "uuid-string",
  "action_type": "create_persons_batch",
  "message": "批量创建完成，成功2个，失败0个",
  "data": {
    "total": 2,
    "success_count": 2,
    "failure_count": 0,
    "items": [
      {
        "success": true,
        "index": 0,
        "data": { "id": "uuid-string-1", "name": "张三" }
      },
      {
        "success": true,
        "index": 1,
        "data": { "id": "uuid-string-2", "name": "李四" }
      }
    ]
  },
  "session": {
    "id": "session-uuid",
    "completed_steps": 1,
    "remaining_steps": 0
  }
}
```

#### 3.4.3 多步骤操作中间响应

```json
{
  "success": true,
  "request_id": "uuid-string",
  "action_type": "get_classes",
  "message": "查询成功",
  "data": {
    "classes": [
      { "id": "uuid-1", "name": "高一(1)班", "grade": 10 },
      { "id": "uuid-2", "name": "高一(2)班", "grade": 10 }
    ]
  },
  "multi_step": {
    "step_id": "step-1",
    "status": "completed",
    "next_step": "step-2"
  }
}
```

#### 3.4.4 错误响应

```json
{
  "success": false,
  "request_id": "uuid-string",
  "action_type": "create_person",
  "error_code": "PERMISSION_DENIED",
  "message": "没有创建人员的权限",
  "details": "用户权限: ['person.view'], 需要权限: ['person.create']",
  "recoverable": false,
  "suggestions": ["联系管理员获取权限", "尝试其他操作"]
}
```

**错误码说明：**
- `PERMISSION_DENIED`: 权限不足
- `INVALID_INPUT`: 输入参数无效
- `DATABASE_ERROR`: 数据库操作错误
- `NOT_FOUND`: 资源不存在
- `CONFLICT`: 数据冲突（如重复学号）
- `INTERNAL_ERROR`: 内部错误
- `MULTI_STEP_ERROR`: 多步骤操作错误
- `DEPENDENCY_MISSING`: 依赖的步骤未完成

## 4. AI操作处理流程

### 4.1 AI操作参数验证器

**文件位置：** `backend/src/core/ai_action_validator.rs`

```rust
use serde_json::Value;
use uuid::Uuid;
use chrono::{NaiveDate, NaiveTime};

use crate::core::error::AppError;

/// 验证操作参数
pub struct AIActionValidator;

impl AIActionValidator {
    /// 验证创建人员参数
    pub fn validate_create_person(params: &Value) -> Result<(), AppError> {
        // 验证姓名字段
        let name = params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: name".to_string()))?;
        
        if name.trim().is_empty() {
            return Err(AppError::InvalidInput("姓名不能为空".to_string()));
        }
        
        if name.len() > 100 {
            return Err(AppError::InvalidInput("姓名长度不能超过100个字符".to_string()));
        }
        
        // 验证性别字段
        let gender = params.get("gender")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: gender".to_string()))?;
        
        if gender < 0 || gender > 2 {
            return Err(AppError::InvalidInput("性别值无效，应为0(未知)、1(男)或2(女)".to_string()));
        }
        
        // 验证人员类型字段
        let type_ = params.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: type".to_string()))?;
        
        if !["student", "teacher", "parent"].contains(&type_) {
            return Err(AppError::InvalidInput("人员类型无效，应为student、teacher或parent".to_string()));
        }
        
        // 根据人员类型验证特定字段
        match type_ {
            "student" => {
                // 验证学号
                let student_no = params.get("student_no")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::InvalidInput("学生缺少必填字段: student_no".to_string()))?;
                
                if student_no.trim().is_empty() {
                    return Err(AppError::InvalidInput("学号不能为空".to_string()));
                }
                
                if student_no.len() > 50 {
                    return Err(AppError::InvalidInput("学号长度不能超过50个字符".to_string()));
                }
            }
            "teacher" => {
                // 验证工号
                let employee_no = params.get("employee_no")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::InvalidInput("教师缺少必填字段: employee_no".to_string()))?;
                
                if employee_no.trim().is_empty() {
                    return Err(AppError::InvalidInput("工号不能为空".to_string()));
                }
                
                if employee_no.len() > 50 {
                    return Err(AppError::InvalidInput("工号长度不能超过50个字符".to_string()));
                }
            }
            _ => {}
        }
        
        // 验证可选的日期字段
        if let Some(birthday) = params.get("birthday").and_then(|v| v.as_str()) {
            if !birthday.is_empty() {
                Self::validate_date(birthday)?;
            }
        }
        
        if let Some(enrollment_date) = params.get("enrollment_date").and_then(|v| v.as_str()) {
            if !enrollment_date.is_empty() {
                Self::validate_date(enrollment_date)?;
            }
        }
        
        if let Some(hire_date) = params.get("hire_date").and_then(|v| v.as_str()) {
            if !hire_date.is_empty() {
                Self::validate_date(hire_date)?;
            }
        }
        
        // 验证可选的UUID字段
        if let Some(class_id) = params.get("class_id").and_then(|v| v.as_str()) {
            if !class_id.is_empty() {
                Self::validate_uuid(class_id)?;
            }
        }
        
        if let Some(department_id) = params.get("department_id").and_then(|v| v.as_str()) {
            if !department_id.is_empty() {
                Self::validate_uuid(department_id)?;
            }
        }
        
        Ok(())
    }
    
    /// 验证创建考勤参数
    pub fn validate_create_attendance(params: &Value) -> Result<(), AppError> {
        // 验证人员ID
        let person_id = params.get("person_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: person_id".to_string()))?;
        
        Self::validate_uuid(person_id)?;
        
        // 验证日期
        let date = params.get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: date".to_string()))?;
        
        Self::validate_date(date)?;
        
        // 验证状态
        let status = params.get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: status".to_string()))?;
        
        if !["present", "absent", "late", "early_leave", "excused"].contains(&status) {
            return Err(AppError::InvalidInput(
                "考勤状态无效，应为present、absent、late、early_leave或excused".to_string()
            ));
        }
        
        // 验证可选的时间字段
        if let Some(time) = params.get("time").and_then(|v| v.as_str()) {
            if !time.is_empty() {
                Self::validate_time(time)?;
            }
        }
        
        Ok(())
    }
    
    /// 验证创建成绩参数
    pub fn validate_create_score(params: &Value) -> Result<(), AppError> {
        // 验证人员ID
        let person_id = params.get("person_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: person_id".to_string()))?;
        
        Self::validate_uuid(person_id)?;
        
        // 验证分数类型
        let score_type = params.get("score_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: score_type".to_string()))?;
        
        if !["personal", "group", "class", "dormitory"].contains(&score_type) {
            return Err(AppError::InvalidInput(
                "分数类型无效，应为personal、group、class或dormitory".to_string()
            ));
        }
        
        // 验证分数值
        let value = params.get("value")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: value".to_string()))?;
        
        if value < 0 || value > 100 {
            return Err(AppError::InvalidInput("分数值应在0-100之间".to_string()));
        }
        
        // 验证评分原因
        let reason = params.get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少必填字段: reason".to_string()))?;
        
        if reason.trim().is_empty() {
            return Err(AppError::InvalidInput("评分原因不能为空".to_string()));
        }
        
        if reason.len() > 500 {
            return Err(AppError::InvalidInput("评分原因长度不能超过500个字符".to_string()));
        }
        
        Ok(())
    }
    
    /// 验证日期格式 (YYYY-MM-DD)
    fn validate_date(date_str: &str) -> Result<(), AppError> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| AppError::InvalidInput(format!("日期格式无效，应为YYYY-MM-DD: {}", date_str)))?;
        Ok(())
    }
    
    /// 验证时间格式 (HH:MM:SS)
    fn validate_time(time_str: &str) -> Result<(), AppError> {
        NaiveTime::parse_from_str(time_str, "%H:%M:%S")
            .map_err(|_| AppError::InvalidInput(format!("时间格式无效，应为HH:MM:SS: {}", time_str)))?;
        Ok(())
    }
    
    /// 验证UUID格式
    fn validate_uuid(uuid_str: &str) -> Result<(), AppError> {
        Uuid::parse_str(uuid_str)
            .map_err(|_| AppError::InvalidInput(format!("UUID格式无效: {}", uuid_str)))?;
        Ok(())
    }
}
```

### 4.2 数据库操作执行器

**文件位置：** `backend/src/api/ai_actions.rs`

```rust
use axum::{Extension, State, Json};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;
use crate::core::ai_action_validator::AIActionValidator;
use crate::utils::date_format::{parse_date, parse_time, format_date, format_time};

/// AI操作请求
#[derive(Debug, Deserialize)]
pub struct AIActionRequest {
    pub action_type: String,
    pub params: Option<Value>,
    pub batch: Option<bool>,
    pub items: Option<Vec<Value>>,
    pub reason: Option<String>,
}

/// AI操作响应
#[derive(Debug, serde::Serialize)]
pub struct AIActionResponse {
    pub success: bool,
    pub action_type: String,
    pub message: String,
    pub data: Option<Value>,
    pub error_code: Option<String>,
}

/// 执行AI操作
pub async fn execute_action(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AIActionRequest>,
) -> Result<Json<AIActionResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    // 检查权限
    let permission_manager = PermissionManager::new(pool.clone());
    let required_permission = get_required_permission(&req.action_type);
    permission_manager.require_permission(user_id, required_permission).await?;
    
    // 验证参数
    if let Some(params) = &req.params {
        validate_action_params(&req.action_type, params)?;
    }
    
    // 执行操作
    let result = match req.action_type.as_str() {
        "create_person" => execute_create_person(&pool, req.params.unwrap()).await,
        "create_persons_batch" => execute_create_persons_batch(&pool, req.items.unwrap()).await,
        "create_attendance" => execute_create_attendance(&pool, req.params.unwrap()).await,
        "create_attendances_batch" => execute_create_attendances_batch(&pool, req.items.unwrap()).await,
        "create_score" => execute_create_score(&pool, req.params.unwrap()).await,
        "create_scores_batch" => execute_create_scores_batch(&pool, req.items.unwrap()).await,
        _ => Err(AppError::InvalidInput(format!("未知的操作类型: {}", req.action_type))),
    };
    
    match result {
        Ok(data) => Ok(Json(AIActionResponse {
            success: true,
            action_type: req.action_type,
            message: "操作执行成功".to_string(),
            data: Some(data),
            error_code: None,
        })),
        Err(e) => Ok(Json(AIActionResponse {
            success: false,
            action_type: req.action_type,
            message: e.to_string(),
            data: None,
            error_code: Some(get_error_code(&e)),
        })),
    }
}

/// 获取操作所需的权限
fn get_required_permission(action_type: &str) -> &str {
    match action_type {
        "create_person" | "create_persons_batch" => "person.create",
        "create_attendance" | "create_attendances_batch" => "attendance.create",
        "create_score" | "create_scores_batch" => "score.create",
        "create_notice" => "notice.create",
        _ => "ai.chat",
    }
}

/// 验证操作参数
fn validate_action_params(action_type: &str, params: &Value) -> Result<(), AppError> {
    match action_type {
        "create_person" => AIActionValidator::validate_create_person(params),
        "create_attendance" => AIActionValidator::validate_create_attendance(params),
        "create_score" => AIActionValidator::validate_create_score(params),
        _ => Ok(()),
    }
}

/// 获取错误码
fn get_error_code(error: &AppError) -> String {
    match error {
        AppError::Auth(_) => "PERMISSION_DENIED".to_string(),
        AppError::InvalidInput(_) => "INVALID_INPUT".to_string(),
        AppError::Database(_) => "DATABASE_ERROR".to_string(),
        AppError::NotFound => "NOT_FOUND".to_string(),
        _ => "INTERNAL_ERROR".to_string(),
    }
}

/// 执行创建人员
async fn execute_create_person(
    pool: &PgPool,
    params: Value,
) -> Result<Value, AppError> {
    let mut tx = pool.begin().await?;
    
    let person_id = Uuid::new_v4();
    let name = params["name"].as_str().unwrap();
    let gender = params["gender"].as_i64().unwrap() as i16;
    let type_ = params["type"].as_str().unwrap();
    
    // 解析可选的日期
    let birthday = params.get("birthday")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .and_then(|s| parse_date(s).ok());
    
    let phone = params.get("phone").and_then(|v| v.as_str());
    let email = params.get("email").and_then(|v| v.as_str());
    
    // 插入persons表
    sqlx::query(
        "INSERT INTO persons (id, name, gender, birthday, phone, email, type)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(person_id)
    .bind(name)
    .bind(gender)
    .bind(birthday)
    .bind(phone)
    .bind(email)
    .bind(type_)
    .execute(&mut *tx)
    .await?;
    
    // 根据类型插入对应的子表
    match type_ {
        "student" => {
            let student_no = params["student_no"].as_str().unwrap();
            let class_id = params.get("class_id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| Uuid::parse_str(s).ok());
            let enrollment_date = params.get("enrollment_date")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| parse_date(s).ok());
            
            sqlx::query(
                "INSERT INTO students (person_id, student_no, class_id, enrollment_date, status)
                 VALUES ($1, $2, $3, $4, 'enrolled')"
            )
            .bind(person_id)
            .bind(student_no)
            .bind(class_id)
            .bind(enrollment_date)
            .execute(&mut *tx)
            .await?;
        }
        "teacher" => {
            let employee_no = params["employee_no"].as_str().unwrap();
            let department_id = params.get("department_id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| Uuid::parse_str(s).ok());
            let title = params.get("title").and_then(|v| v.as_str());
            let hire_date = params.get("hire_date")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| parse_date(s).ok());
            
            sqlx::query(
                "INSERT INTO teachers (person_id, employee_no, department_id, title, hire_date)
                 VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(person_id)
            .bind(employee_no)
            .bind(department_id)
            .bind(title)
            .bind(hire_date)
            .execute(&mut *tx)
            .await?;
        }
        "parent" => {
            let wechat_openid = params.get("wechat_openid").and_then(|v| v.as_str());
            let occupation = params.get("occupation").and_then(|v| v.as_str());
            
            sqlx::query(
                "INSERT INTO parents (person_id, wechat_openid, occupation)
                 VALUES ($1, $2, $3)"
            )
            .bind(person_id)
            .bind(wechat_openid)
            .bind(occupation)
            .execute(&mut *tx)
            .await?;
        }
        _ => {}
    }
    
    tx.commit().await?;
    
    Ok(serde_json::json!({
        "id": person_id.to_string(),
        "name": name,
        "type": type_
    }))
}

/// 执行批量创建人员
async fn execute_create_persons_batch(
    pool: &PgPool,
    items: Vec<Value>,
) -> Result<Value, AppError> {
    let mut tx = pool.begin().await?;
    let mut results = Vec::new();
    
    for (index, params) in items.into_iter().enumerate() {
        let result = match execute_create_person_single(&mut tx, params).await {
            Ok(data) => serde_json::json!({
                "success": true,
                "index": index,
                "data": data
            }),
            Err(e) => serde_json::json!({
                "success": false,
                "index": index,
                "error": e.to_string()
            }),
        };
        results.push(result);
    }
    
    tx.commit().await?;
    
    let success_count = results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count();
    
    Ok(serde_json::json!({
        "total": results.len(),
        "success_count": success_count,
        "failure_count": results.len() - success_count,
        "items": results
    }))
}

/// 执行单个创建人员（用于批量操作）
async fn execute_create_person_single(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: Value,
) -> Result<Value, AppError> {
    // 这里复用execute_create_person的逻辑，但使用传入的tx
    // 简化版实现...
    Ok(serde_json::json!({}))
}

/// 执行创建考勤
async fn execute_create_attendance(
    pool: &PgPool,
    params: Value,
) -> Result<Value, AppError> {
    let person_id = Uuid::parse_str(params["person_id"].as_str().unwrap())?;
    let date = parse_date(params["date"].as_str().unwrap())?;
    let status = params["status"].as_str().unwrap();
    let time = params.get("time")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .and_then(|s| parse_time(s).ok());
    let remark = params.get("remark").and_then(|v| v.as_str());
    
    let attendance_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO attendances (id, person_id, date, status, time, remark)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(attendance_id)
    .bind(person_id)
    .bind(date)
    .bind(status)
    .bind(time)
    .bind(remark)
    .execute(pool)
    .await?;
    
    Ok(serde_json::json!({
        "id": attendance_id.to_string(),
        "person_id": person_id.to_string(),
        "date": format_date(&date),
        "status": status
    }))
}

/// 执行批量创建考勤
async fn execute_create_attendances_batch(
    pool: &PgPool,
    items: Vec<Value>,
) -> Result<Value, AppError> {
    let mut tx = pool.begin().await?;
    let mut results = Vec::new();
    
    for (index, params) in items.into_iter().enumerate() {
        let result = match execute_create_attendance_single(&mut tx, params).await {
            Ok(data) => serde_json::json!({
                "success": true,
                "index": index,
                "data": data
            }),
            Err(e) => serde_json::json!({
                "success": false,
                "index": index,
                "error": e.to_string()
            }),
        };
        results.push(result);
    }
    
    tx.commit().await?;
    
    let success_count = results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count();
    
    Ok(serde_json::json!({
        "total": results.len(),
        "success_count": success_count,
        "failure_count": results.len() - success_count,
        "items": results
    }))
}

/// 执行单个创建考勤（用于批量操作）
async fn execute_create_attendance_single(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: Value,
) -> Result<Value, AppError> {
    Ok(serde_json::json!({}))
}

/// 执行创建成绩
async fn execute_create_score(
    pool: &PgPool,
    params: Value,
) -> Result<Value, AppError> {
    let person_id = Uuid::parse_str(params["person_id"].as_str().unwrap())?;
    let score_type = params["score_type"].as_str().unwrap();
    let value = params["value"].as_i64().unwrap() as i32;
    let reason = params["reason"].as_str().unwrap();
    let group_id = params.get("group_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(s).ok());
    
    let score_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO scores (id, person_id, score_type, value, reason, group_id)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(score_id)
    .bind(person_id)
    .bind(score_type)
    .bind(value)
    .bind(reason)
    .bind(group_id)
    .execute(pool)
    .await?;
    
    Ok(serde_json::json!({
        "id": score_id.to_string(),
        "person_id": person_id.to_string(),
        "value": value,
        "reason": reason
    }))
}

/// 执行批量创建成绩
async fn execute_create_scores_batch(
    pool: &PgPool,
    items: Vec<Value>,
) -> Result<Value, AppError> {
    let mut tx = pool.begin().await?;
    let mut results = Vec::new();
    
    for (index, params) in items.into_iter().enumerate() {
        let result = match execute_create_score_single(&mut tx, params).await {
            Ok(data) => serde_json::json!({
                "success": true,
                "index": index,
                "data": data
            }),
            Err(e) => serde_json::json!({
                "success": false,
                "index": index,
                "error": e.to_string()
            }),
        };
        results.push(result);
    }
    
    tx.commit().await?;
    
    let success_count = results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count();
    
    Ok(serde_json::json!({
        "total": results.len(),
        "success_count": success_count,
        "failure_count": results.len() - success_count,
        "items": results
    }))
}

/// 执行单个创建成绩（用于批量操作）
async fn execute_create_score_single(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: Value,
) -> Result<Value, AppError> {
    Ok(serde_json::json!({}))
}
```

### 4.3 完整的AI交互流程

```
用户输入
    ↓
前端发送消息到后端
    ↓
后端接收请求
    ↓
构建系统提示词（包含用户权限、上下文数据）
    ↓
调用AI模型
    ↓
解析AI响应
    ↓
检测[AI_ACTION]标记
    ↓
有操作？→ 是 → 提取操作参数
    ↓           ↓
    否       验证操作权限
    ↓           ↓
返回自然语言  验证操作参数
    ↓           ↓
    结束      执行数据库操作
                ↓
            返回操作结果
                ↓
            构建下一步提示词
                ↓
            再次调用AI模型（可选）
                ↓
            返回最终响应
```

### 4.2 AI编排器工作流程

**文件位置：** `backend/src/core/ai_orchestrator.rs`

**职责：**
1. 管理多步骤操作会话
2. 处理步骤间的数据依赖
3. 维护操作上下文
4. 处理错误和回滚
5. 建议下一步操作

**核心方法：**
- `start_session()`: 开始新会话
- `execute_step()`: 执行单个步骤
- `resolve_dependencies()`: 解析步骤依赖
- `rollback_session()`: 回滚会话
- `suggest_next_steps()`: 建议下一步操作

### 4.3 操作结果反馈给AI

每次操作执行完成后，将结果以结构化方式反馈给AI：

```
## 操作执行结果

操作类型: create_person
状态: 成功
数据:
{
  "id": "uuid-string",
  "name": "张三",
  "type": "student"
}

您现在可以：
1. 继续创建更多学生
2. 为该学生创建考勤记录
3. 将该学生添加到小组

请告诉我您接下来想做什么？
```

## 5. 页面上下文信息获取

### 5.1 上下文数据策略

**原则：**
1. 不发送全部数据，只发送关键信息
2. 优先发送统计数据而非详细列表
3. 发送当前页面的筛选条件和状态
4. 限制数据量，避免token溢出
5. 优先发送最新的数据

### 5.2 数据库类型映射与格式化

#### 5.2.1 数据类型映射表

| JSON字段类型 | PostgreSQL类型 | 格式化要求 | 示例 |
|-------------|---------------|-----------|------|
| string | VARCHAR/TEXT | 直接使用，确保长度不超过数据库限制 | "张三" |
| number/integer | INTEGER/SMALLINT | 确保在有效范围内 | 1, 100 |
| boolean | BOOLEAN | 使用true/false | true |
| date | DATE | 格式：YYYY-MM-DD | "2026-03-07" |
| time | TIME | 格式：HH:MM:SS | "08:30:00" |
| datetime | TIMESTAMP | 格式：YYYY-MM-DDTHH:MM:SSZ (RFC3339) | "2026-03-07T08:30:00Z" |
| uuid | UUID | 标准UUID格式 | "550e8400-e29b-41d4-a716-446655440000" |
| array | ARRAY | JSON数组格式 | ["a", "b", "c"] |
| object | JSONB | JSON对象格式 | {"key": "value"} |

#### 5.2.2 时间格式化函数

**后端Rust实现：**
```rust
// backend/src/utils/date_format.rs

use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Utc};

/// 格式化日期为YYYY-MM-DD字符串
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// 格式化时间为HH:MM:SS字符串
pub fn format_time(time: &NaiveTime) -> String {
    time.format("%H:%M:%S").to_string()
}

/// 格式化日期时间为RFC3339字符串
pub fn format_datetime(datetime: &NaiveDateTime) -> String {
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// 解析YYYY-MM-DD日期字符串
pub fn parse_date(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("日期格式错误，应为YYYY-MM-DD: {}", e))
}

/// 解析HH:MM:SS时间字符串
pub fn parse_time(time_str: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(time_str, "%H:%M:%S")
        .map_err(|e| format!("时间格式错误，应为HH:MM:SS: {}", e))
}

/// 解析日期时间字符串（支持多种格式）
pub fn parse_datetime(datetime_str: &str) -> Result<NaiveDateTime, String> {
    // 尝试RFC3339格式
    if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%SZ") {
        return Ok(dt);
    }
    // 尝试YYYY-MM-DD HH:MM:SS格式
    if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    Err("日期时间格式错误".to_string())
}
```

**前端TypeScript实现：**
```typescript
// frontend/src/utils/dateFormat.ts

/**
 * 格式化日期为YYYY-MM-DD字符串
 */
export function formatDate(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/**
 * 格式化时间为HH:MM:SS字符串
 */
export function formatTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  const hours = String(d.getHours()).padStart(2, '0');
  const minutes = String(d.getMinutes()).padStart(2, '0');
  const seconds = String(d.getSeconds()).padStart(2, '0');
  return `${hours}:${minutes}:${seconds}`;
}

/**
 * 格式化日期时间为RFC3339字符串
 */
export function formatDateTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toISOString();
}

/**
 * 验证日期格式是否为YYYY-MM-DD
 */
export function isValidDate(dateStr: string): boolean {
  const regex = /^\d{4}-\d{2}-\d{2}$/;
  if (!regex.test(dateStr)) return false;
  
  const date = new Date(dateStr);
  return date instanceof Date && !isNaN(date.getTime());
}

/**
 * 验证时间格式是否为HH:MM:SS
 */
export function isValidTime(timeStr: string): boolean {
  const regex = /^\d{2}:\d{2}:\d{2}$/;
  if (!regex.test(timeStr)) return false;
  
  const [hours, minutes, seconds] = timeStr.split(':').map(Number);
  return hours >= 0 && hours < 24 && 
         minutes >= 0 && minutes < 60 && 
         seconds >= 0 && seconds < 60;
}
```

### 5.3 后端上下文数据获取函数

#### 5.3.1 上下文数据提供者模块

**文件位置：** `backend/src/api/ai_context_provider.rs`

```rust
use axum::{Extension, State};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::api::routes::AppState;

/// 页面上下文数据请求
#[derive(Debug, Deserialize)]
pub struct PageContextRequest {
    pub page: String,
    pub path: String,
    pub params: Option<serde_json::Value>,
    pub query: Option<serde_json::Value>,
}

/// 统一的页面上下文响应
#[derive(Debug, Serialize)]
pub struct PageContext {
    pub page: String,
    pub data: serde_json::Value,
}

/// 主要的上下文获取函数
pub async fn get_page_context(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PageContextRequest>,
) -> Result<Json<PageContext>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let data = match req.page.as_str() {
        "dashboard" => get_dashboard_context(&pool, user_id).await?,
        "person" => get_person_context(&pool, user_id, req.params, req.query).await?,
        "attendance" => get_attendance_context(&pool, user_id, req.params, req.query).await?,
        "notice" => get_notice_context(&pool, user_id).await?,
        "class" => get_class_context(&pool, user_id).await?,
        "group" => get_group_context(&pool, user_id, req.params, req.query).await?,
        _ => serde_json::json!({"page": req.page}),
    };
    
    Ok(Json(PageContext {
        page: req.page,
        data,
    }))
}

/// 获取Dashboard上下文
async fn get_dashboard_context(
    pool: &PgPool,
    _user_id: Uuid,
) -> Result<serde_json::Value, AppError> {
    // 今日考勤统计
    let today = chrono::Utc::now().date_naive();
    
    let attendance_stats = sqlx::query!(
        "SELECT 
            COUNT(*) as total,
            SUM(CASE WHEN status = 'present' THEN 1 ELSE 0 END) as present,
            SUM(CASE WHEN status = 'absent' THEN 1 ELSE 0 END) as absent,
            SUM(CASE WHEN status = 'late' THEN 1 ELSE 0 END) as late
         FROM attendances 
         WHERE date = $1",
        today
    )
    .fetch_one(pool)
    .await?;
    
    // 待处理通知数
    let pending_notices = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM notices WHERE created_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(pool)
    .await?;
    
    // 本月积分变化
    let month_start = today - chrono::Duration::days(today.day() as i64 - 1);
    let month_score_change = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(value), 0) FROM scores 
         WHERE created_at >= $1",
        month_start
    )
    .fetch_one(pool)
    .await?;
    
    Ok(serde_json::json!({
        "page": "dashboard",
        "summary": {
            "today_attendance": {
                "total": attendance_stats.total.unwrap_or(0),
                "present": attendance_stats.present.unwrap_or(0),
                "absent": attendance_stats.absent.unwrap_or(0),
                "late": attendance_stats.late.unwrap_or(0)
            },
            "pending_notices": pending_notices.unwrap_or(0),
            "month_score_change": format!("{:+}", month_score_change.unwrap_or(0))
        }
    }))
}

/// 获取人员管理上下文
async fn get_person_context(
    pool: &PgPool,
    _user_id: Uuid,
    _params: Option<serde_json::Value>,
    _query: Option<serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    // 人员总数
    let total = sqlx::query_scalar!("SELECT COUNT(*) FROM persons")
        .fetch_one(pool)
        .await?;
    
    // 信息不完整的人员数
    let incomplete_info = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM persons 
         WHERE phone IS NULL OR email IS NULL OR birthday IS NULL"
    )
    .fetch_one(pool)
    .await?;
    
    // 无手机号的人员数
    let no_phone = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM persons WHERE phone IS NULL"
    )
    .fetch_one(pool)
    .await?;
    
    // 无邮箱的人员数
    let no_email = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM persons WHERE email IS NULL"
    )
    .fetch_one(pool)
    .await?;
    
    // 获取班级列表（简化）
    let classes = sqlx::query!(
        "SELECT id, name, 
                (SELECT COUNT(*) FROM students WHERE class_id = classes.id) as student_count
         FROM classes 
         LIMIT 5"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(serde_json::json!({
        "page": "person",
        "stats": {
            "total": total.unwrap_or(0),
            "incomplete_info": incomplete_info.unwrap_or(0),
            "no_phone": no_phone.unwrap_or(0),
            "no_email": no_email.unwrap_or(0)
        },
        "classes": classes.into_iter().map(|c| {
            serde_json::json!({
                "id": c.id.to_string(),
                "name": c.name,
                "student_count": c.student_count.unwrap_or(0)
            })
        }).collect::<Vec<_>>()
    }))
}

/// 获取考勤管理上下文
async fn get_attendance_context(
    pool: &PgPool,
    _user_id: Uuid,
    _params: Option<serde_json::Value>,
    _query: Option<serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    let today = chrono::Utc::now().date_naive();
    
    // 今日考勤统计
    let today_stats = sqlx::query!(
        "SELECT 
            COUNT(*) as total,
            SUM(CASE WHEN status = 'present' THEN 1 ELSE 0 END) as present,
            SUM(CASE WHEN status = 'absent' THEN 1 ELSE 0 END) as absent,
            SUM(CASE WHEN status = 'late' THEN 1 ELSE 0 END) as late,
            SUM(CASE WHEN status = 'early_leave' THEN 1 ELSE 0 END) as early_leave,
            SUM(CASE WHEN status = 'excused' THEN 1 ELSE 0 END) as excused
         FROM attendances 
         WHERE date = $1",
        today
    )
    .fetch_one(pool)
    .await?;
    
    // 获取总学生数（用于计算未记录数）
    let total_students = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM students"
    )
    .fetch_one(pool)
    .await?;
    
    let recorded = today_stats.total.unwrap_or(0);
    let unrecorded = total_students.unwrap_or(0) - recorded;
    
    Ok(serde_json::json!({
        "page": "attendance",
        "current_date": format_date(&today),
        "today_stats": {
            "total": recorded,
            "present": today_stats.present.unwrap_or(0),
            "absent": today_stats.absent.unwrap_or(0),
            "late": today_stats.late.unwrap_or(0),
            "early_leave": today_stats.early_leave.unwrap_or(0),
            "excused": today_stats.excused.unwrap_or(0)
        },
        "unrecorded": unrecorded
    }))
}

/// 获取通知公告上下文
async fn get_notice_context(
    pool: &PgPool,
    _user_id: Uuid,
) -> Result<serde_json::Value, AppError> {
    // 最近的通知
    let recent_notices = sqlx::query!(
        "SELECT id, title, created_at, author_id, 
                LEFT(content, 100) as summary
         FROM notices 
         ORDER BY created_at DESC 
         LIMIT 3"
    )
    .fetch_all(pool)
    .await?;
    
    // 通知统计
    let total = sqlx::query_scalar!("SELECT COUNT(*) FROM notices")
        .fetch_one(pool)
        .await?;
    
    let week_ago = chrono::Utc::now() - chrono::Duration::days(7);
    let this_week = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM notices WHERE created_at >= $1",
        week_ago
    )
    .fetch_one(pool)
    .await?;
    
    Ok(serde_json::json!({
        "page": "notice",
        "recent_notices": recent_notices.into_iter().map(|n| {
            serde_json::json!({
                "id": n.id.to_string(),
                "title": n.title,
                "created_at": format_datetime(&n.created_at.naive_utc()),
                "summary": n.summary.unwrap_or_default()
            })
        }).collect::<Vec<_>>(),
        "stats": {
            "total": total.unwrap_or(0),
            "this_week": this_week.unwrap_or(0)
        }
    }))
}

/// 获取班级管理上下文
async fn get_class_context(
    pool: &PgPool,
    _user_id: Uuid,
) -> Result<serde_json::Value, AppError> {
    // 班级列表（简化）
    let classes = sqlx::query!(
        "SELECT c.id, c.name, c.grade, 
                p.name as teacher_name,
                (SELECT COUNT(*) FROM students WHERE class_id = c.id) as student_count
         FROM classes c
         LEFT JOIN persons p ON c.teacher_id = p.id
         LIMIT 5"
    )
    .fetch_all(pool)
    .await?;
    
    // 统计
    let total_classes = sqlx::query_scalar!("SELECT COUNT(*) FROM classes")
        .fetch_one(pool)
        .await?;
    
    let no_teacher = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM classes WHERE teacher_id IS NULL"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(serde_json::json!({
        "page": "class",
        "classes": classes.into_iter().map(|c| {
            serde_json::json!({
                "id": c.id.to_string(),
                "name": c.name,
                "grade": c.grade,
                "teacher_name": c.teacher_name,
                "student_count": c.student_count.unwrap_or(0)
            })
        }).collect::<Vec<_>>(),
        "stats": {
            "total_classes": total_classes.unwrap_or(0),
            "no_teacher": no_teacher.unwrap_or(0)
        }
    }))
}

/// 获取小组管理上下文
async fn get_group_context(
    pool: &PgPool,
    _user_id: Uuid,
    _params: Option<serde_json::Value>,
    _query: Option<serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    // 小组列表（简化）
    let groups = sqlx::query!(
        "SELECT g.id, g.name, g.class_id, c.name as class_name,
                (SELECT COUNT(*) FROM group_members WHERE group_id = g.id) as member_count
         FROM groups g
         LEFT JOIN classes c ON g.class_id = c.id
         LIMIT 5"
    )
    .fetch_all(pool)
    .await?;
    
    // 统计
    let total_groups = sqlx::query_scalar!("SELECT COUNT(*) FROM groups")
        .fetch_one(pool)
        .await?;
    
    Ok(serde_json::json!({
        "page": "group",
        "groups": groups.into_iter().map(|g| {
            serde_json::json!({
                "id": g.id.to_string(),
                "name": g.name,
                "class_name": g.class_name,
                "member_count": g.member_count.unwrap_or(0)
            })
        }).collect::<Vec<_>>(),
        "stats": {
            "total_groups": total_groups.unwrap_or(0)
        }
    }))
}
```

### 5.4 各页面上下文数据详细说明

#### 5.2.1 Dashboard（仪表盘）

**发送给AI的数据：**
```json
{
  "page": "dashboard",
  "summary": {
    "today_attendance": {
      "total": 150,
      "present": 145,
      "absent": 3,
      "late": 2
    },
    "pending_notices": 2,
    "month_score_change": "+125",
    "recent_activities": [
      "张三考勤迟到",
      "高一(1)班积分+5",
      "李四新增"
    ]
  },
  "quick_stats": {
    "total_students": 500,
    "total_teachers": 50,
    "total_classes": 12
  }
}
```

**AI分析方向：**
- 考勤异常提醒
- 待处理事项提示
- 数据趋势建议

#### 5.2.2 PersonView（人员管理）

**发送给AI的数据：**
```json
{
  "page": "person",
  "current_filter": {
    "type": "student",
    "class_id": "uuid-string",
    "search": ""
  },
  "stats": {
    "total": 45,
    "incomplete_info": 8,
    "no_phone": 5,
    "no_email": 3
  },
  "recent_changes": [
    { "action": "create", "name": "张三", "time": "5分钟前" },
    { "action": "update", "name": "李四", "time": "10分钟前" }
  ],
  "classes": [
    { "id": "uuid-1", "name": "高一(1)班", "student_count": 45 },
    { "id": "uuid-2", "name": "高一(2)班", "student_count": 42 }
  ]
}
```

**AI分析方向：**
- 提示完善缺失的学生信息
- 建议批量导入人员
- 提醒重复数据检查

#### 5.2.3 AttendanceView（考勤管理）

**发送给AI的数据：**
```json
{
  "page": "attendance",
  "current_date": "2026-03-07",
  "current_filter": {
    "class_id": "uuid-string",
    "status": ""
  },
  "today_stats": {
    "total": 45,
    "present": 40,
    "absent": 3,
    "late": 2,
    "early_leave": 0,
    "excused": 0
  },
  "unrecorded": 5,
  "recent_abnormal": [
    { "name": "王五", "status": "late", "count": 3 },
    { "name": "赵六", "status": "absent", "count": 2 }
  ]
}
```

**AI分析方向：**
- 提示未记录考勤的学生
- 提醒频繁缺勤的学生
- 建议批量录入考勤

#### 5.2.4 NoticeView（通知公告）

**发送给AI的数据：**
```json
{
  "page": "notice",
  "recent_notices": [
    {
      "id": "uuid-1",
      "title": "期中考试通知",
      "created_at": "2026-03-05",
      "author": "张老师",
      "summary": "下周一进行期中考试，请同学们做好准备"
    },
    {
      "id": "uuid-2",
      "title": "家长会通知",
      "created_at": "2026-03-03",
      "author": "李校长",
      "summary": "本周六下午2点召开家长会"
    }
  ],
  "stats": {
    "total": 15,
    "this_week": 3,
    "unread": 2
  },
  "available_targets": [
    { "type": "school", "name": "全校" },
    { "type": "class", "name": "高一(1)班" },
    { "type": "department", "name": "教务处" }
  ]
}
```

**AI分析方向：**
- 总结最新公告内容
- 建议发布新公告
- 提醒重要公告

#### 5.2.5 ClassView（班级管理）

**发送给AI的数据：**
```json
{
  "page": "class",
  "classes": [
    {
      "id": "uuid-1",
      "name": "高一(1)班",
      "grade": 10,
      "student_count": 45,
      "teacher_name": "张老师",
      "avg_score": 85
    },
    {
      "id": "uuid-2",
      "name": "高一(2)班",
      "grade": 10,
      "student_count": 42,
      "teacher_name": "李老师",
      "avg_score": 82
    }
  ],
  "stats": {
    "total_classes": 12,
    "no_teacher": 1,
    "low_score_classes": 2
  }
}
```

**AI分析方向：**
- 提醒无班主任的班级
- 建议关注低分班级
- 班级对比分析

#### 5.2.6 GroupView（小组管理）

**发送给AI的数据：**
```json
{
  "page": "group",
  "current_class": "高一(1)班",
  "groups": [
    {
      "id": "uuid-1",
      "name": "第一组",
      "member_count": 6,
      "total_score": 125,
      "rank": 1
    },
    {
      "id": "uuid-2",
      "name": "第二组",
      "member_count": 6,
      "total_score": 98,
      "rank": 3
    }
  ],
  "stats": {
    "total_groups": 8,
    "this_month_change": "+256"
  }
}
```

**AI分析方向：**
- 小组积分变化分析
- 落后小组改进建议
- 小组活动建议

### 5.3 前端上下文获取实现

**文件位置：** `frontend/src/composables/usePageContext.ts`

```typescript
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'

export function usePageContext() {
  const route = useRoute()
  const pageData = ref<any>(null)

  const getContext = async () => {
    const path = route.path
    
    if (path.includes('person')) {
      return await getPersonContext()
    } else if (path.includes('attendance')) {
      return await getAttendanceContext()
    } else if (path.includes('notice')) {
      return await getNoticeContext()
    } else if (path.includes('class')) {
      return await getClassContext()
    } else if (path.includes('group')) {
      return await getGroupContext()
    } else {
      return await getDashboardContext()
    }
  }

  return {
    getContext,
    pageData
  }
}
```

## 6. 提示词设计

### 6.1 AI聊天系统提示词（完整版）

```
你是一个专业的学校管理系统AI助手。你的职责是帮助用户完成学校管理相关的任务。

## 用户信息
- 用户ID: {{user_id}}
- 用户角色: {{user_role}}
- 用户权限: {{user_permissions}}

## 当前页面上下文
{{page_context}}

## 可用操作
根据用户权限，你可以执行以下操作：

### 数据查询操作（仅读取）
- get_persons: 查询人员列表，参数：type(可选), search(可选), class_id(可选), limit(可选)
- get_attendances: 查询考勤记录，参数：person_id(可选), date(可选), status(可选), class_id(可选)
- get_classes: 查询班级列表，参数：grade(可选)
- get_groups: 查询小组列表，参数：class_id(可选)
- get_departments: 查询部门列表
- get_notices: 查询公告列表，参数：limit(可选), sort_by(可选), order(可选)

### 数据操作（需要相应权限）
- create_person: 创建单个人员
- create_persons_batch: 批量创建人员
- create_attendance: 创建单条考勤记录
- create_attendances_batch: 批量创建考勤记录
- create_score: 创建单条成绩记录
- create_scores_batch: 批量创建成绩记录
- create_notice: 创建公告

## 操作格式要求

当你需要执行操作时，请使用以下格式：

### 单条操作
```
[AI_ACTION]
{
  "action_type": "操作名称",
  "params": {参数对象},
  "reason": "执行此操作的原因"
}
[/AI_ACTION]
```

### 批量操作
```
[AI_ACTION]
{
  "action_type": "操作名称_batch",
  "batch": true,
  "items": [参数对象数组],
  "reason": "执行此批量操作的原因"
}
[/AI_ACTION]
```

### 多步骤操作
当需要多个步骤完成任务时，可以分步骤执行：

1. 首先查询需要的信息
2. 使用查询结果执行下一步操作
3. 每步完成后等待确认再继续

格式同单条操作，但需要说明后续步骤计划。

## 操作执行规则

1. **先查询后操作**：在执行创建操作前，如果需要选择班级、部门等关联数据，请先使用get_*操作查询可用选项
2. **先确认后执行**：在执行创建、更新等操作前，如果用户指令不够明确，请先询问用户确认细节
3. **权限检查**：始终检查用户是否有执行操作的权限，如果没有，礼貌地告知用户
4. **数据验证**：确保操作参数完整且格式正确，如有缺失请询问用户补充
5. **批量操作优先**：当用户要求创建多个类似记录时，优先使用批量操作接口
6. **结果反馈**：操作执行完成后，用简洁明了的语言向用户反馈结果
7. **多步骤协调**：复杂任务可以分多步骤执行，每步完成后告知用户进度

## 上下文数据分析

根据当前页面的上下文数据，你可以：
1. 分析数据异常并提醒用户
2. 给出改进建议
3. 预测用户可能的下一步操作
4. 主动提供有用的信息

## 响应风格

1. 使用中文回复
2. 回答要简洁、准确、专业
3. 对于复杂操作，分步骤说明
4. 可以使用Markdown格式来格式化回复，但避免过度使用
5. 友好、耐心、乐于助人
```

### 6.2 AI助手提示系统提示词（非对话模式，完整版）

```
你是一个学校管理系统的智能助手。现在你处于页面提示模式。

## 任务
根据用户当前页面的信息，分析数据，给出一条简短、有用、可操作的建议。

## 当前页面详细信息
{{page_context}}

## 用户权限
{{user_permissions}}

## 分析指南

请按以下步骤分析：
1. 首先查看页面统计数据
2. 识别异常或需要关注的情况
3. 考虑用户权限范围内可以执行的操作
4. 给出具体、可操作的建议

## 建议方向

根据不同页面，建议可以是：

### Dashboard（仪表盘）
- 考勤异常提醒："今日有3名学生缺勤，建议查看详情"
- 待办事项提醒："有2条待处理通知，请及时处理"
- 数据趋势："本月积分增长良好，继续保持"

### PersonView（人员管理）
- 信息完善："有8名学生信息不完整，建议补充"
- 批量操作："可以批量导入新生，提高效率"
- 数据质量："发现5名学生未填手机号"

### AttendanceView（考勤管理）
- 未记录提醒："还有5名学生未记录考勤"
- 异常关注："王五本周已迟到3次，建议关注"
- 批量建议："可以批量录入今天的考勤"

### NoticeView（通知公告）
- 总结公告："最新通知：期中考试将于下周一进行"
- 发布建议："可以发布关于家长会的提醒"
- 重要提醒："有2条重要公告未读"

### ClassView（班级管理）
- 班主任提醒："高一(3)班暂无班主任，请安排"
- 成绩关注："高一(2)班平均分较低，建议关注"
- 人员配置："建议为新班级分配教师"

### GroupView（小组管理）
- 积分分析："第二组积分落后，建议组织活动提升"
- 活动建议："可以组织小组竞赛，提高积极性"
- 成员调整："部分小组人数不均，可以调整"

## 回答要求

1. **字数限制**：回答必须非常简短，不超过50个字
2. **单条建议**：只给出一条最关键、最有用的建议
3. **禁止Markdown**：不要使用任何Markdown格式
4. **仅建议**：不要执行任何操作，只给出建议
5. **使用中文**：必须使用中文回复
6. **具体可操作**：建议要具体、有可操作性，不要太笼统
7. **结合数据**：建议要基于提供的上下文数据
8. **权限范围内**：建议要在用户权限范围内

## 好的示例
✓ "这个班级有3名学生缺勤，建议查看详情"
✓ "有8名学生信息不完整，建议补充"
✓ "王五本周已迟到3次，建议关注"
✓ "最新通知：期中考试将于下周一进行"
✓ "高一(3)班暂无班主任，请安排"

## 不好的示例
✗ "这个页面看起来不错"（太笼统）
✗ "你可以做很多事情"（不具体）
✗ "**有3名学生缺勤**"（使用Markdown）
✗ "建议创建考勤记录"（没有结合数据）
```

## 7. AI助手提示功能设计

### 7.1 组件位置

AI助手提示组件将集成在每个页面的顶部，位于面包屑导航下方，页面内容卡片上方。

### 7.2 样式规范

```css
/* frontend/src/styles/ai-assistant.css */

.ai-assistant {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  margin-bottom: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 8px;
  color: white;
  font-size: 14px;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.2);
  animation: slideIn 0.3s ease-out;
}

.ai-assistant-icon {
  margin-right: 12px;
  font-size: 18px;
}

.ai-assistant-content {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ai-assistant-close {
  margin-left: 12px;
  cursor: pointer;
  opacity: 0.8;
  transition: opacity 0.2s;
}

.ai-assistant-close:hover {
  opacity: 1;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

### 7.3 组件结构

```vue
<!-- frontend/src/components/AIAssistant.vue -->
<template>
  <div v-if="visible" class="ai-assistant">
    <div class="ai-assistant-icon">💡</div>
    <div class="ai-assistant-content">{{ suggestion }}</div>
    <div class="ai-assistant-close" @click="close">×</div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { aiApi } from '../api'
import { usePageContext } from '../composables/usePageContext'

const visible = ref(false)
const suggestion = ref('')
const loading = ref(false)
const route = useRoute()
const { getContext } = usePageContext()

const fetchSuggestion = async () => {
  if (loading.value) return
  
  loading.value = true
  try {
    const pageContext = await getContext()
    const response = await aiApi.getAssistantSuggestion({
      page_context: pageContext,
      path: route.path,
      name: route.name as string
    })
    
    if (response.data && response.data.suggestion) {
      suggestion.value = response.data.suggestion
      visible.value = true
    } else {
      visible.value = false
    }
  } catch (error) {
    console.error('获取AI建议失败:', error)
    visible.value = false
  } finally {
    loading.value = false
  }
}

const close = () => {
  visible.value = false
}

// 路由变化时重新获取建议
watch(() => route.path, () => {
  fetchSuggestion()
}, { immediate: true })

// 也可以定时刷新（可选）
// onMounted(() => {
//   setInterval(fetchSuggestion, 5 * 60 * 1000) // 5分钟刷新一次
// })
</script>

<style scoped>
@import '../styles/ai-assistant.css';
</style>
```

## 8. 后端API设计

### 8.1 新增API接口

| 接口路径 | 方法 | 功能描述 |
|---------|------|---------|
| `/api/ai/actions` | POST | 执行AI操作 |
| `/api/ai/actions/multi-step` | POST | 执行多步骤AI操作 |
| `/api/ai/assistant/suggestion` | POST | 获取AI助手建议 |
| `/api/ai/actions/batch` | POST | 批量执行AI操作 |
| `/api/ai/actions/available` | GET | 获取可用操作列表 |
| `/api/ai/context` | POST | 获取页面上下文数据 |

### 8.2 AI操作执行流程详解

```
1. 接收AI操作请求
   ↓
2. 解析请求头和请求体
   ↓
3. 验证请求格式
   ├─ 检查必需字段
   ├─ 验证数据类型
   └─ 验证参数范围
   ↓
4. 检查用户权限
   ├─ 获取用户权限列表
   ├─ 检查操作所需权限
   └─ 权限不足返回错误
   ↓
5. 验证操作参数
   ├─ 检查必需参数
   ├─ 验证数据格式
   ├─ 验证外键存在
   └─ 验证业务规则
   ↓
6. 处理多步骤依赖
   ├─ 检查依赖步骤
   ├─ 获取依赖数据
   └─ 替换模板变量
   ↓
7. 执行数据库操作
   ├─ 开始事务
   ├─ 执行单条/批量操作
   ├─ 提交/回滚事务
   └─ 返回操作结果
   ↓
8. 构建响应
   ├─ 格式化成功/错误响应
   ├─ 添加建议的下一步操作
   └─ 返回给AI
```

### 8.3 批量操作处理

- 使用数据库事务确保数据一致性
- 支持部分失败不影响其他操作（可选）
- 提供详细的执行报告
- 支持事务回滚选项

### 8.4 多步骤操作会话管理

**会话数据结构：**
```json
{
  "session_id": "uuid",
  "user_id": "uuid",
  "created_at": "timestamp",
  "status": "in_progress",
  "steps": [
    {
      "step_id": "step-1",
      "action_type": "get_classes",
      "status": "completed",
      "result": {...},
      "executed_at": "timestamp"
    },
    {
      "step_id": "step-2",
      "action_type": "create_person",
      "status": "pending",
      "depends_on": "step-1"
    }
  ]
}
```

## 9. 相关文件清单

### 后端新增/修改文件

| 文件路径 | 操作 | 说明 |
|---------|------|------|
| `backend/src/api/ai_actions.rs` | 修改 | AI操作执行模块 |
| `backend/src/api/ai_data.rs` | 修改 | AI数据查询模块 |
| `backend/src/api/ai_enhanced.rs` | 修改 | 增强版AI聊天 |
| `backend/src/api/ai_assistant.rs` | 新增 | AI助手提示模块 |
| `backend/src/api/ai_context_provider.rs` | 新增 | AI上下文数据提供者 |
| `backend/src/models/ai_action.rs` | 新增 | AI操作数据模型 |
| `backend/src/core/ai_action_validator.rs` | 新增 | AI操作验证器 |
| `backend/src/core/ai_orchestrator.rs` | 新增 | AI编排器 |

### 前端新增/修改文件

| 文件路径 | 操作 | 说明 |
|---------|------|------|
| `frontend/src/api/ai.ts` | 修改 | AI API接口 |
| `frontend/src/components/AIAssistant.vue` | 新增 | AI助手提示组件 |
| `frontend/src/components/AIActionExecutor.vue` | 新增 | AI操作执行器 |
| `frontend/src/store/ai.ts` | 新增 | AI状态管理 |
| `frontend/src/styles/ai-assistant.css` | 新增 | AI助手样式 |
| `frontend/src/views/AIView.vue` | 修改 | AI对话页面 |
| `frontend/src/composables/usePageContext.ts` | 新增 | 页面上下文获取 |

## 10. 开发步骤

1. **阶段一：后端基础架构**
   - 创建AI操作数据模型
   - 实现AI操作验证器
   - 实现AI编排器
   - 重构AI操作执行模块

2. **阶段二：页面上下文数据**
   - 实现AI上下文数据提供者
   - 为每个页面实现数据提取逻辑
   - 实现数据精简和优化

3. **阶段三：数据库操作实现**
   - 实现人员创建操作（单条/批量）
   - 实现考勤创建操作（单条/批量）
   - 实现成绩创建操作（单条/批量）
   - 实现查询操作

4. **阶段四：AI助手功能**
   - 创建AI助手提示后端接口
   - 实现AI助手系统提示词
   - 创建AI助手前端组件
   - 集成为各页面

5. **阶段五：前端集成**
   - 更新AI API接口
   - 修改AI对话页面
   - 集成AI助手组件
   - 实现页面上下文获取

6. **阶段六：测试验证**
   - 单元测试
   - 集成测试
   - 用户验收测试

## 11. AI消息格式化与发送函数

### 11.1 消息发送函数设计

为了确保发送给AI的消息格式统一、数据完整，我们使用专门的函数来构建和发送消息。

#### 11.1.1 后端消息构建函数

**文件位置：** `backend/src/api/ai_message_builder.rs`

```rust
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::api::ai_context_provider::{get_page_context, PageContextRequest};
use crate::utils::date_format::format_datetime;

/// AI消息构建器
pub struct AIMessageBuilder {
    user_id: Uuid,
    user_role: String,
    user_permissions: Vec<String>,
    current_page: Option<String>,
    page_context: Option<Value>,
}

impl AIMessageBuilder {
    /// 创建新的消息构建器
    pub fn new(user_id: Uuid, user_role: String, user_permissions: Vec<String>) -> Self {
        Self {
            user_id,
            user_role,
            user_permissions,
            current_page: None,
            page_context: None,
        }
    }

    /// 设置当前页面
    pub fn with_page(&mut self, page: String, page_context: Value) -> &mut Self {
        self.current_page = Some(page);
        self.page_context = Some(page_context);
        self
    }

    /// 构建完整的系统提示词
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = String::from("你是一个专业的学校管理系统AI助手。你的职责是帮助用户完成学校管理相关的任务。\n\n");

        // 添加用户信息
        prompt.push_str(&format!("## 用户信息\n"));
        prompt.push_str(&format!("- 用户ID: {}\n", self.user_id));
        prompt.push_str(&format!("- 用户角色: {}\n", self.user_role));
        prompt.push_str(&format!("- 用户权限: {:?}\n\n", self.user_permissions));

        // 添加页面上下文
        if let Some(page) = &self.current_page {
            prompt.push_str(&format!("## 当前页面上下文\n"));
            prompt.push_str(&format!("页面: {}\n", page));
            if let Some(context) = &self.page_context {
                prompt.push_str(&format!("数据: {}\n\n", serde_json::to_string_pretty(context).unwrap_or_default()));
            }
        }

        // 添加可用操作说明
        prompt.push_str(&self.build_available_operations());

        // 添加操作格式要求
        prompt.push_str(&self.build_action_format());

        // 添加操作执行规则
        prompt.push_str(&self.build_action_rules());

        prompt
    }

    /// 构建可用操作说明
    fn build_available_operations(&self) -> String {
        let mut ops = String::from("## 可用操作\n");
        ops.push_str("根据用户权限，你可以执行以下操作：\n\n");
        
        ops.push_str("### 数据查询操作（仅读取）\n");
        ops.push_str("- get_persons: 查询人员列表，参数：type(可选), search(可选), class_id(可选), limit(可选)\n");
        ops.push_str("- get_attendances: 查询考勤记录，参数：person_id(可选), date(可选), status(可选), class_id(可选)\n");
        ops.push_str("- get_classes: 查询班级列表，参数：grade(可选)\n");
        ops.push_str("- get_groups: 查询小组列表，参数：class_id(可选)\n");
        ops.push_str("- get_departments: 查询部门列表\n");
        ops.push_str("- get_notices: 查询公告列表，参数：limit(可选), sort_by(可选), order(可选)\n\n");
        
        ops.push_str("### 数据操作（需要相应权限）\n");
        if self.user_permissions.contains(&"person.create".to_string()) {
            ops.push_str("- create_person: 创建单个人员\n");
            ops.push_str("- create_persons_batch: 批量创建人员\n");
        }
        if self.user_permissions.contains(&"attendance.create".to_string()) {
            ops.push_str("- create_attendance: 创建单条考勤记录\n");
            ops.push_str("- create_attendances_batch: 批量创建考勤记录\n");
        }
        if self.user_permissions.contains(&"score.create".to_string()) {
            ops.push_str("- create_score: 创建单条成绩记录\n");
            ops.push_str("- create_scores_batch: 批量创建成绩记录\n");
        }
        if self.user_permissions.contains(&"notice.create".to_string()) {
            ops.push_str("- create_notice: 创建公告\n");
        }
        ops.push_str("\n");
        
        ops
    }

    /// 构建操作格式要求
    fn build_action_format(&self) -> String {
        let mut format = String::from("## 操作格式要求\n\n");
        format.push_str("当你需要执行操作时，请使用以下格式：\n\n");
        
        format.push_str("### 单条操作\n");
        format.push_str("```\n");
        format.push_str("[AI_ACTION]\n");
        format.push_str("{\n");
        format.push_str("  \"action_type\": \"操作名称\",\n");
        format.push_str("  \"params\": {参数对象},\n");
        format.push_str("  \"reason\": \"执行此操作的原因\"\n");
        format.push_str("}\n");
        format.push_str("[/AI_ACTION]\n");
        format.push_str("```\n\n");
        
        format.push_str("### 批量操作\n");
        format.push_str("```\n");
        format.push_str("[AI_ACTION]\n");
        format.push_str("{\n");
        format.push_str("  \"action_type\": \"操作名称_batch\",\n");
        format.push_str("  \"batch\": true,\n");
        format.push_str("  \"items\": [参数对象数组],\n");
        format.push_str("  \"reason\": \"执行此批量操作的原因\"\n");
        format.push_str("}\n");
        format.push_str("[/AI_ACTION]\n");
        format.push_str("```\n\n");
        
        format
    }

    /// 构建操作执行规则
    fn build_action_rules(&self) -> String {
        let mut rules = String::from("## 操作执行规则\n\n");
        rules.push_str("1. **先查询后操作**：在执行创建操作前，如果需要选择班级、部门等关联数据，请先使用get_*操作查询可用选项\n");
        rules.push_str("2. **先确认后执行**：在执行创建、更新等操作前，如果用户指令不够明确，请先询问用户确认细节\n");
        rules.push_str("3. **权限检查**：始终检查用户是否有执行操作的权限，如果没有，礼貌地告知用户\n");
        rules.push_str("4. **数据验证**：确保操作参数完整且格式正确，如有缺失请询问用户补充\n");
        rules.push_str("5. **批量操作优先**：当用户要求创建多个类似记录时，优先使用批量操作接口\n");
        rules.push_str("6. **结果反馈**：操作执行完成后，用简洁明了的语言向用户反馈结果\n");
        rules.push_str("7. **多步骤协调**：复杂任务可以分多步骤执行，每步完成后告知用户进度\n\n");
        
        rules.push_str("## 响应风格\n\n");
        rules.push_str("1. 使用中文回复\n");
        rules.push_str("2. 回答要简洁、准确、专业\n");
        rules.push_str("3. 对于复杂操作，分步骤说明\n");
        rules.push_str("4. 可以使用Markdown格式来格式化回复，但避免过度使用\n");
        rules.push_str("5. 友好、耐心、乐于助人\n");
        
        rules
    }
}

/// 完整的消息发送函数
pub async fn send_message_to_ai(
    pool: &PgPool,
    claims: &Claims,
    user_message: String,
    page_info: Option<PageContextRequest>,
) -> Result<String, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)?;
    
    // 1. 获取用户权限
    let permissions = get_user_permissions(pool, user_id).await?;
    
    // 2. 创建消息构建器
    let mut builder = AIMessageBuilder::new(
        user_id,
        claims.role.clone(),
        permissions,
    );
    
    // 3. 获取页面上下文（如果提供）
    if let Some(page_req) = page_info {
        let page_context = get_page_context_data(pool, user_id, &page_req).await?;
        builder.with_page(page_req.page.clone(), page_context);
    }
    
    // 4. 构建系统提示词
    let system_prompt = builder.build_system_prompt();
    
    // 5. 调用AI模型（这里简化处理，实际需要调用AI服务）
    let ai_response = call_ai_model(system_prompt, user_message).await?;
    
    Ok(ai_response)
}

/// 获取用户权限
async fn get_user_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, AppError> {
    let permissions = sqlx::query_scalar!(
        "SELECT permission_code FROM user_permissions WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await?;
    
    Ok(permissions)
}

/// 获取页面上下文数据
async fn get_page_context_data(
    pool: &PgPool,
    user_id: Uuid,
    req: &PageContextRequest,
) -> Result<Value, AppError> {
    // 这里复用ai_context_provider中的函数
    // 简化版实现
    Ok(serde_json::json!({
        "page": req.page,
        "path": req.path,
        "timestamp": format_datetime(&Utc::now().naive_utc())
    }))
}

/// 调用AI模型（模拟）
async fn call_ai_model(system_prompt: String, user_message: String) -> Result<String, AppError> {
    // 实际实现需要调用真实的AI服务
    Ok(format!("AI回复: 收到系统提示词和用户消息"))
}
```

### 11.2 页面上下文信息获取详解

#### 11.2.1 上下文获取流程

```
用户访问页面
    ↓
前端检测路由变化
    ↓
调用usePageContext.getContext()
    ↓
根据路由判断当前页面类型
    ↓
调用对应的上下文获取函数
    ↓
向后端/api/ai/context发送请求
    ↓
后端调用ai_context_provider中的对应函数
    ↓
从数据库查询相关数据
    ↓
格式化数据（确保类型正确）
    ↓
返回精简的上下文数据
    ↓
前端存储在usePageContext中
    ↓
调用AI时传递给消息构建器
```

#### 11.2.2 前端页面上下文获取函数（完整实现）

**文件位置：** `frontend/src/composables/usePageContext.ts`

```typescript
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { aiApi } from '../api/ai'
import { formatDate, formatDateTime } from '../utils/dateFormat'

export interface PageContext {
  page: string
  path: string
  data: any
  timestamp: string
}

export function usePageContext() {
  const route = useRoute()
  const pageContext = ref<PageContext | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  /**
   * 根据当前路由判断页面类型
   */
  const getPageType = (): string => {
    const path = route.path
    
    if (path.includes('/person')) return 'person'
    if (path.includes('/attendance')) return 'attendance'
    if (path.includes('/notice')) return 'notice'
    if (path.includes('/class')) return 'class'
    if (path.includes('/group')) return 'group'
    if (path.includes('/dashboard') || path === '/') return 'dashboard'
    
    return 'unknown'
  }

  /**
   * 获取页面上下文的主要函数
   * 这个函数会：
   * 1. 确定当前页面类型
   * 2. 向后端请求上下文数据
   * 3. 格式化并缓存数据
   */
  const getContext = async (): Promise<PageContext | null> => {
    if (loading.value) return pageContext.value
    
    loading.value = true
    error.value = null
    
    try {
      const pageType = getPageType()
      const path = route.path
      
      // 向后端请求上下文数据
      const response = await aiApi.getPageContext({
        page: pageType,
        path: path,
        params: route.params,
        query: route.query
      })
      
      if (response.data) {
        pageContext.value = {
          page: pageType,
          path: path,
          data: response.data,
          timestamp: formatDateTime(new Date())
        }
      }
      
      return pageContext.value
    } catch (err) {
      error.value = err instanceof Error ? err.message : '获取上下文失败'
      console.error('获取页面上下文失败:', err)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * 格式化上下文数据用于显示
   */
  const formattedContext = computed(() => {
    if (!pageContext.value) return null
    
    return {
      ...pageContext.value,
      data: formatContextData(pageContext.value.data)
    }
  })

  /**
   * 格式化上下文数据（确保类型正确）
   */
  const formatContextData = (data: any): any => {
    if (!data) return data
    
    const formatted = { ...data }
    
    // 格式化日期字段
    if (formatted.current_date) {
      formatted.current_date = formatDate(formatted.current_date)
    }
    
    // 格式化数组中的日期
    if (Array.isArray(formatted.classes)) {
      formatted.classes = formatted.classes.map((cls: any) => ({
        ...cls,
        // 如果有日期字段，在这里格式化
      }))
    }
    
    if (Array.isArray(formatted.recent_notices)) {
      formatted.recent_notices = formatted.recent_notices.map((notice: any) => ({
        ...notice,
        created_at: notice.created_at ? formatDateTime(notice.created_at) : undefined
      }))
    }
    
    return formatted
  }

  /**
   * 清除上下文缓存
   */
  const clearContext = () => {
    pageContext.value = null
    error.value = null
  }

  return {
    getContext,
    pageContext,
    formattedContext,
    loading,
    error,
    getPageType,
    clearContext
  }
}
```

### 11.3 数据库类型匹配与验证

#### 11.3.1 JSON类型与PostgreSQL类型的详细映射

| JSON字段类型 | PostgreSQL类型 | 验证规则 | 格式化要求 | 示例 | 错误提示 |
|-------------|---------------|---------|-----------|------|---------|
| string | VARCHAR(100) | 长度 ≤ 100，非空（必填字段） | 直接使用 | "张三" | "姓名长度不能超过100个字符" |
| string | VARCHAR(50) | 长度 ≤ 50 | 直接使用 | "2026001" | "学号长度不能超过50个字符" |
| string | TEXT | 长度 ≤ 500（备注类） | 直接使用 | "正常出勤" | "备注长度不能超过500个字符" |
| integer | SMALLINT | 0 ≤ value ≤ 32767 | 直接使用 | 1 | "性别值应在0-2之间" |
| integer | INTEGER | -2147483648 ≤ value ≤ 2147483647 | 直接使用 | 95 | "分数值应在0-100之间" |
| boolean | BOOLEAN | true/false | 直接使用 | true | - |
| date | DATE | 格式YYYY-MM-DD | parse_date()验证 | "2026-03-07" | "日期格式无效，应为YYYY-MM-DD" |
| time | TIME | 格式HH:MM:SS | parse_time()验证 | "08:30:00" | "时间格式无效，应为HH:MM:SS" |
| datetime | TIMESTAMP | 格式RFC3339 | parse_datetime()验证 | "2026-03-07T08:30:00Z" | "日期时间格式错误" |
| uuid | UUID | 标准UUID格式 | Uuid::parse_str()验证 | "550e8400-e29b-41d4-a716-446655440000" | "UUID格式无效" |
| array | ARRAY | JSON数组格式 | 直接使用 | ["a", "b", "c"] | - |
| object | JSONB | JSON对象格式 | 直接使用 | {"key": "value"} | - |

#### 11.3.2 参数验证器增强版（包含数据库类型检查）

**文件位置：** `backend/src/core/ai_action_validator.rs`

```rust
use serde_json::Value;
use uuid::Uuid;
use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use std::collections::HashMap;

use crate::core::error::AppError;

/// 字段验证规则
#[derive(Debug, Clone)]
pub struct FieldRule {
    pub field_name: &'static str,
    pub required: bool,
    pub field_type: FieldType,
    pub max_length: Option<usize>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub allowed_values: Option<Vec<&'static str>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Boolean,
    Date,
    Time,
    DateTime,
    Uuid,
}

/// 验证规则集合
pub struct ValidationRules;

impl ValidationRules {
    /// create_person操作的验证规则
    pub fn create_person_rules() -> Vec<FieldRule> {
        vec![
            FieldRule {
                field_name: "name",
                required: true,
                field_type: FieldType::String,
                max_length: Some(100),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "gender",
                required: true,
                field_type: FieldType::Integer,
                max_length: None,
                min_value: Some(0),
                max_value: Some(2),
                allowed_values: None,
            },
            FieldRule {
                field_name: "type",
                required: true,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: Some(vec!["student", "teacher", "parent"]),
            },
            FieldRule {
                field_name: "birthday",
                required: false,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "phone",
                required: false,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "email",
                required: false,
                field_type: FieldType::String,
                max_length: Some(100),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "student_no",
                required: false, // 条件必填，在validate_create_person中检查
                field_type: FieldType::String,
                max_length: Some(50),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "class_id",
                required: false,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "enrollment_date",
                required: false,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "employee_no",
                required: false, // 条件必填
                field_type: FieldType::String,
                max_length: Some(50),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "department_id",
                required: false,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "title",
                required: false,
                field_type: FieldType::String,
                max_length: Some(50),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "hire_date",
                required: false,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
        ]
    }

    /// create_attendance操作的验证规则
    pub fn create_attendance_rules() -> Vec<FieldRule> {
        vec![
            FieldRule {
                field_name: "person_id",
                required: true,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "date",
                required: true,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "status",
                required: true,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: Some(vec!["present", "absent", "late", "early_leave", "excused"]),
            },
            FieldRule {
                field_name: "time",
                required: false,
                field_type: FieldType::Time,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "remark",
                required: false,
                field_type: FieldType::String,
                max_length: Some(500),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
        ]
    }

    /// create_score操作的验证规则
    pub fn create_score_rules() -> Vec<FieldRule> {
        vec![
            FieldRule {
                field_name: "person_id",
                required: true,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "score_type",
                required: true,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: Some(vec!["personal", "group", "class", "dormitory"]),
            },
            FieldRule {
                field_name: "value",
                required: true,
                field_type: FieldType::Integer,
                max_length: None,
                min_value: Some(0),
                max_value: Some(100),
                allowed_values: None,
            },
            FieldRule {
                field_name: "reason",
                required: true,
                field_type: FieldType::String,
                max_length: Some(500),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "group_id",
                required: false,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
        ]
    }
}

/// AI操作参数验证器
pub struct AIActionValidator;

impl AIActionValidator {
    /// 通用字段验证函数
    pub fn validate_field(value: &Value, rule: &FieldRule) -> Result<(), AppError> {
        let field_value = value.get(rule.field_name);
        
        // 检查必填字段
        if rule.required && field_value.is_none() {
            return Err(AppError::InvalidInput(
                format!("缺少必填字段: {}", rule.field_name)
            ));
        }
        
        let Some(field_value) = field_value else {
            return Ok(()); // 非必填且不存在，跳过
        };
        
        // 如果字段是null或空字符串，也跳过（非必填）
        if field_value.is_null() || 
           (field_value.is_string() && field_value.as_str().unwrap_or("").is_empty()) {
            if rule.required {
                return Err(AppError::InvalidInput(
                    format!("必填字段不能为空: {}", rule.field_name)
                ));
            }
            return Ok(());
        }
        
        // 根据字段类型进行验证
        match rule.field_type {
            FieldType::String => {
                let s = field_value.as_str()
                    .ok_or_else(|| AppError::InvalidInput(
                        format!("字段 {} 必须是字符串类型", rule.field_name)
                    ))?;
                
                if let Some(max_len) = rule.max_length {
                    if s.len() > max_len {
                        return Err(AppError::InvalidInput(
                            format!("字段 {} 长度不能超过 {} 个字符", rule.field_name, max_len)
                        ));
                    }
                }
                
                if let Some(allowed) = &rule.allowed_values {
                    if !allowed.contains(&s) {
                        return Err(AppError::InvalidInput(
                            format!("字段 {} 的值无效，允许的值: {:?}", rule.field_name, allowed)
                        ));
                    }
                }
            }
            
            FieldType::Integer => {
                let n = field_value.as_i64()
                    .ok_or_else(|| AppError::InvalidInput(
                        format!("字段 {} 必须是整数类型", rule.field_name)
                    ))?;
                
                if let Some(min) = rule.min_value {
                    if n < min {
                        return Err(AppError::InvalidInput(
                            format!("字段 {} 的值不能小于 {}", rule.field_name, min)
                        ));
                    }
                }
                
                if let Some(max) = rule.max_value {
                    if n > max {
                        return Err(AppError::InvalidInput(
                            format!("字段 {} 的值不能大于 {}", rule.field_name, max)
                        ));
                    }
                }
            }
            
            FieldType::Boolean => {
                if !field_value.is_boolean() {
                    return Err(AppError::InvalidInput(
                        format!("字段 {} 必须是布尔类型", rule.field_name)
                    ));
                }
            }
            
            FieldType::Date => {
                let s = field_value.as_str()
                    .ok_or_else(|| AppError::InvalidInput(
                        format!("字段 {} 必须是字符串类型", rule.field_name)
                    ))?;
                
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|_| AppError::InvalidInput(
                        format!("字段 {} 日期格式无效，应为YYYY-MM-DD: {}", rule.field_name, s)
                    ))?;
            }
            
            FieldType::Time => {
                let s = field_value.as_str()
                    .ok_or_else(|| AppError::InvalidInput(
                        format!("字段 {} 必须是字符串类型", rule.field_name)
                    ))?;
                
                NaiveTime::parse_from_str(s, "%H:%M:%S")
                    .map_err(|_| AppError::InvalidInput(
                        format!("字段 {} 时间格式无效，应为HH:MM:SS: {}", rule.field_name, s)
                    ))?;
            }
            
            FieldType::DateTime => {
                let s = field_value.as_str()
                    .ok_or_else(|| AppError::InvalidInput(
                        format!("字段 {} 必须是字符串类型", rule.field_name)
                    ))?;
                
                // 尝试多种格式
                let _ = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                    .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                    .map_err(|_| AppError::InvalidInput(
                        format!("字段 {} 日期时间格式无效: {}", rule.field_name, s)
                    ))?;
            }
            
            FieldType::Uuid => {
                let s = field_value.as_str()
                    .ok_or_else(|| AppError::InvalidInput(
                        format!("字段 {} 必须是字符串类型", rule.field_name)
                    ))?;
                
                Uuid::parse_str(s)
                    .map_err(|_| AppError::InvalidInput(
                        format!("字段 {} UUID格式无效: {}", rule.field_name, s)
                    ))?;
            }
        }
        
        Ok(())
    }

    /// 验证创建人员参数
    pub fn validate_create_person(params: &Value) -> Result<(), AppError> {
        let rules = ValidationRules::create_person_rules();
        
        // 验证所有字段
        for rule in &rules {
            Self::validate_field(params, rule)?;
        }
        
        // 条件必填验证
        let type_ = params.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少type字段".to_string()))?;
        
        match type_ {
            "student" => {
                let student_no = params.get("student_no")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty());
                
                if student_no.is_none() {
                    return Err(AppError::InvalidInput("学生缺少必填字段: student_no".to_string()));
                }
            }
            "teacher" => {
                let employee_no = params.get("employee_no")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty());
                
                if employee_no.is_none() {
                    return Err(AppError::InvalidInput("教师缺少必填字段: employee_no".to_string()));
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// 验证创建考勤参数
    pub fn validate_create_attendance(params: &Value) -> Result<(), AppError> {
        let rules = ValidationRules::create_attendance_rules();
        
        for rule in &rules {
            Self::validate_field(params, rule)?;
        }
        
        Ok(())
    }

    /// 验证创建成绩参数
    pub fn validate_create_score(params: &Value) -> Result<(), AppError> {
        let rules = ValidationRules::create_score_rules();
        
        for rule in &rules {
            Self::validate_field(params, rule)?;
        }
        
        Ok(())
    }
}
```

### 11.4 完整的函数调用示例

#### 11.4.1 在AI聊天中使用消息构建函数

**后端使用示例：**

```rust
// 在 ai_enhanced.rs 中
pub async fn enhanced_chat(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<EnhancedChatRequest>,
) -> Result<Json<EnhancedChatResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 构建页面上下文请求
    let page_context_req = req.page_info.map(|info| PageContextRequest {
        page: info.page,
        path: info.path,
        params: info.params,
        query: info.query,
    });
    
    // 使用消息发送函数
    let ai_response = send_message_to_ai(
        &pool,
        &claims,
        req.user_message,
        page_context_req,
    ).await?;
    
    // 解析AI响应，检测是否有操作
    let parsed_response = parse_ai_response(&ai_response);
    
    Ok(Json(EnhancedChatResponse {
        success: true,
        message: parsed_response.message,
        has_action: parsed_response.has_action,
        action: parsed_response.action,
    }))
}
```

**前端调用示例：**

```typescript
// 在 AIView.vue 中
<script setup lang="ts">
import { ref } from 'vue'
import { aiApi } from '../api/ai'
import { usePageContext } from '../composables/usePageContext'

const { getContext, pageContext } = usePageContext()
const userInput = ref('')
const messages = ref<Array<{role: string, content: string}>>([])

const sendMessage = async () => {
  if (!userInput.value.trim()) return
  
  // 添加用户消息
  messages.value.push({
    role: 'user',
    content: userInput.value
  })
  
  // 获取当前页面上下文
  const context = await getContext()
  
  try {
    // 调用增强版AI聊天接口
    const response = await aiApi.enhancedChat({
      user_message: userInput.value,
      page_info: context ? {
        page: context.page,
        path: context.path,
        params: {},
        query: {}
      } : undefined
    })
    
    // 添加AI回复
    messages.value.push({
      role: 'assistant',
      content: response.data.message
    })
    
    // 如果有操作，执行操作
    if (response.data.has_action && response.data.action) {
      await executeAIAction(response.data.action)
    }
  } catch (error) {
    console.error('发送消息失败:', error)
  }
  
  userInput.value = ''
}

const executeAIAction = async (action: any) => {
  // 执行AI操作的逻辑
  try {
    const response = await aiApi.executeAction(action)
    if (response.data.success) {
      messages.value.push({
        role: 'system',
        content: `操作执行成功: ${response.data.message}`
      })
    }
  } catch (error) {
    console.error('执行操作失败:', error)
  }
}
</script>
```

#### 11.4.2 格式化后的消息示例

**发送给AI的完整消息格式：**

```
你是一个专业的学校管理系统AI助手。你的职责是帮助用户完成学校管理相关的任务。

## 用户信息
- 用户ID: 550e8400-e29b-41d4-a716-446655440000
- 用户角色: teacher
- 用户权限: ["person.create", "person.view", "attendance.create", "attendance.view"]

## 当前页面上下文
页面: person
数据: {
  "page": "person",
  "stats": {
    "total": 45,
    "incomplete_info": 8,
    "no_phone": 5,
    "no_email": 3
  },
  "classes": [
    {
      "id": "uuid-1",
      "name": "高一(1)班",
      "student_count": 45
    },
    {
      "id": "uuid-2",
      "name": "高一(2)班",
      "student_count": 42
    }
  ]
}

## 可用操作
根据用户权限，你可以执行以下操作：

### 数据查询操作（仅读取）
- get_persons: 查询人员列表，参数：type(可选), search(可选), class_id(可选), limit(可选)
- get_attendances: 查询考勤记录，参数：person_id(可选), date(可选), status(可选), class_id(可选)
- get_classes: 查询班级列表，参数：grade(可选)
- get_groups: 查询小组列表，参数：class_id(可选)
- get_departments: 查询部门列表
- get_notices: 查询公告列表，参数：limit(可选), sort_by(可选), order(可选)

### 数据操作（需要相应权限）
- create_person: 创建单个人员
- create_persons_batch: 批量创建人员
- create_attendance: 创建单条考勤记录
- create_attendances_batch: 批量创建考勤记录

## 操作格式要求

当你需要执行操作时，请使用以下格式：

### 单条操作
```
[AI_ACTION]
{
  "action_type": "操作名称",
  "params": {参数对象},
  "reason": "执行此操作的原因"
}
[/AI_ACTION]
```

### 批量操作
```
[AI_ACTION]
{
  "action_type": "操作名称_batch",
  "batch": true,
  "items": [参数对象数组],
  "reason": "执行此批量操作的原因"
}
[/AI_ACTION]
```

## 操作执行规则

1. **先查询后操作**：在执行创建操作前，如果需要选择班级、部门等关联数据，请先使用get_*操作查询可用选项
2. **先确认后执行**：在执行创建、更新等操作前，如果用户指令不够明确，请先询问用户确认细节
3. **权限检查**：始终检查用户是否有执行操作的权限，如果没有，礼貌地告知用户
4. **数据验证**：确保操作参数完整且格式正确，如有缺失请询问用户补充
5. **批量操作优先**：当用户要求创建多个类似记录时，优先使用批量操作接口
6. **结果反馈**：操作执行完成后，用简洁明了的语言向用户反馈结果
7. **多步骤协调**：复杂任务可以分多步骤执行，每步完成后告知用户进度

## 响应风格

1. 使用中文回复
2. 回答要简洁、准确、专业
3. 对于复杂操作，分步骤说明
4. 可以使用Markdown格式来格式化回复，但避免过度使用
5. 友好、耐心、乐于助人

[用户消息]
请帮我创建一个新学生
```

#### 11.4.3 验证失败时的错误提示示例

```json
{
  "success": false,
  "action_type": "create_person",
  "error_code": "INVALID_INPUT",
  "message": "字段 student_no 长度不能超过 50 个字符",
  "details": "参数验证失败",
  "recoverable": true,
  "suggestions": ["请缩短学号长度", "检查学号输入是否正确"]
}
```

```json
{
  "success": false,
  "action_type": "create_attendance",
  "error_code": "INVALID_INPUT",
  "message": "字段 date 日期格式无效，应为YYYY-MM-DD: 2026/03/07",
  "details": "日期格式错误",
  "recoverable": true,
  "suggestions": ["请使用YYYY-MM-DD格式", "例如: 2026-03-07"]
}
```

```json
{
  "success": false,
  "action_type": "create_score",
  "error_code": "INVALID_INPUT",
  "message": "字段 person_id UUID格式无效: invalid-uuid",
  "details": "UUID格式验证失败",
  "recoverable": true,
  "suggestions": ["请使用有效的UUID格式", "例如: 550e8400-e29b-41d4-a716-446655440000"]
}
```

## 12. 风险分析与优化建议

### 12.1 发现的潜在风险

#### 12.1.1 递归调用风险
✅ **无风险** - 经过检查，代码中没有发现递归调用，结构良好。

#### 12.1.2 过多的if判断条件

**问题1：build_available_operations()中的重复if判断**
```rust
// 原代码
if self.user_permissions.contains(&"person.create".to_string()) {
    ops.push_str("- create_person: 创建单个人员\n");
    ops.push_str("- create_persons_batch: 批量创建人员\n");
}
if self.user_permissions.contains(&"attendance.create".to_string()) {
    ops.push_str("- create_attendance: 创建单条考勤记录\n");
    ops.push_str("- create_attendances_batch: 批量创建考勤记录\n");
}
// ... 更多类似的if判断
```

**优化建议：** 使用映射表来减少重复代码

**问题2：validate_field()中的长match语句**
虽然match语句在Rust中是惯用的，但可以考虑进一步优化代码组织。

**问题3：前端formatContextData()中的多个独立if判断**
```typescript
// 原代码
if (formatted.current_date) {
  formatted.current_date = formatDate(formatted.current_date)
}
if (Array.isArray(formatted.classes)) {
  // ...
}
if (Array.isArray(formatted.recent_notices)) {
  // ...
}
```

**优化建议：** 使用字段映射表

#### 12.1.3 其他潜在问题

**问题1：未使用的导入**
在 `ai_action_validator.rs` 中导入了 `HashMap` 但未使用：
```rust
use std::collections::HashMap;  // 未使用
```

**问题2：usePageContext.ts中的竞态条件**
当loading为true时，直接返回旧的pageContext.value，可能导致数据不一致：
```typescript
const getContext = async (): Promise<PageContext | null> => {
  if (loading.value) return pageContext.value  // 潜在竞态条件
  // ...
}
```

**优化建议：** 使用请求队列或取消机制

**问题3：前端缺少防抖/节流**
`getContext()` 可能被频繁调用（如路由快速切换时），没有防抖机制。

**问题4：formatContextData()中的浅层拷贝问题**
```typescript
const formatted = { ...data }  // 只做了浅层拷贝
// 如果data中有嵌套对象，修改formatted会影响原data
```

**问题5：缺少输入大小限制**
- 没有对用户消息长度进行限制
- 没有对page_context大小进行限制，可能导致token溢出

**问题6：错误处理可以更细致**
验证器的错误信息可以更具体，帮助调试。

### 12.2 优化后的代码示例

#### 12.2.1 优化build_available_operations()

```rust
fn build_available_operations(&self) -> String {
    let mut ops = String::from("## 可用操作\n");
    ops.push_str("根据用户权限，你可以执行以下操作：\n\n");
    
    ops.push_str("### 数据查询操作（仅读取）\n");
    ops.push_str("- get_persons: 查询人员列表，参数：type(可选), search(可选), class_id(可选), limit(可选)\n");
    ops.push_str("- get_attendances: 查询考勤记录，参数：person_id(可选), date(可选), status(可选), class_id(可选)\n");
    ops.push_str("- get_classes: 查询班级列表，参数：grade(可选)\n");
    ops.push_str("- get_groups: 查询小组列表，参数：class_id(可选)\n");
    ops.push_str("- get_departments: 查询部门列表\n");
    ops.push_str("- get_notices: 查询公告列表，参数：limit(可选), sort_by(可选), order(可选)\n\n");
    
    // 使用映射表优化权限检查
    #[derive(Debug)]
    struct OperationGroup {
        permission: &'static str,
        operations: &'static [&'static str],
    }
    
    let operation_groups = [
        OperationGroup {
            permission: "person.create",
            operations: &[
                "- create_person: 创建单个人员",
                "- create_persons_batch: 批量创建人员",
            ],
        },
        OperationGroup {
            permission: "attendance.create",
            operations: &[
                "- create_attendance: 创建单条考勤记录",
                "- create_attendances_batch: 批量创建考勤记录",
            ],
        },
        OperationGroup {
            permission: "score.create",
            operations: &[
                "- create_score: 创建单条成绩记录",
                "- create_scores_batch: 批量创建成绩记录",
            ],
        },
        OperationGroup {
            permission: "notice.create",
            operations: &[
                "- create_notice: 创建公告",
            ],
        },
    ];
    
    ops.push_str("### 数据操作（需要相应权限）\n");
    for group in &operation_groups {
        if self.user_permissions.contains(&group.permission.to_string()) {
            for op in group.operations {
                ops.push_str(op);
                ops.push('\n');
            }
        }
    }
    ops.push('\n');
    
    ops
}
```

#### 12.2.2 优化前端formatContextData()

```typescript
const formatContextData = (data: any): any => {
  if (!data) return data
  
  // 深拷贝避免修改原数据
  const formatted = JSON.parse(JSON.stringify(data))
  
  // 定义字段格式化规则
  const formatRules = {
    current_date: formatDate,
    created_at: formatDateTime,
    updated_at: formatDateTime
  }
  
  // 递归格式化所有日期字段
  const formatObject = (obj: any) => {
    if (!obj || typeof obj !== 'object') return obj
    
    if (Array.isArray(obj)) {
      return obj.map(formatObject)
    }
    
    const result = { ...obj }
    for (const [key, value] of Object.entries(result)) {
      if (formatRules[key as keyof typeof formatRules]) {
        result[key] = formatRules[key as keyof typeof formatRules](value)
      } else if (typeof value === 'object') {
        result[key] = formatObject(value)
      }
    }
    return result
  }
  
  return formatObject(formatted)
}
```

#### 12.2.3 优化前端getContext()添加防抖

```typescript
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { aiApi } from '../api/ai'
import { formatDate, formatDateTime } from '../utils/dateFormat'

// 简单的防抖函数
function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

export function usePageContext() {
  const route = useRoute()
  const pageContext = ref<PageContext | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastFetchTime = ref(0)
  const CACHE_DURATION = 30000 // 30秒缓存

  // ... getPageType 函数保持不变 ...

  const getContext = async (force = false): Promise<PageContext | null> => {
    const now = Date.now()
    
    // 检查缓存
    if (!force && pageContext.value && (now - lastFetchTime.value) < CACHE_DURATION) {
      return pageContext.value
    }
    
    if (loading.value) {
      // 返回Promise等待当前请求完成
      return new Promise((resolve) => {
        const checkLoading = setInterval(() => {
          if (!loading.value) {
            clearInterval(checkLoading)
            resolve(pageContext.value)
          }
        }, 100)
      })
    }
    
    loading.value = true
    error.value = null
    
    try {
      const pageType = getPageType()
      const path = route.path
      
      const response = await aiApi.getPageContext({
        page: pageType,
        path: path,
        params: route.params,
        query: route.query
      })
      
      if (response.data) {
        pageContext.value = {
          page: pageType,
          path: path,
          data: response.data,
          timestamp: formatDateTime(new Date())
        }
        lastFetchTime.value = now
      }
      
      return pageContext.value
    } catch (err) {
      error.value = err instanceof Error ? err.message : '获取上下文失败'
      console.error('获取页面上下文失败:', err)
      return null
    } finally {
      loading.value = false
    }
  }

  // 防抖版本，用于路由变化时
  const getContextDebounced = debounce(() => getContext(), 500)

  // ... 其余函数保持不变 ...

  return {
    getContext,
    getContextDebounced,
    pageContext,
    formattedContext,
    loading,
    error,
    getPageType,
    clearContext
  }
}
```

#### 12.2.4 添加输入大小限制

```rust
// 在 send_message_to_ai 函数中添加
const MAX_USER_MESSAGE_LENGTH: usize = 10000;
const MAX_PAGE_CONTEXT_SIZE: usize = 50000;

pub async fn send_message_to_ai(
    pool: &PgPool,
    claims: &Claims,
    user_message: String,
    page_info: Option<PageContextRequest>,
) -> Result<String, AppError> {
    // 检查用户消息长度
    if user_message.len() > MAX_USER_MESSAGE_LENGTH {
        return Err(AppError::InvalidInput(
            format!("用户消息过长，不能超过 {} 字符", MAX_USER_MESSAGE_LENGTH)
        ));
    }
    
    // ... 其余代码 ...
}
```

### 12.3 优化优先级建议

| 优先级 | 问题 | 影响 | 难度 |
|-------|------|------|------|
| P0 | 输入大小限制 | 高（安全性） | 低 |
| P0 | 未使用的导入 | 低（代码质量） | 极低 |
| P1 | 竞态条件 | 中（数据一致性） | 中 |
| P1 | 防抖机制 | 中（性能） | 中 |
| P2 | 减少if判断重复 | 中（可维护性） | 低 |
| P2 | 深拷贝问题 | 低（数据安全） | 低 |
| P3 | 更细致的错误信息 | 低（调试体验） | 低 |

## 13. 注意事项

1. **权限安全**：所有AI操作都必须经过严格的权限检查
2. **数据验证**：所有输入参数都必须经过验证
3. **事务处理**：批量操作必须使用数据库事务
4. **错误处理**：提供友好的错误提示和详细的日志
5. **性能优化**：批量操作注意性能，避免长时间阻塞
6. **日志记录**：记录所有AI操作，便于审计和问题排查
7. **提示词管理**：提示词需要可配置，便于后续优化
8. **数据安全**：不要向AI发送敏感信息，如密码、身份证号等
9. **Token限制**：注意控制发送给AI的数据量，避免token溢出
10. **多步骤协调**：复杂任务要合理分步骤，避免单次操作过于复杂
11. **类型安全**：始终使用验证器确保JSON类型与数据库类型匹配
12. **日期格式**：严格使用YYYY-MM-DD、HH:MM:SS、RFC3339格式
13. **字符串长度**：注意VARCHAR字段的长度限制，避免数据库报错
14. **消息格式化**：始终使用AIMessageBuilder来构建发送给AI的消息
15. **输入限制**：对用户输入和上下文数据进行大小限制，防止滥用
16. **防抖优化**：在频繁调用的场景使用防抖机制
17. **代码清理**：定期清理未使用的导入和代码
