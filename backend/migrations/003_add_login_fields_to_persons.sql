-- 添加登录相关字段到 persons 表
ALTER TABLE persons 
ADD COLUMN username VARCHAR(50) UNIQUE,
ADD COLUMN password_hash VARCHAR(255),
ADD COLUMN role VARCHAR(20) DEFAULT 'user',
ADD COLUMN is_active BOOLEAN DEFAULT true,
ADD COLUMN last_login_at TIMESTAMP WITH TIME ZONE;

-- 用户名索引（登录时快速查找）
CREATE INDEX idx_persons_username ON persons(username);

-- 角色索引（权限查询优化）
CREATE INDEX idx_persons_role ON persons(role);

-- 预留管理员账户
INSERT INTO persons (id, name, username, password_hash, role, type, is_active) 
VALUES (
    '00000000-0000-0000-0000-000000000000',
    '系统管理员',
    'admin',
    -- bcrypt hash of 'admin' (cost 12)
    '$2b$12$LQv3c1yqBWVHxpd5g6TAkO6l4dQjHZjXlWfLp.aC.9r7t4bJF1WKK',
    'admin',
    'teacher',
    true
) ON CONFLICT (id) DO NOTHING;