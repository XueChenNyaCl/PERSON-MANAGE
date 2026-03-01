# AI 权限操作功能说明

## 概述

本文档详细说明了 AI 助手功能的权限操作实现，包括 AI 如何识别用户权限、如何执行用户权限范围内的操作，以及错误处理机制。

## 功能特点

### 1. 权限识别

AI 助手能够：
- 自动获取当前用户的权限列表
- 根据用户权限动态调整可执行的操作
- 严格遵守权限边界，不越权执行操作

### 2. 支持的操作类型

根据用户权限，AI 可以执行以下操作：

#### 公告管理
- **创建公告** (`notice.create`)
  - 需要权限：`notice.create` 或 `notice.*`
  - 参数：title(标题), content(内容), target_type(目标类型), target_id(可选), is_important(是否重要)

#### 小组管理
- **创建小组** (`group.create`)
  - 需要权限：`group.create` 或 `group.*`
  - 参数：class_id(班级ID), name(小组名称), description(描述,可选)

- **更新小组积分** (`update_group_score`)
  - 需要权限：`group.update.score` 或 `group.update` 或 `group.*`
  - 参数：group_id(小组ID), score_change(积分变化), reason(原因)

- **添加小组成员** (`add_group_member`)
  - 需要权限：`group.update.member` 或 `group.update` 或 `group.*`
  - 参数：group_id(小组ID), person_id(人员ID)

- **移除小组成员** (`remove_group_member`)
  - 需要权限：`group.update.member` 或 `group.update` 或 `group.*`
  - 参数：group_id(小组ID), person_id(人员ID)

#### 考勤管理
- **创建考勤记录** (`create_attendance`)
  - 需要权限：`attendance.create` 或 `attendance.*`
  - 参数：person_id(人员ID), date(日期), status(状态), time(时间,可选), remark(备注,可选)
  - 状态可选值：present(出勤), late(迟到), absent(缺勤), early_leave(早退), excused(请假)

#### 成绩管理
- **创建成绩记录** (`create_score`)
  - 需要权限：`score.create` 或 `score.*`
  - 参数：student_id(学生ID), subject(科目), score(分数,0-100), exam_date(考试日期,可选), remark(备注,可选)

### 3. 数据查询功能

AI 助手可以继续执行数据查询操作：
- `class_list` - 获取所有班级列表
- `class_detail` - 获取班级详情
- `group_list` - 获取所有小组列表
- `group_detail` - 获取小组详情
- `department_list` - 获取所有部门列表
- `department_detail` - 获取部门详情
- `overview` - 获取学校数据概览

## 技术实现

### 后端实现

#### 文件结构
```
backend/src/api/
├── ai_actions.rs      # AI 操作执行模块
├── ai_enhanced.rs     # 增强版AI聊天模块
├── ai_data.rs         # 数据查询模块
└── routes.rs          # 路由配置
```

#### 核心组件

1. **AIActionExecutor** (`ai_actions.rs`)
   - 执行所有AI请求的操作
   - 严格的权限检查
   - 完整的错误处理

2. **系统提示词模板** (`ai_enhanced.rs`)
   - 动态生成基于用户权限的提示词
   - 包含可用操作的详细说明
   - 指导AI正确使用操作标记

3. **AI响应解析器** (`ai_enhanced.rs`)
   - 解析 `[AI_ACTION]` 标记
   - 提取操作请求参数
   - 支持操作执行和数据查询

### 前端实现

#### API接口 (`frontend/src/api/ai.ts`)
```typescript
// 执行AI操作
executeAction(request: AIActionRequest)

// 获取用户可用的AI操作列表
getAvailableActions()
```

#### 界面更新
- 显示用户可用的操作快捷按钮
- 操作执行状态指示
- 权限不足提示

## API 接口

### 1. 执行AI操作
```
POST /api/ai/actions
```

**请求体：**
```json
{
  "action_type": "create_notice",
  "params": {
    "title": "期中考试通知",
    "content": "下周一进行期中考试",
    "target_type": "all"
  },
  "reason": "发布考试通知"
}
```

**响应：**
```json
{
  "success": true,
  "message": "公告 '期中考试通知' 创建成功",
  "data": {
    "id": "xxx",
    "title": "期中考试通知",
    "author_name": "张三",
    "created_at": "2026-03-01T10:00:00Z"
  },
  "user_permissions": ["notice.create", "group.view", ...]
}
```

### 2. 获取可用操作列表
```
GET /api/ai/actions/available
```

**响应：**
```json
{
  "available_actions": [
    {
      "action_type": "create_notice",
      "name": "创建公告",
      "description": "创建新的学校公告",
      "required_params": ["title", "content", "target_type"],
      "optional_params": ["target_id", "is_important"]
    }
  ],
  "user_permissions": ["notice.create", "group.view", ...]
}
```

### 3. 增强版聊天（支持操作执行）
```
POST /api/ai/enhanced-chat
```

AI 会在响应中使用 `[AI_ACTION]` 标记请求执行操作：
```
[AI_ACTION]
{
    "action_type": "create_notice",
    "params": {"title": "期中考试通知", "content": "下周一进行期中考试", "target_type": "all"},
    "reason": "用户要求创建期中考试通知公告"
}
[/AI_ACTION]
```

## 错误处理

### 权限错误
当用户没有某个操作的权限时：
- AI 会收到权限不足的提示
- AI 会礼貌地告知用户没有权限
- 不会尝试执行越权操作

### 参数错误
当操作参数不正确时：
- 返回具体的参数错误信息
- 提示用户补充必要信息
- AI 会询问缺失的参数

### 业务逻辑错误
当操作违反业务规则时：
- 返回具体的业务错误信息
- 例如："该成员已在小组中"、"小组不存在"等
- AI 会将错误信息友好地呈现给用户

## 使用示例

### 示例1：创建公告
**用户：** "帮我创建一个公告，标题是期中考试通知"

**AI响应：**
```
[AI_ACTION]
{
    "action_type": "create_notice",
    "params": {
        "title": "期中考试通知",
        "content": "下周一进行期中考试，请同学们做好准备。",
        "target_type": "all"
    },
    "reason": "用户要求创建期中考试通知公告"
}
[/AI_ACTION]
```

**执行结果：** "公告 '期中考试通知' 创建成功！"

### 示例2：权限不足
**用户：** "帮我创建一个公告"

**AI响应：** "抱歉，您没有创建公告的权限。请联系管理员获取相应权限。"

### 示例3：更新小组积分
**用户：** "给第一小组增加10分，因为课堂表现优秀"

**AI响应：**
```
[AI_ACTION]
{
    "action_type": "update_group_score",
    "params": {
        "group_id": "xxx",
        "score_change": 10,
        "reason": "课堂表现优秀"
    },
    "reason": "用户要求奖励小组课堂表现"
}
[/AI_ACTION]
```

**执行结果：** "成功为第一小组增加10分！"

## 安全考虑

1. **权限验证**
   - 每次操作执行前都进行权限检查
   - 使用 PermissionManager 获取用户实际权限
   - 支持通配符权限匹配

2. **参数验证**
   - 所有参数都经过验证
   - 必填字段检查
   - 数据格式验证（如UUID、日期等）

3. **错误隔离**
   - 操作失败不会影响系统其他功能
   - 详细的错误日志记录
   - 友好的错误提示

## 后续优化建议

1. **操作历史记录**
   - 记录AI执行的所有操作
   - 支持操作审计和回溯

2. **批量操作支持**
   - 支持一次执行多个操作
   - 批量创建考勤、成绩等

3. **操作确认机制**
   - 重要操作需要用户确认
   - 可配置的操作风险等级

4. **自然语言优化**
   - 提升AI理解复杂指令的能力
   - 支持更自然的对话方式

## 相关文件

### 后端文件
- `backend/src/api/ai_actions.rs` - AI操作执行模块
- `backend/src/api/ai_enhanced.rs` - 增强版AI聊天模块
- `backend/src/api/routes.rs` - 路由配置

### 前端文件
- `frontend/src/api/ai.ts` - AI API接口
- `frontend/src/views/AIView.vue` - AI对话界面

### 文档
- `docs/AI 功能实现说明.md` - AI功能总体说明
- `docs/权限组管理指南.md` - 权限系统说明
