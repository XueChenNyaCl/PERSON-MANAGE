-- 为已存在的permissions表添加value字段
-- 用于存储权限是允许(true)还是拒绝(false)

-- 检查字段是否存在，如果不存在则添加
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'permissions' 
        AND column_name = 'value'
    ) THEN
        ALTER TABLE permissions ADD COLUMN value BOOLEAN NOT NULL DEFAULT true;
    END IF;
END $$;
