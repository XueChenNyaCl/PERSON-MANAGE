-- 启用 UUID 扩展（如果尚未启用）
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 启用 trigram 扩展以支持模糊搜索
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- 创建更新 updated_at 自动更新触发器
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 1. 基础表结构

-- persons 表（人员基础信息）
CREATE TABLE IF NOT EXISTS persons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    gender SMALLINT NOT NULL DEFAULT 0, -- 0:未知 1:男 2:女
    birthday DATE,
    phone VARCHAR(20),
    email VARCHAR(100),
    type VARCHAR(20) NOT NULL, -- 'student', 'teacher', 'parent'
    username VARCHAR(50) UNIQUE,
    password_hash VARCHAR(255),
    role VARCHAR(20) DEFAULT 'user',
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- departments 表（部门）
CREATE TABLE IF NOT EXISTS departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    parent_id UUID, -- 自引用外键
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- classes 表（班级）
CREATE TABLE IF NOT EXISTS classes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    grade SMALLINT NOT NULL, -- 年级，如 1 表示一年级
    teacher_id UUID, -- 关联班主任
    academic_year VARCHAR(20), -- 学年，如 "2025-2026"
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- teachers 表（教职工扩展信息）
CREATE TABLE IF NOT EXISTS teachers (
    person_id UUID PRIMARY KEY,
    employee_no VARCHAR(50) UNIQUE NOT NULL,
    department_id UUID, -- 关联部门
    title VARCHAR(50), -- 职称（高级教师、一级教师等）
    hire_date DATE
);

-- students 表（学生扩展信息）
CREATE TABLE IF NOT EXISTS students (
    person_id UUID PRIMARY KEY,
    student_no VARCHAR(50) UNIQUE NOT NULL,
    class_id UUID, -- 关联班级
    enrollment_date DATE,
    status VARCHAR(20) DEFAULT 'enrolled' -- enrolled, graduated, transferred
);

-- parents 表（家长扩展信息）
CREATE TABLE IF NOT EXISTS parents (
    person_id UUID PRIMARY KEY,
    wechat_openid VARCHAR(100), -- 可选，用于推送
    occupation VARCHAR(100)
);

-- student_parent 关联表（学生-家长关系）
CREATE TABLE IF NOT EXISTS student_parent (
    student_id UUID,
    parent_id UUID,
    relationship VARCHAR(20), -- father, mother, guardian
    PRIMARY KEY (student_id, parent_id)
);

-- teacher_class 关联表（教师-班级关系）
CREATE TABLE IF NOT EXISTS teacher_class (
    teacher_id UUID,
    class_id UUID,
    is_main_teacher BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (teacher_id, class_id)
);

-- 2. 权限系统

-- permissions 表：存储角色权限节点
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role VARCHAR(50) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    value BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(role, permission)
);

-- user_permissions 表：用户特定权限（可选，用于覆盖角色权限）
CREATE TABLE IF NOT EXISTS user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    permission VARCHAR(255) NOT NULL,
    value BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER DEFAULT 100,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, permission)
);

-- 3. 小组系统

-- class_groups 表（小组）
CREATE TABLE IF NOT EXISTS class_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    class_id UUID,
    name VARCHAR(50) NOT NULL,
    description TEXT,
    score INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- group_members 表（小组成员）
CREATE TABLE IF NOT EXISTS group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID,
    person_id UUID,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(group_id, person_id)
);

-- group_score_records 表（小组积分记录）
CREATE TABLE IF NOT EXISTS group_score_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID,
    score_change INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 4. AI 系统

-- ai_settings 表（AI 设置）
CREATE TABLE IF NOT EXISTS ai_settings (
    id SERIAL PRIMARY KEY,
    api_key VARCHAR(512) NOT NULL,
    api_base_url VARCHAR(512) DEFAULT 'https://api.deepseek.com',
    model VARCHAR(100) DEFAULT 'deepseek-chat',
    default_prompt TEXT DEFAULT 'You are an AI assistant for a school management system.',
    temperature DOUBLE PRECISION DEFAULT 0.7,
    max_tokens INTEGER DEFAULT 1000,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 5. 考勤、通知、评分系统

-- attendances 表（考勤记录）
CREATE TABLE IF NOT EXISTS attendances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id UUID,
    date DATE NOT NULL,
    status VARCHAR(20) NOT NULL, -- present, absent, late, early_leave, excused
    time TIME,
    remark TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- notices 表（通知公告）
CREATE TABLE IF NOT EXISTS notices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    author_id UUID,
    target_type VARCHAR(20) NOT NULL, -- school, class, department, group
    target_id UUID, -- 关联的班级/部门/小组ID
    is_important BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- scores 表（评分记录）
CREATE TABLE IF NOT EXISTS scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id UUID,
    group_id UUID,
    score_type VARCHAR(20) NOT NULL, -- personal, group
    value INTEGER NOT NULL,
    reason TEXT NOT NULL,
    event_id UUID,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 添加外键约束

-- departments 自引用外键
ALTER TABLE departments 
ADD CONSTRAINT fk_departments_parent_id 
FOREIGN KEY (parent_id) REFERENCES departments(id) ON DELETE CASCADE;

-- teachers 外键
ALTER TABLE teachers 
ADD CONSTRAINT fk_teachers_person_id 
FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE;

ALTER TABLE teachers 
ADD CONSTRAINT fk_teachers_department_id 
FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE SET NULL;

-- classes 外键
ALTER TABLE classes 
ADD CONSTRAINT fk_classes_teacher_id 
FOREIGN KEY (teacher_id) REFERENCES persons(id) ON DELETE SET NULL;

-- students 外键
ALTER TABLE students 
ADD CONSTRAINT fk_students_person_id 
FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE;

ALTER TABLE students 
ADD CONSTRAINT fk_students_class_id 
FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE SET NULL;

-- parents 外键
ALTER TABLE parents 
ADD CONSTRAINT fk_parents_person_id 
FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE;

-- student_parent 外键
ALTER TABLE student_parent 
ADD CONSTRAINT fk_student_parent_student_id 
FOREIGN KEY (student_id) REFERENCES students(person_id) ON DELETE CASCADE;

ALTER TABLE student_parent 
ADD CONSTRAINT fk_student_parent_parent_id 
FOREIGN KEY (parent_id) REFERENCES parents(person_id) ON DELETE CASCADE;

-- teacher_class 外键
ALTER TABLE teacher_class
ADD CONSTRAINT fk_teacher_class_teacher_id
FOREIGN KEY (teacher_id) REFERENCES teachers(person_id) ON DELETE CASCADE;

ALTER TABLE teacher_class
ADD CONSTRAINT fk_teacher_class_class_id
FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE CASCADE;

-- user_permissions 外键
ALTER TABLE user_permissions 
ADD CONSTRAINT fk_user_permissions_user_id 
FOREIGN KEY (user_id) REFERENCES persons(id) ON DELETE CASCADE;

-- class_groups 外键
ALTER TABLE class_groups 
ADD CONSTRAINT fk_class_groups_class_id 
FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE CASCADE;

-- group_members 外键
ALTER TABLE group_members 
ADD CONSTRAINT fk_group_members_group_id 
FOREIGN KEY (group_id) REFERENCES class_groups(id) ON DELETE CASCADE;

ALTER TABLE group_members 
ADD CONSTRAINT fk_group_members_person_id 
FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE;

-- group_score_records 外键
ALTER TABLE group_score_records 
ADD CONSTRAINT fk_group_score_records_group_id 
FOREIGN KEY (group_id) REFERENCES class_groups(id) ON DELETE CASCADE;

ALTER TABLE group_score_records 
ADD CONSTRAINT fk_group_score_records_created_by 
FOREIGN KEY (created_by) REFERENCES persons(id) ON DELETE SET NULL;

-- attendances 外键
ALTER TABLE attendances 
ADD CONSTRAINT fk_attendances_person_id 
FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE;

ALTER TABLE attendances 
ADD CONSTRAINT fk_attendances_created_by 
FOREIGN KEY (created_by) REFERENCES persons(id) ON DELETE SET NULL;

-- notices 外键
ALTER TABLE notices 
ADD CONSTRAINT fk_notices_author_id 
FOREIGN KEY (author_id) REFERENCES persons(id) ON DELETE SET NULL;

-- scores 外键
ALTER TABLE scores 
ADD CONSTRAINT fk_scores_person_id 
FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE;

ALTER TABLE scores 
ADD CONSTRAINT fk_scores_group_id 
FOREIGN KEY (group_id) REFERENCES class_groups(id) ON DELETE SET NULL;

ALTER TABLE scores 
ADD CONSTRAINT fk_scores_created_by 
FOREIGN KEY (created_by) REFERENCES persons(id) ON DELETE SET NULL;

-- 索引设计

-- persons 表索引
CREATE INDEX IF NOT EXISTS idx_persons_name ON persons(name);
CREATE INDEX IF NOT EXISTS idx_persons_type ON persons(type);
CREATE INDEX IF NOT EXISTS idx_persons_username ON persons(username);
CREATE INDEX IF NOT EXISTS idx_persons_role ON persons(role);

-- departments 表索引
CREATE INDEX IF NOT EXISTS idx_departments_parent_id ON departments(parent_id);

-- classes 表索引
CREATE INDEX IF NOT EXISTS idx_classes_teacher_id ON classes(teacher_id);

-- teachers 表索引
CREATE INDEX IF NOT EXISTS idx_teachers_employee_no ON teachers(employee_no);
CREATE INDEX IF NOT EXISTS idx_teachers_department_id ON teachers(department_id);

-- students 表索引
CREATE INDEX IF NOT EXISTS idx_students_student_no ON students(student_no);
CREATE INDEX IF NOT EXISTS idx_students_class_id ON students(class_id);

-- student_parent 表索引
CREATE INDEX IF NOT EXISTS idx_student_parent_student_id ON student_parent(student_id);
CREATE INDEX IF NOT EXISTS idx_student_parent_parent_id ON student_parent(parent_id);

-- teacher_class 表索引
CREATE INDEX IF NOT EXISTS idx_teacher_class_teacher_id ON teacher_class(teacher_id);
CREATE INDEX IF NOT EXISTS idx_teacher_class_class_id ON teacher_class(class_id);

-- permissions 表索引
CREATE INDEX IF NOT EXISTS idx_permissions_role ON permissions(role);
CREATE INDEX IF NOT EXISTS idx_permissions_role_permission ON permissions(role, permission);
CREATE INDEX IF NOT EXISTS idx_permissions_permission ON permissions(permission);

-- user_permissions 表索引
CREATE INDEX IF NOT EXISTS idx_user_permissions_user_id ON user_permissions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_permissions_user_id_permission ON user_permissions(user_id, permission);

-- class_groups 表索引
CREATE INDEX IF NOT EXISTS idx_class_groups_class_id ON class_groups(class_id);

-- group_members 表索引
CREATE INDEX IF NOT EXISTS idx_group_members_group_id ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_group_members_person_id ON group_members(person_id);

-- group_score_records 表索引
CREATE INDEX IF NOT EXISTS idx_group_score_records_group_id ON group_score_records(group_id);

-- attendances 表索引
CREATE INDEX IF NOT EXISTS idx_attendances_person_id ON attendances(person_id);
CREATE INDEX IF NOT EXISTS idx_attendances_date ON attendances(date);
CREATE INDEX IF NOT EXISTS idx_attendances_status ON attendances(status);

-- notices 表索引
CREATE INDEX IF NOT EXISTS idx_notices_author_id ON notices(author_id);
CREATE INDEX IF NOT EXISTS idx_notices_target_type ON notices(target_type);
CREATE INDEX IF NOT EXISTS idx_notices_target_id ON notices(target_id);

-- scores 表索引
CREATE INDEX IF NOT EXISTS idx_scores_person_id ON scores(person_id);
CREATE INDEX IF NOT EXISTS idx_scores_group_id ON scores(group_id);
CREATE INDEX IF NOT EXISTS idx_scores_score_type ON scores(score_type);

-- 创建触发器

-- persons 表触发器
CREATE TRIGGER update_persons_updated_at BEFORE UPDATE ON persons
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- permissions 表触发器
CREATE TRIGGER update_permissions_updated_at BEFORE UPDATE ON permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- user_permissions 表触发器
CREATE TRIGGER update_user_permissions_updated_at BEFORE UPDATE ON user_permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- class_groups 表触发器
CREATE TRIGGER update_class_groups_updated_at BEFORE UPDATE ON class_groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ai_settings 表触发器
CREATE TRIGGER update_ai_settings_updated_at
    BEFORE UPDATE ON ai_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- attendances 表触发器
CREATE TRIGGER update_attendances_updated_at BEFORE UPDATE ON attendances
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- notices 表触发器
CREATE TRIGGER update_notices_updated_at BEFORE UPDATE ON notices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- scores 表触发器
CREATE TRIGGER update_scores_updated_at BEFORE UPDATE ON scores
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 初始化数据

-- 1. 预留管理员账户
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

-- 2. 插入默认 AI 设置（使用DeepSeek API）
INSERT INTO ai_settings (api_key, api_base_url, model, default_prompt, temperature, max_tokens)
VALUES ('your-api-key-here', 'https://api.deepseek.com', 'deepseek-chat', 'You are an AI assistant for a school management system.', 0.7, 1000)
ON CONFLICT DO NOTHING;

-- 3. 添加默认权限

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
    ('teacher', 'ai.analyze', true, 10)
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
