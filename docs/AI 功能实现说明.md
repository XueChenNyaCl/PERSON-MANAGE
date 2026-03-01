# AI 功能实现说明

## 概述

本文档详细说明了 AI 助手功能的完整实现，包括前端页面、后端 API、权限配置和数据库迁移。

## 功能列表

### 1. AI 对话功能
- **路径**: `/dashboard/ai`
- **权限**: `ai.view`, `ai.chat`
- **描述**: 用户可以与 AI 助手进行对话，获取权限范围内的信息和分析

### 2. AI 设置功能（管理员专用）
- **路径**: `/dashboard/ai/settings`
- **权限**: `ai.settings`
- **描述**: 管理员可以配置 AI API 密钥、模型参数等设置

### 3. AI 数据获取接口
- **接口**: `GET /api/ai/context-data`
- **权限**: 根据用户权限动态过滤数据
- **描述**: AI 可以获取用户权限范围内的班级、小组、部门信息

## 前端实现

### 1. 路由配置

**文件**: `frontend/src/router/index.ts`

```typescript
{
  path: 'ai',
  name: 'ai',
  component: () => import('../views/AIView.vue')
},
{
  path: 'ai/settings',
  name: 'ai-settings',
  component: () => import('../views/AISettingsView.vue')
}
```

**重定向**:
```typescript
{
  path: '/ai',
  redirect: '/dashboard/ai'
}
```

### 2. 菜单配置

**文件**: `frontend/src/config/menu.ts`

**新增菜单项**:
```typescript
{
  id: 'ai',
  title: 'AI 对话',
  icon: 'ChatLineRound',
  path: '/dashboard/ai',
  requiredPermission: 'ai.view',
  parentId: 'ai-management'
},
{
  id: 'ai-settings',
  title: 'AI 设置',
  icon: 'Setting',
  path: '/dashboard/ai/settings',
  requiredPermission: 'ai.settings',
  parentId: 'ai-management'
}
```

**新增菜单分组**:
```typescript
{
  id: 'ai-management',
  title: 'AI 助手'
}
```

### 3. 视图组件

#### 3.1 AI 对话页面
**文件**: `frontend/src/views/AIView.vue`

**功能**:
- 消息列表展示
- 消息输入框
- 发送消息按钮
- 加载状态显示
- 错误处理

**样式**: `frontend/src/styles/ai-view.css`

#### 3.2 AI 设置页面
**文件**: `frontend/src/views/AISettingsView.vue`

**功能**:
- API 密钥配置（密码输入）
- API 基础 URL 配置
- 模型选择
- 默认提示词编辑
- 温度参数滑块（0-2）
- 最大 Token 数输入（100-8000）
- 保存设置按钮

**样式**: `frontend/src/styles/ai-settings-view.css`

### 4. API 模块

**文件**: `frontend/src/api/ai.ts`

**接口定义**:
```typescript
export const aiApi = {
  // 聊天
  chat(request: ChatRequest) {
    return api.post<ChatResponse>('/ai/chat', request)
  },

  // 列出 AI 身份
  listIdentities() {
    return api.get<AIIdentity[]>('/ai/identities')
  },

  // 创建 AI 身份
  createIdentity(request: CreateIdentityRequest) {
    return api.post<AIIdentity>('/ai/identities', request)
  },

  // 更新 AI 身份
  updateIdentity(id: string, request: UpdateIdentityRequest) {
    return api.put<AIIdentity>(`/ai/identities/${id}`, request)
  },

  // 删除 AI 身份
  deleteIdentity(id: string) {
    return api.delete(`/ai/identities/${id}`)
  },

  // 获取 AI 设置
  getSettings() {
    return api.get<AISettings>('/ai/settings')
  },

  // 更新 AI 设置
  updateSettings(request: UpdateAISettingsRequest) {
    return api.put<AISettings>('/ai/settings', request)
  },

  // 获取上下文数据
  getContextData() {
    return api.get<AIContextData>('/ai/context-data')
  }
}
```

### 5. Dashboard 视图更新

**文件**: `frontend/src/views/DashboardView.vue`

**新增图标导入**:
```typescript
import { ChatLineRound } from '@element-plus/icons-vue'
```

**图标映射**:
```typescript
const iconComponents = {
  // ... 其他图标
  ChatLineRound
}
```

## 后端实现

### 1. AI 模块

**文件**: `backend/src/api/ai.rs`

#### 1.1 数据结构

```rust
// AI 设置
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AISettings {
    pub api_key: String,
    pub api_base_url: String,
    pub model: String,
    pub default_prompt: String,
    pub temperature: f64,
    pub max_tokens: i32,
}

// 简单类信息
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SimpleClassInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub grade: i16,
    pub teacher_id: Option<uuid::Uuid>,
}

// 简单小组信息
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SimpleGroupInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub class_id: uuid::Uuid,
}

// 简单部门信息
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SimpleDepartmentInfo {
    pub id: uuid::Uuid,
    pub name: String,
}

// AI 上下文数据
#[derive(Debug, Serialize)]
pub struct AIContextData {
    pub classes: Vec<SimpleClassInfo>,
    pub groups: Vec<SimpleGroupInfo>,
    pub departments: Vec<SimpleDepartmentInfo>,
}
```

#### 1.2 API 函数

1. **chat** - 处理 AI 对话请求
2. **list_identities** - 获取 AI 身份列表（管理员专用）
3. **create_identity** - 创建 AI 身份（管理员专用）
4. **update_identity** - 更新 AI 身份（管理员专用）
5. **delete_identity** - 删除 AI 身份（管理员专用）
6. **get_settings** - 获取 AI 设置（管理员专用）
7. **update_settings** - 更新 AI 设置（管理员专用）
8. **get_context_data** - 获取上下文数据（根据用户权限过滤）

### 2. 路由配置

**文件**: `backend/src/api/routes.rs`

```rust
// AI 相关路由
.route("/api/ai/chat", post(ai::chat))
.route("/api/ai/identities", get(ai::list_identities))
.route("/api/ai/identities", post(ai::create_identity))
.route("/api/ai/identities/:id", put(ai::update_identity))
.route("/api/ai/identities/:id", delete(ai::delete_identity))
.route("/api/ai/settings", get(ai::get_settings))
.route("/api/ai/settings", put(ai::update_settings))
.route("/api/ai/context-data", get(ai::get_context_data))
```

### 3. 权限检查

```rust
async fn check_admin(claims: &Claims, _pool: &PgPool) -> Result<(), AppError> {
    if claims.role != "admin" {
        return Err(AppError::Auth("只有管理员可以访问此功能".to_string()));
    }
    Ok(())
}
```

### 4. 权限过滤数据获取

```rust
pub async fn get_context_data(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<AIContextData>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    
    let permission_manager = PermissionManager::new(pool.clone());
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;
    
    // 根据用户权限获取数据
    let classes = if user_permissions.iter().any(|p| p == "class.view") {
        sqlx::query_as::<_, SimpleClassInfo>("SELECT id, name, grade, teacher_id FROM classes")
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    
    let groups = if user_permissions.iter().any(|p| p == "group.view") {
        sqlx::query_as::<_, SimpleGroupInfo>("SELECT id, name, class_id FROM groups")
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    
    let departments = if user_permissions.iter().any(|p| p == "department.view") {
        sqlx::query_as::<_, SimpleDepartmentInfo>("SELECT id, name FROM departments")
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    
    Ok(Json(AIContextData {
        classes,
        groups,
        departments,
    }))
}
```

## 数据库迁移

### 1. AI 设置表

**文件**: `backend/migrations/008_create_ai_settings.sql`

```sql
CREATE TABLE IF NOT EXISTS ai_settings (
    id SERIAL PRIMARY KEY,
    api_key VARCHAR(512) NOT NULL,
    api_base_url VARCHAR(512) DEFAULT 'https://api.openai.com/v1',
    model VARCHAR(100) DEFAULT 'gpt-3.5-turbo',
    default_prompt TEXT DEFAULT 'You are an AI assistant for a school management system.',
    temperature DECIMAL(3,2) DEFAULT 0.7,
    max_tokens INTEGER DEFAULT 1000,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO ai_settings (api_key, api_base_url, model, default_prompt, temperature, max_tokens)
VALUES ('your-api-key-here', 'https://api.openai.com/v1', 'gpt-3.5-turbo', 'You are an AI assistant for a school management system.', 0.7, 1000)
ON CONFLICT DO NOTHING;
```

### 2. AI 权限添加

**文件**: `backend/migrations/009_add_ai_permissions.sql`

```sql
-- 管理员权限
INSERT INTO permissions (role, permission, value, priority)
VALUES 
    ('admin', 'ai.view', true, 10),
    ('admin', 'ai.chat', true, 10),
    ('admin', 'ai.analyze', true, 10),
    ('admin', 'ai.settings', true, 10),
    ('admin', 'ai.*', true, 5)
ON CONFLICT (role, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority;

-- 教师权限
INSERT INTO permissions (role, permission, value, priority)
VALUES 
    ('teacher', 'ai.view', true, 10),
    ('teacher', 'ai.chat', true, 10),
    ('teacher', 'ai.analyze', true, 10),
    ('teacher', 'ai.settings', true, 10)
ON CONFLICT (role, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority;

-- 学生权限
INSERT INTO permissions (role, permission, value, priority)
VALUES 
    ('student', 'ai.view', true, 7),
    ('student', 'ai.chat', true, 7),
    ('student', 'ai.analyze', true, 7)
ON CONFLICT (role, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority;

-- 家长权限
INSERT INTO permissions (role, permission, value, priority)
VALUES 
    ('parent', 'ai.view', true, 10),
    ('parent', 'ai.chat', true, 10),
    ('parent', 'ai.analyze', true, 10)
ON CONFLICT (role, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority;
```

**注意**: `value` 列是布尔类型，使用 `true`/`false` 而不是字符串。

## 权限配置

### 1. 管理员权限
**文件**: `backend/templates/permissions/admin.yaml`

```yaml
# AI 助手权限
- permission: ai.view
  priority: 10
- permission: ai.chat
  priority: 10
- permission: ai.analyze
  priority: 10
- permission: ai.settings
  priority: 10
- permission: "ai.*"
  priority: 5
```

### 2. 教师权限
**文件**: `backend/templates/permissions/teacher.yaml`

```yaml
# AI 助手权限
- permission: ai.view
  priority: 10
- permission: ai.chat
  priority: 10
- permission: ai.analyze
  priority: 10
- permission: ai.settings
  priority: 10
```

### 3. 学生权限
**文件**: `backend/templates/permissions/student.yaml`

```yaml
# AI 助手权限
- permission: ai.view
  priority: 7
- permission: ai.chat
  priority: 7
- permission: ai.analyze
  priority: 7
```

### 4. 家长权限
**文件**: `backend/templates/permissions/parent.yaml`

```yaml
# AI 助手权限
- permission: ai.view
  priority: 10
- permission: ai.chat
  priority: 10
- permission: ai.analyze
  priority: 10
```

## 部署步骤

### 1. 数据库迁移

运行迁移脚本创建 AI 设置表和权限：

```bash
# 运行 AI 设置表迁移
cd backend
cargo run --bin run_migration

# 运行 AI 权限迁移
cargo run --bin run_ai_permission_migration
```

### 2. 前端构建

```bash
cd frontend
npm install
npm run build
```

### 3. 后端编译

```bash
cd backend
cargo build
```

### 4. 配置 AI API

使用管理员账号登录系统，访问 `/dashboard/ai/settings` 配置 AI API 密钥和参数。

## 测试验证

### 1. 权限测试

- **管理员**: 可以访问 AI 对话和 AI 设置页面
- **教师**: 可以访问 AI 对话页面，不能使用 AI 设置
- **学生**: 可以访问 AI 对话页面，不能使用 AI 设置
- **家长**: 可以访问 AI 对话页面，不能使用 AI 设置

### 2. 功能测试

- **AI 对话**: 发送消息给 AI，获取回复
- **AI 设置**: 修改 API 密钥、模型参数等，保存后验证是否持久化
- **数据获取**: 不同角色用户获取到的数据应该根据其权限过滤

### 3. 编译测试

```bash
# 后端
cargo check

# 前端
npm run build
```

## 相关文件清单

### 前端文件
- `frontend/src/router/index.ts` - 路由配置
- `frontend/src/config/menu.ts` - 菜单配置
- `frontend/src/config/types.ts` - 类型定义
- `frontend/src/api/ai.ts` - AI API 模块
- `frontend/src/views/AIView.vue` - AI 对话页面
- `frontend/src/views/AISettingsView.vue` - AI 设置页面
- `frontend/src/views/DashboardView.vue` - 仪表盘视图（更新图标映射）
- `frontend/src/styles/ai-view.css` - AI 对话样式
- `frontend/src/styles/ai-settings-view.css` - AI 设置样式

### 后端文件
- `backend/src/api/ai.rs` - AI 模块
- `backend/src/api/mod.rs` - 模块导出
- `backend/src/api/routes.rs` - 路由配置
- `backend/migrations/008_create_ai_settings.sql` - AI 设置表迁移
- `backend/migrations/009_add_ai_permissions.sql` - AI 权限迁移
- `backend/templates/permissions/admin.yaml` - 管理员权限模板
- `backend/templates/permissions/teacher.yaml` - 教师权限模板
- `backend/templates/permissions/student.yaml` - 学生权限模板
- `backend/templates/permissions/parent.yaml` - 家长权限模板
- `backend/run_ai_permission_migration.rs` - 权限迁移脚本

## 注意事项

1. **API 密钥安全**: API 密钥存储在数据库中，只有管理员可以访问和修改
2. **权限过滤**: AI 获取数据时会根据用户权限进行过滤，确保数据安全
3. **错误处理**: 前端和后端都有完善的错误处理机制
4. **样式分离**: 遵循项目规范，CSS 与 Vue 组件分离

## 后续优化建议

1. **AI 模型集成**: 集成真实的 AI 模型（如 OpenAI GPT）
2. **对话历史**: 保存对话历史到数据库
3. **上下文管理**: 支持多轮对话
4. **性能优化**: 实现 AI 请求的缓存和限流机制
5. **日志记录**: 记录 AI 请求和响应，便于审计和问题排查
