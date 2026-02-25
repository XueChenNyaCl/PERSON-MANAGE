-- 启用 UUID 扩展（如果尚未启用）
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 启用 trigram 扩展以支持模糊搜索
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- 创建表（不含外键约束，除了自引用）

-- persons 表（人员基础信息）
CREATE TABLE persons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    gender SMALLINT NOT NULL DEFAULT 0, -- 0:未知 1:男 2:女
    birthday DATE,
    phone VARCHAR(20),
    email VARCHAR(100),
    type VARCHAR(20) NOT NULL, -- 'student', 'teacher', 'parent'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- departments 表（部门）
CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    parent_id UUID, -- 暂不加外键约束
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- teachers 表（教职工扩展信息）
CREATE TABLE teachers (
    person_id UUID PRIMARY KEY,
    employee_no VARCHAR(50) UNIQUE NOT NULL,
    department_id UUID, -- 暂不加外键约束
    class_id UUID, -- 关联班级，暂不加外键约束
    title VARCHAR(50), -- 职称（高级教师、一级教师等）
    hire_date DATE
);

-- classes 表（班级）
CREATE TABLE classes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    grade SMALLINT NOT NULL, -- 年级，如 1 表示一年级
    teacher_id UUID, -- 暂不加外键约束
    academic_year VARCHAR(20), -- 学年，如 "2025-2026"
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- students 表（学生扩展信息）
CREATE TABLE students (
    person_id UUID PRIMARY KEY,
    student_no VARCHAR(50) UNIQUE NOT NULL,
    class_id UUID, -- 暂不加外键约束
    enrollment_date DATE,
    status VARCHAR(20) DEFAULT 'enrolled' -- enrolled, graduated, transferred
);

-- parents 表（家长扩展信息）
CREATE TABLE parents (
    person_id UUID PRIMARY KEY,
    wechat_openid VARCHAR(100), -- 可选，用于推送
    occupation VARCHAR(100)
);

-- student_parent 关联表（学生-家长关系）
CREATE TABLE student_parent (
    student_id UUID,
    parent_id UUID,
    relationship VARCHAR(20), -- father, mother, guardian
    PRIMARY KEY (student_id, parent_id)
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

ALTER TABLE teachers 
ADD CONSTRAINT fk_teachers_class_id 
FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE SET NULL;

-- classes 外键
ALTER TABLE classes 
ADD CONSTRAINT fk_classes_teacher_id 
FOREIGN KEY (teacher_id) REFERENCES teachers(person_id) ON DELETE SET NULL;

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

-- 索引设计
-- 加速搜索
CREATE INDEX idx_persons_name ON persons(name);
CREATE INDEX idx_persons_type ON persons(type);
CREATE INDEX idx_students_student_no ON students(student_no);
CREATE INDEX idx_teachers_employee_no ON teachers(employee_no);
CREATE INDEX idx_students_class_id ON students(class_id);
CREATE INDEX idx_teachers_department_id ON teachers(department_id);
CREATE INDEX idx_teachers_class_id ON teachers(class_id);
CREATE INDEX idx_classes_teacher_id ON classes(teacher_id);
CREATE INDEX idx_departments_parent_id ON departments(parent_id);
CREATE INDEX idx_student_parent_student_id ON student_parent(student_id);
CREATE INDEX idx_student_parent_parent_id ON student_parent(parent_id);

-- 创建 trigram 索引以支持模糊搜索（可选，根据需要启用）
-- CREATE INDEX idx_persons_name_trgm ON persons USING gin (name gin_trgm_ops);

-- 创建 updated_at 自动更新触发器
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_persons_updated_at BEFORE UPDATE ON persons
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();