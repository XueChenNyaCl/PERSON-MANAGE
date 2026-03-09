# 智联校园多维管理平台

一个面向校园场景的综合管理平台，采用前后端分离架构，支持人员管理、班级管理、考勤管理、评分管理、通知管理等核心业务，适用于本地部署、比赛展示和功能演示。

## 项目概览

- **项目类型**：前后端分离 Web 管理系统
- **适用场景**：学校日常管理、功能展示、课程设计、比赛作品提交
- **前端地址**：`http://localhost:3001`
- **后端地址**：`http://localhost:3000`
- **默认数据库**：PostgreSQL
- **认证方式**：JWT

## 主要功能

### 基础业务模块
- **人员管理**：学生、教师、家长等人员信息维护
- **班级管理**：班级信息、班主任、成员关联管理
- **部门管理**：学校组织结构管理
- **考勤管理**：考勤记录录入与查询
- **评分管理**：个人分、小组分等评分记录管理
- **通知管理**：学校公告、班级通知发布

### 系统能力
- **登录认证**：基于用户名密码登录与 JWT 鉴权
- **权限控制**：支持角色权限与模板化权限初始化
- **数据库迁移**：支持初始化数据库结构与基础数据
- **打包发布**：支持 Windows 环境一键整理发布目录

## 技术栈

### 前端
- Vue 3
- TypeScript
- Vite
- Element Plus
- Pinia
- Vue Router
- Axios

### 后端
- Rust
- Axum
- SQLx
- PostgreSQL
- JSON Web Token（JWT）

## 目录结构

```text
.
├── frontend/                          # 前端工程（Vue3 + TS）
│   ├── src/
│   │   ├── App.vue                   # 根组件
│   │   ├── main.ts                   # 前端入口
│   │   ├── api/                      # 接口请求封装
│   │   ├── router/                   # 路由模块
│   │   ├── store/                    # Pinia 状态管理（兼容目录）
│   │   ├── stores/                   # Pinia 状态管理
│   │   ├── views/                    # 页面视图模块
│   │   ├── components/               # 通用组件模块
│   │   ├── composables/              # 组合式函数模块
│   │   ├── utils/                    # 前端工具模块
│   │   ├── types/                    # 类型声明模块
│   │   ├── language/                 # 国际化模块
│   │   ├── config/                   # 配置模块
│   │   ├── assets/                   # 静态资源
│   │   └── styles/                   # 样式模块
│   └── package.json                  # 前端依赖配置
├── backend/                           # Rust 后端工程（Axum + SQLx）
│   ├── src/
│   │   ├── main.rs                   # 后端入口
│   │   ├── api/                      # 路由与接口模块
│   │   ├── core/                     # 核心模块（配置/鉴权/数据库）
│   │   ├── models/                   # 数据模型模块
│   │   ├── plugins/                  # 插件扩展模块
│   │   ├── ws/                       # WebSocket 模块
│   │   ├── utils/                    # 工具函数模块
│   │   └── bin/                      # 可执行子命令模块
│   ├── migrations/                   # SQL 迁移文件
│   ├── templates/permissions/        # 权限模板
│   ├── run_migration.rs              # 迁移执行入口
│   └── Cargo.toml                    # 后端依赖配置
├── static/                            # 根目录静态资源
├── templates/permissions/             # 根目录权限模板
├── docs/                              # 项目文档
├── scripts/                           # 构建/发布脚本
├── package.json                       # 根脚本配置（联动前后端）
└── README.md                          # 项目说明文档
```

## 环境要求

建议在 Windows 10 / Windows 11 64 位系统下运行。

### 必要软件
- Node.js `24`
- npm
- Rust `1.60+`
- Cargo
- PostgreSQL `18`

### 推荐工具
- Visual Studio Code
- Chrome / Edge 浏览器
- pgAdmin 或其他 PostgreSQL 客户端

## 快速开始

## 1. 安装依赖

在项目根目录执行：

```bash
npm install
```

该命令会安装根目录依赖，并自动安装前端依赖、编译后端依赖。

## 2. 配置环境变量

项目根目录已提供 `.env` 配置，核心内容如下：

```env
DATABASE_URL=postgres://postgres:root@localhost:5432/example_db
JWT_SECRET=your_jwt_secret_key_here_please_change_in_production
JWT_EXPIRES_IN=24h
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

如需修改数据库账号、密码或端口，请同步更新 `DATABASE_URL`。

## 3. 启动 PostgreSQL

示例命令：

```bash
# 请在命令行中运行（cmd）
net start postgresql-x64-18
```

> 实际服务名可能因本机 PostgreSQL 版本不同而变化，请以本机安装的服务名为准。

## 4. 创建数据库

推荐创建项目使用的数据库：

```sql
CREATE DATABASE example_db;
```

如果你需要单独创建数据库用户，也可以按实际环境配置。当前 `.env` 默认使用：

- 用户名：`postgres`
- 密码：`root`
- 数据库：`example_db`

## 5. 初始化数据库

在 `backend` 目录执行：

```bash
cargo run --bin run_migration -- all
```

该命令会完成：

- 数据库结构初始化
- 基础数据导入
- 权限模板初始化

## 6. 启动开发环境

在项目根目录执行：

```bash
npm run dev
```

启动后将同时运行前后端服务：

- 前端开发服务器：`http://localhost:3001`
- 后端 API 服务：`http://localhost:3000`

其中前端已通过 `Vite` 配置代理 `/api` 到后端服务。

## 默认账号信息

系统内置管理员账号：

- 用户名：`admin`
- 密码：`admin`
- 角色：`admin`

> 首次展示或部署完成后，建议尽快修改默认密码与密钥配置。

## 常用命令

### 开发命令

```bash
# 同时启动前后端
npm run dev

# 单独启动前端
npm run dev:frontend

# 单独启动后端
npm run dev:backend
```

### 构建命令

```bash
# 构建前后端
npm run build

# 仅构建前端
npm run build:frontend

# 仅构建后端（release）
npm run build:backend
```

说明：

- 后端 release 构建产物通常位于 `backend/target/release/` 或工作区对应的 Cargo 输出目录
- 前端构建产物会输出到 `newnewnew/static/`

### 检查命令

```bash
# 前端检查
npm run lint

# 后端格式检查
cargo fmt --all -- --check

# 后端 Clippy 检查
cargo clippy --workspace -- -D warnings
```

## 打包发布

项目提供 Windows 一键打包脚本：`scripts/build_release.bat`

### 使用方式

在项目根目录执行：

```bash
scripts\build_release.bat
```

执行成功后，会生成发布目录：`newnewnew`

### 关键发布文件

- `newnewnew/school-management-backend.exe`
- `newnewnew/migration.exe`
- `newnewnew/static/`
- `newnewnew/migrations/`
- `newnewnew/templates/permissions/`

### 发布运行步骤

1. 先运行 `newnewnew\migration.exe all`
2. 再运行 `newnewnew\school-management-backend.exe`
3. 服务默认监听端口为 `3000`

前端静态资源由后端统一托管，发布时通常只需部署 `newnewnew` 目录。

## 核心接口示例

### 认证接口
- `POST /api/auth/login`：用户登录
- `POST /api/auth/logout`：用户登出
- `GET /api/auth/me`：获取当前用户信息

### 班级接口
- `GET /api/classes`：获取班级列表
- `POST /api/classes`：创建班级
- `GET /api/classes/:id`：获取班级详情
- `PUT /api/classes/:id`：更新班级
- `DELETE /api/classes/:id`：删除班级

### 评分接口
- `GET /api/scores`：获取评分列表
- `POST /api/scores`：创建评分记录
- `GET /api/scores/:id`：获取评分详情
- `PUT /api/scores/:id`：更新评分记录
- `DELETE /api/scores/:id`：删除评分记录

## 开发说明

### 前端开发
- 入口文件：`frontend/src/main.ts`
- 路由配置：`frontend/src/router/`
- 页面视图：`frontend/src/views/`
- 接口封装：`frontend/src/api/`
- 状态管理：`frontend/src/stores/` 与 `frontend/src/store/`

### 后端开发
- 主入口：`backend/src/main.rs`
- 接口层：`backend/src/api/`
- 核心模块：`backend/src/core/`
- 数据模型：`backend/src/models/`
- 迁移入口：`backend/run_migration.rs`

### 权限模板
- 权限模板目录：`backend/templates/permissions/`
- 支持角色：`admin`、`teacher`、`student`、`parent`

## 常见问题

### 1. 数据库连接失败
请检查以下内容：

- PostgreSQL 服务是否已启动
- `.env` 中 `DATABASE_URL` 是否正确
- 数据库 `example_db` 是否已创建
- 用户名和密码是否与本地环境一致

### 2. 前端接口请求失败
请确认：

- 后端服务是否已在 `3000` 端口正常启动
- 前端是否通过 `3001` 端口访问
- `frontend/vite.config.ts` 中的代理配置是否生效

### 3. 迁移执行失败
请确认：

- 数据库连接配置正确
- 当前账号具备建表权限
- `backend/migrations/001_initial_schema.sql` 文件存在

### 4. 打包后无法启动
请检查：

- 是否先执行了 `migration.exe`
- 发布目录中的 `.env` 是否存在且配置正确
- 端口是否被占用

## 注意事项

1. 开发环境默认端口为：前端 `3001`，后端 `3000`。
2. 生产或演示环境请修改默认的 `JWT_SECRET`、数据库密码和管理员密码。
3. 如端口冲突，请调整 `.env` 与相关代理配置。

## 许可证

MIT

## 贡献

欢迎提交问题、改进建议和功能优化方案。
 *仓库链接*
`https://github.com/XueChenNyaCl/PERSON-MANAGE`