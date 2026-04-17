# 权限与人员类别优化实施计划

## 一、需求概述

### 1.1 核心需求
1. **特殊用户系统**：新增 `system` 和 `admin` 用户类型
2. **Admin 用户行为优化**：当 admin 人员被注销（对应投射如 `000000000000`）后，不再允许从网页登录，身份挂载到标有 admin 权限的用户上
3. **System 用户**：程序操作视为 `system` 用户在操作（如连接服务器、开放端口），`system` 不可登录
4. **日志标识格式**：
   - `[time][system][open server][open the server on 3000]`
   - `[time][admin:username][creat person][creat the person {name}……]`
   - `[time][501001][creat person][creat the person {name}……]`
5. **AI 用户**：创建 `SysAI` 和 `ChatAI` 用户，`[ChatAI:user]` 记录用户让聊天 AI 做的事
6. **前端特殊用户 Tab**：在人员管理界面添加 Tab 栏切换到特殊用户（鉴权），可创建如 `[loT:id]`、`[scerm:id]` 等类型用户

### 1.2 现有系统分析
- **数据库**：使用 `persons` 表存储用户，有 `username`、`password_hash`、`role` 等字段
- **权限系统**：`permissions` 表存储角色权限，`user_permissions` 表存储用户特定权限
- **认证**：`/api/auth/login` 接口，支持 `admin` 特殊登录
- **前端**：Vue 3 + Element Plus，`PersonView.vue` 管理页面

---

## 二、系统架构设计

### 2.1 用户类型分类

| 用户类型 | 标识格式 | 说明 | 可登录 |
|---------|---------|------|-------|
| 正常人员 | `501001` | 普通人员 ID | 是 |
| Admin 人员 | `[admin:username]` | 挂在 admin 权限的人员 | 是（身份转为 admin） |
| System | `system` | 程序操作 | 否 |
| ChatAI | `[ChatAI:user]` | AI 操作记录 | 暂留 |
| SysAI | `SysAI` | 暂留 | 暂留 |
| 物联网用户 | `[loT:id]` | 物联网设备 | 是 |
| 大屏用户 | `[scerm:id]` | 大屏展示 | 是 |

### 2.2 Admin 用户特殊处理逻辑

```
admin 用户登录流程：
1. 用户使用 admin 账号登录
2. 检查 persons 表中是否有 id='00000000-0000-0000-0000-000000000000' 的记录
3. 如果该记录存在且 is_active=true：
   - 允许登录，操作日志显示为 [admin:username]
4. 如果该记录不存在或 is_active=false：
   - 检查当前用户是否标有 admin 权限（role='admin' 或有 system.settings 权限）
   - 如果有：允许登录，操作日志显示为 [admin:username]
   - 如果没有：拒绝登录
```

### 2.3 日志记录格式

```rust
// 系统操作日志
[2024-01-01 12:00:00][system][open server][open the server on 3000]

// Admin 用户操作日志
[2024-01-01 12:00:00][admin:zhangsan][create person][create person {name: 李四}]

// 普通用户操作日志
[2024-01-01 12:00:00][501001][create person][create person {name: 王五}]

// ChatAI 操作日志
[2024-01-01 12:00:00][ChatAI:user_id][ai action][user asked to ...]
```

---

## 三、后端实施计划

### 3.1 数据库变更

#### 新增表：`special_users`（特殊用户表）
```sql
CREATE TABLE special_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_type VARCHAR(20) NOT NULL,  -- 'system', 'iot', 'scerm', 'sysai', 'chatai'
    identifier VARCHAR(100) NOT NULL,  -- 如 'system', 'iot:device001', 'scerm:screen01'
    linked_person_id UUID,  -- 关联的人员 ID（可选）
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_type, identifier)
);
```

#### 修改表：`persons`
```sql
-- 新增字段
ALTER TABLE persons ADD COLUMN IF NOT EXISTS is_admin_user BOOLEAN DEFAULT false;
ALTER TABLE persons ADD COLUMN IF NOT EXISTS linked_admin_id UUID REFERENCES persons(id);
```

#### 新增表：`操作日志`（可选，先用 println）
```sql
CREATE TABLE operation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id UUID,  -- 操作者 ID，system 为 NULL
    operator_type VARCHAR(20),  -- 'system', 'admin', 'user', 'chatai'
    operator_name VARCHAR(100),  -- 如 'admin:zhangsan' 或 '501001'
    action VARCHAR(100),  -- 操作类型
    details JSONB,  -- 详细信息
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### 3.2 后端模块变更

#### 3.2.1 新建文件
- `backend/src/core/special_user.rs` - 特殊用户管理
- `backend/src/core/operation_logger.rs` - 操作日志记录
- `backend/src/models/special_user.rs` - 特殊用户数据模型

#### 3.2.2 修改文件

**`backend/src/api/auth.rs`**
- 修改 `login` 函数：添加 admin 用户特殊处理逻辑
- 检查 `00000000-0000-0000-0000-000000000000` 是否存在
- 返回 `operator_type` 用于日志标识

**`backend/src/core/auth.rs`**
- Claims 结构体添加 `operator_type` 字段
- Token 生成时包含操作者类型信息

**`backend/src/models/person.rs`**
- 添加 `is_admin_user` 和 `linked_admin_id` 字段

**`backend/src/api/person.rs`**
- 在创建/更新/删除操作时记录操作日志

### 3.3 API 接口变更

#### 新增接口

| 接口 | 方法 | 功能 | 鉴权 |
|-----|------|------|-----|
| `/api/special-users` | GET | 获取特殊用户列表 | admin |
| `/api/special-users` | POST | 创建特殊用户 | admin |
| `/api/special-users/:id` | DELETE | 删除特殊用户 | admin |
| `/api/special-users/:id/link` | POST | 关联人员 | admin |
| `/api/special-users/iot/login` | POST | IoT 用户登录 | - |
| `/api/special-users/scerm/login` | POST | 大屏用户登录 | - |
| `/api/operation-logs` | GET | 获取操作日志 | admin |
| `/api/persons/:id/link-admin` | POST | 人员关联 admin | admin |

### 3.4 实施步骤（后端）

1. **数据库迁移脚本**
   - 创建 `migrations/002_special_users.sql`
   - 包含 `special_users` 表创建
   - 包含 `operation_logs` 表创建
   - 包含 `persons` 表字段添加

2. **特殊用户管理模块**
   - 实现 `special_user.rs`：查询、创建、删除特殊用户
   - 实现 `operation_logger.rs`：统一日志记录接口

3. **修改登录逻辑**
   - 在 `auth.rs` 中添加 admin 特殊处理
   - 检查预留 admin ID 的状态

4. **添加日志记录**
   - 在关键操作（person CRUD）添加日志记录宏
   - 实现统一的日志格式

---

## 四、前端实施计划

### 4.1 页面变更

#### 4.1.1 `PersonView.vue` 修改
- 添加新的 Tab：`特殊用户`
- 在 `特殊用户` Tab 中：
  - 显示 `system`、`iot`、`scerm` 等特殊用户列表
  - 创建特殊用户的表单（选择类型、输入标识符）
  - 删除特殊用户（需要确认）
  - 关联人员操作

#### 4.1.2 新增组件
- `SpecialUserTab.vue` - 特殊用户管理 Tab 组件
- `SpecialUserForm.vue` - 创建/编辑特殊用户表单
- `SpecialUserList.vue` - 特殊用户列表组件

### 4.2 API 变更

#### 新增前端 API
- `frontend/src/api/specialUser.ts` - 特殊用户 API 封装

```typescript
interface SpecialUserResponse {
  id: string;
  user_type: 'system' | 'iot' | 'scerm' | 'sysai' | 'chatai';
  identifier: string;
  linked_person_id?: string;
  linked_person_name?: string;
  description?: string;
  is_active: boolean;
}

export const specialUserApi = {
  list: () => api.get<SpecialUserResponse[]>('/special-users'),
  create: (data: CreateSpecialUser) => api.post('/special-users', data),
  delete: (id: string) => api.delete(`/special-users/${id}`),
  linkPerson: (id: string, personId: string) => api.post(`/special-users/${id}/link`, { person_id: personId }),
};
```

### 4.3 实施步骤（前端）

1. **创建 API 文件**
   - `frontend/src/api/specialUser.ts`

2. **创建组件**
   - `frontend/src/components/SpecialUserTab.vue`
   - `frontend/src/components/SpecialUserForm.vue`

3. **修改 PersonView.vue**
   - 导入并添加特殊用户 Tab
   - 实现 Tab 切换逻辑
   - 添加特殊用户列表和操作按钮

4. **样式调整**
   - 如需要，在 `person-view.css` 中添加特殊用户相关样式

---

## 五、实施顺序

### 第一阶段：后端基础（优先级：高）
1. 数据库迁移脚本
2. 特殊用户数据模型
3. 基础 CRUD API

### 第二阶段：后端认证与日志（优先级：高）
1. 修改登录逻辑（admin 特殊处理）
2. 操作日志模块
3. 日志记录宏

### 第三阶段：前端实现（优先级：中）
1. API 封装
2. 特殊用户管理组件
3. Tab 栏集成

### 第四阶段：集成测试（优先级：高）
1. 后端 API 测试
2. 前端功能测试
3. 登录流程测试

---

## 六、注意事项

1. **Admin 预留 ID**：`00000000-0000-0000-0000-000000000000` 已存在且作为 admin 使用
2. **System 用户不可登录**：登录接口需要过滤 `system` 类型用户
3. **鉴权要求**：创建/删除特殊用户需要 admin 权限
4. **向后兼容**：现有功能不受影响，新功能逐步添加

---

## 七、文件清单

### 后端新增文件
```
backend/src/models/special_user.rs
backend/src/core/special_user.rs
backend/src/core/operation_logger.rs
migrations/002_special_users.sql
```

### 后端修改文件
```
backend/src/api/auth.rs
backend/src/core/auth.rs
backend/src/models/person.rs
backend/src/api/person.rs
backend/src/api/routes.rs
backend/src/api/mod.rs
```

### 前端新增文件
```
frontend/src/api/specialUser.ts
frontend/src/components/SpecialUserTab.vue
```

### 前端修改文件
```
frontend/src/views/PersonView.vue
frontend/src/language/zh-CN.json
```

---

## 八、测试用例

### 8.1 登录测试
- [ ] 正常人员使用 username/password 登录
- [ ] admin 人员登录，日志显示 `[admin:username]`
- [ ] system 用户尝试登录被拒绝
- [ ] 被注销的 admin (id='000000000000') 尝试登录被拒绝

### 8.2 特殊用户测试
- [ ] 创建 IoT 用户
- [ ] 创建 Scerm 用户
- [ ] 关联人员到特殊用户
- [ ] 删除特殊用户

### 8.3 日志测试
- [ ] 系统操作日志格式正确
- [ ] admin 用户操作日志带有 `[admin:username]` 前缀
- [ ] 普通用户操作日志只显示 ID
