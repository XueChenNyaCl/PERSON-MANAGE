# 学校综合管理系统

## 项目简介
学校综合管理系统是一个基于前后端分离架构的现代化管理系统，支持人员管理、考勤管理、评分管理、通知管理等功能。

## 技术栈
- **前端**：Vue 3 + TypeScript + Vite + Element Plus
- **后端**：Rust + Axum + SQLx + PostgreSQL
- **认证**：JWT
- **样式**：Element Plus

## 快速开始

### 环境要求
- Node.js (v16+)
- Rust (v1.60+)
- PostgreSQL (v14+)

### 安装依赖

1. 安装前端依赖
```bash
cd frontend
npm install
```

2. 安装后端依赖（自动处理）
```bash
cd backend
cargo build
```

3. 安装根目录依赖
```bash
npm install
```

### 配置数据库

1. 确保PostgreSQL已启动
2. 创建数据库
```sql
CREATE DATABASE example_db;
CREATE USER admin WITH PASSWORD 'password';
GRANT ALL PRIVILEGES ON DATABASE example_db TO admin;
```

### 启动项目

```bash
npm run dev
```

这会同时启动前端开发服务器和后端API服务器。

- 前端地址：http://localhost:5173
- 后端地址：http://localhost:3000

### 构建项目

```bash
npm run build
```

## 项目结构

```
school-management-system/
├── frontend/             # 前端项目
│   ├── src/              # 前端源代码
│   │   ├── api/          # API调用
│   │   ├── views/        # 页面组件
│   │   ├── router/       # 路由配置
│   │   └── ...
│   ├── package.json      # 前端依赖
│   └── ...
├── backend/              # 后端项目
│   ├── src/              # 后端源代码
│   │   ├── api/          # API处理函数
│   │   ├── core/         # 核心模块
│   │   ├── models/       # 数据模型
│   │   └── ...
│   ├── Cargo.toml        # 后端依赖
│   └── ...
├── docs/                 # 项目文档
├── support/              # PostgreSQL支持文件
├── package.json          # 根目录配置（整合启动命令）
├── README.md             # 项目说明
└── ...
```

## 主要功能

1. **人员管理**：学生/教职工档案管理
2. **考勤管理**：到校/离校记录、课堂考勤
3. **评分管理**：个人分、小组分管理
4. **通知管理**：学校公告、班级通知
5. **家长端**：查看孩子考勤、分数
6. **一体机端**：班级展示、学生自助签到

## 核心API

### 班级管理
- `GET /api/classes` - 获取班级列表
- `POST /api/classes` - 创建班级
- `GET /api/classes/:id` - 获取单个班级
- `PUT /api/classes/:id` - 更新班级
- `DELETE /api/classes/:id` - 删除班级
- `GET /api/classes/:id/students` - 获取班级学生列表
- `GET /api/classes/:id/teachers` - 获取班级教师列表

### 认证管理
- `POST /api/auth/login` - 用户登录
- `POST /api/auth/logout` - 用户登出
- `GET /api/auth/me` - 获取当前用户信息

### 评分管理
- `GET /api/scores` - 获取评分列表
- `POST /api/scores` - 创建评分
- `GET /api/scores/:id` - 获取单个评分
- `PUT /api/scores/:id` - 更新评分
- `DELETE /api/scores/:id` - 删除评分

## 开发指南

### 前端开发
- 代码位于 `frontend/src/` 目录
- 页面组件位于 `frontend/src/views/` 目录
- 路由配置位于 `frontend/src/router/index.ts`
- API调用位于 `frontend/src/api/` 目录

### 后端开发
- 代码位于 `backend/src/` 目录
- API处理函数位于 `backend/src/api/` 目录
- 核心模块位于 `backend/src/core/` 目录
- 数据模型位于 `backend/src/models/` 目录

### 数据库迁移
- 数据库迁移文件位于 `backend/migrations/` 目录
- 使用 SQLx 的离线迁移功能

## 注意事项

1. 开发环境使用的数据库配置为：
   - 数据库名：example_db
   - 用户名：admin
   - 密码：password

2. 生产环境请修改 `.env` 文件中的配置，尤其是 JWT_SECRET。

3. 前端开发服务器默认端口为 5173，后端 API 服务器默认端口为 3000。

## 许可证
MIT

## 贡献
欢迎贡献代码、报告问题或提出建议！

## 联系方式
如有问题，请联系项目维护者。