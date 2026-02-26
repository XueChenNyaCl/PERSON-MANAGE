-- 权限系统迁移
-- 创建权限表，支持LuckPerms风格的权限管理

-- 启用UUID扩展（如果尚未启用）
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 权限表：存储角色权限节点
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role VARCHAR(50) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(role, permission)
);

-- 用户特定权限表（可选，用于覆盖角色权限）
CREATE TABLE IF NOT EXISTS user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    permission VARCHAR(255) NOT NULL,
    value BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER DEFAULT 100,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT fk_user_permissions_user_id 
        FOREIGN KEY (user_id) 
        REFERENCES persons(id) 
        ON DELETE CASCADE,
    UNIQUE(user_id, permission)
);

-- 索引设计
CREATE INDEX IF NOT EXISTS idx_permissions_role ON permissions(role);
CREATE INDEX IF NOT EXISTS idx_permissions_role_permission ON permissions(role, permission);
CREATE INDEX IF NOT EXISTS idx_permissions_permission ON permissions(permission);
CREATE INDEX IF NOT EXISTS idx_user_permissions_user_id ON user_permissions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_permissions_user_id_permission ON user_permissions(user_id, permission);