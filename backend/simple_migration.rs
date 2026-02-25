use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 读取 .env 文件
    dotenv::from_filename(".env").ok();
    
    // 获取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL 未设置");
    
    // 连接数据库
    let pool = PgPool::connect(&database_url).await?;
    
    // 1. 创建 teacher_class 表
    println!("创建 teacher_class 表...");
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS teacher_class (
            teacher_id UUID,
            class_id UUID,
            is_main_teacher BOOLEAN DEFAULT FALSE,
            PRIMARY KEY (teacher_id, class_id)
        )
    "#).execute(&pool).await?;
    
    // 2. 从 teachers 表中移除 class_id 字段
    println!("从 teachers 表中移除 class_id 字段...");
    sqlx::query("ALTER TABLE teachers DROP COLUMN IF EXISTS class_id").execute(&pool).await?;
    
    // 3. 添加 teacher_class 表的外键约束
    println!("添加 teacher_class 表的外键约束...");
    sqlx::query(r#"
        ALTER TABLE teacher_class
        ADD CONSTRAINT fk_teacher_class_teacher_id
        FOREIGN KEY (teacher_id) REFERENCES teachers(person_id) ON DELETE CASCADE
    "#).execute(&pool).await?;
    
    sqlx::query(r#"
        ALTER TABLE teacher_class
        ADD CONSTRAINT fk_teacher_class_class_id
        FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE CASCADE
    "#).execute(&pool).await?;
    
    // 4. 删除旧的索引
    println!("删除旧的索引...");
    sqlx::query("DROP INDEX IF EXISTS idx_teachers_class_id").execute(&pool).await?;
    
    // 5. 添加新的索引
    println!("添加新的索引...");
    sqlx::query("CREATE INDEX idx_teacher_class_teacher_id ON teacher_class(teacher_id)").execute(&pool).await?;
    sqlx::query("CREATE INDEX idx_teacher_class_class_id ON teacher_class(class_id)").execute(&pool).await?;
    
    // 6. 更新 classes 表的外键约束
    println!("更新 classes 表的外键约束...");
    sqlx::query("ALTER TABLE classes DROP CONSTRAINT IF EXISTS fk_classes_teacher_id").execute(&pool).await?;
    
    sqlx::query(r#"
        ALTER TABLE classes
        ADD CONSTRAINT fk_classes_teacher_id
        FOREIGN KEY (teacher_id) REFERENCES persons(id) ON DELETE SET NULL
    "#).execute(&pool).await?;
    
    println!("数据库迁移成功!");
    
    Ok(())
}