CREATE TABLE IF NOT EXISTS teacher_class (
    teacher_id UUID,
    class_id UUID,
    is_main_teacher BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (teacher_id, class_id)
);

-- 从 teachers 表中移除 class_id 字段（因为现在使用多对多关联）
ALTER TABLE teachers DROP COLUMN IF EXISTS class_id;

-- 添加 teacher_class 表的外键约束
ALTER TABLE teacher_class
ADD CONSTRAINT fk_teacher_class_teacher_id
FOREIGN KEY (teacher_id) REFERENCES teachers(person_id) ON DELETE CASCADE;

ALTER TABLE teacher_class
ADD CONSTRAINT fk_teacher_class_class_id
FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE CASCADE;

-- 删除旧的 teachers_class_id 索引
DROP INDEX IF EXISTS idx_teachers_class_id;

-- 添加新的索引
CREATE INDEX idx_teacher_class_teacher_id ON teacher_class(teacher_id);
CREATE INDEX idx_teacher_class_class_id ON teacher_class(class_id);

-- 保留 classes 表中的 teacher_id 字段，用于标识班主任
-- 但需要更新外键约束，使其引用 persons 表而不是 teachers 表
ALTER TABLE classes DROP CONSTRAINT IF EXISTS fk_classes_teacher_id;

ALTER TABLE classes
ADD CONSTRAINT fk_classes_teacher_id
FOREIGN KEY (teacher_id) REFERENCES persons(id) ON DELETE SET NULL;