use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取 .env 文件
    dotenv::from_filename(".env").ok();

    // 获取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL 未设置");

    // 连接数据库
    println!("连接到数据库...");
    let pool = PgPool::connect(&database_url).await?;

    // 执行特殊用户迁移
    println!("执行特殊用户迁移...");

    // 1. 创建特殊用户表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS special_users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_type VARCHAR(20) NOT NULL,
            identifier VARCHAR(100) NOT NULL,
            linked_person_id UUID REFERENCES persons(id) ON DELETE SET NULL,
            api_key_hash VARCHAR(255),
            description TEXT,
            is_active BOOLEAN DEFAULT true,
            last_login_at TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            UNIQUE(user_type, identifier)
        )
        "#,
    )
    .execute(&pool)
    .await?;
    println!("✓ 创建 special_users 表");

    // 2. 创建操作日志表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operation_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            operator_id UUID REFERENCES persons(id) ON DELETE SET NULL,
            operator_type VARCHAR(20) NOT NULL,
            operator_name VARCHAR(100) NOT NULL,
            action VARCHAR(100) NOT NULL,
            resource_type VARCHAR(50),
            resource_id UUID,
            details JSONB,
            ip_address VARCHAR(50),
            user_agent TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;
    println!("✓ 创建 operation_logs 表");

    // 3. 添加索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_special_users_type ON special_users(user_type)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_special_users_identifier ON special_users(identifier)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON operation_logs(created_at)")
        .execute(&pool)
        .await?;
    println!("✓ 创建索引");

    // 4. 添加 persons 表字段
    sqlx::query("ALTER TABLE persons ADD COLUMN IF NOT EXISTS is_system_user BOOLEAN DEFAULT false")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE persons ADD COLUMN IF NOT EXISTS system_user_type VARCHAR(20)")
        .execute(&pool)
        .await?;
    println!("✓ 添加 persons 表字段");

    // 5. 初始化特殊用户
    sqlx::query(
        r#"
        INSERT INTO special_users (user_type, identifier, description, is_active)
        VALUES ('system', 'system', '系统内部操作用户，用于记录程序自动执行的操作', true)
        ON CONFLICT (user_type, identifier) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await?;
    println!("✓ 初始化 system 用户");

    sqlx::query(
        r#"
        INSERT INTO special_users (user_type, identifier, description, is_active)
        VALUES ('sysai', 'SysAI', '系统AI用户，暂留功能', true)
        ON CONFLICT (user_type, identifier) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await?;
    println!("✓ 初始化 SysAI 用户");

    sqlx::query(
        r#"
        INSERT INTO special_users (user_type, identifier, description, is_active)
        VALUES ('chatai', 'ChatAI', '聊天AI操作记录用户，用于记录用户让AI执行的操作', true)
        ON CONFLICT (user_type, identifier) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await?;
    println!("✓ 初始化 ChatAI 用户");

    // 6. 添加特殊用户权限
    let permissions = vec![
        ("admin", "special_user.view", true, 10),
        ("admin", "special_user.create", true, 10),
        ("admin", "special_user.delete", true, 10),
        ("admin", "special_user.link", true, 10),
        ("admin", "operation_log.view", true, 10),
    ];

    for (role, permission, value, priority) in permissions {
        sqlx::query(
            r#"
            INSERT INTO permissions (role, permission, value, priority)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (role, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority
            "#,
        )
        .bind(role)
        .bind(permission)
        .bind(value)
        .bind(priority)
        .execute(&pool)
        .await?;
        println!("✓ 添加权限: {}.{} (priority: {})", role, permission, priority);
    }

    // 7. 更新预留管理员账户
    sqlx::query(
        r#"
        UPDATE persons 
        SET is_system_user = true, 
            system_user_type = 'admin',
            role = 'admin'
        WHERE id = '00000000-0000-0000-0000-000000000000'
        "#,
    )
    .execute(&pool)
    .await?;
    println!("✓ 更新预留管理员账户标记");

    println!("\n特殊用户迁移完成!");
    Ok(())
}
