use sqlx::PgPool;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 读取 .env 文件
    dotenv::from_filename(".env").ok();
    
    // 获取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL 未设置");
    
    // 连接数据库
    let pool = PgPool::connect(&database_url).await?;
    
    // 读取权限迁移文件
    let migration_content = fs::read_to_string("migrations/004_create_permission_tables.sql")
        .expect("无法读取权限迁移文件");
    
    // 拆分成单独的 SQL 命令并执行
    let mut commands = Vec::new();
    let mut current_command = String::new();
    
    for line in migration_content.lines() {
        let trimmed_line = line.trim();
        
        // 跳过注释行
        if trimmed_line.starts_with("--") || trimmed_line.is_empty() {
            continue;
        }
        
        // 添加当前行到命令
        current_command.push_str(line);
        current_command.push(' ');
        
        // 如果行以分号结尾，说明这是一个完整的命令
        if trimmed_line.ends_with(';') {
            commands.push(current_command.trim().to_string());
            current_command.clear();
        }
    }
    
    // 执行所有命令
    for command in &commands {
        println!("执行 SQL: {}", command);
        sqlx::query(command).execute(&pool).await?;
    }
    
    println!("权限数据库迁移成功!");
    
    // 根据用户要求：默认没有任何权限，需要手动添加
    // 但为了测试，我们初始化一些基本权限
    println!("初始化基本权限用于测试...");
    sqlx::query("DELETE FROM permissions").execute(&pool).await?;
    
    // 插入基本权限配置
    init_default_permissions(&pool).await?;
    
    println!("权限初始化完成!");
    
    Ok(())
}

async fn init_default_permissions(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 管理员权限
    let admin_permissions = vec![
        "dashboard.view",
        "person.view",
        "person.manage",
        "person.sensitive.view",
        "class.view",
        "class.manage",
        "class.update_teacher",
        "department.view",
        "department.manage",
        "department.update",
        "attendance.view",
        "attendance.manage",
        "score.view",
        "score.manage",
        "notice.view",
        "notice.manage",
        "system.settings",
    ];
    
    // 教师权限
    let teacher_permissions = vec![
        "dashboard.view",
        "person.view",
        "person.sensitive.view",
        "class.view",
        "class.manage",
        "class.update_teacher",
        "attendance.manage",
        "score.manage",
        "notice.view",
        "department.view",
    ];
    
    // 学生权限
    let student_permissions = vec![
        "dashboard.view",
        "person.view",
        "class.view",
        "attendance.view",
        "score.view",
        "notice.view",
    ];
    
    // 家长权限
    let parent_permissions = vec![
        "dashboard.view",
        "person.view",
        "class.view",
        "attendance.view",
        "score.view",
        "notice.view",
    ];
    
    // 插入管理员权限
    for perm in admin_permissions {
        sqlx::query(
            "INSERT INTO permissions (role, permission, priority) VALUES ($1, $2, $3) 
             ON CONFLICT (role, permission) DO NOTHING"
        )
        .bind("admin")
        .bind(perm)
        .bind(10)
        .execute(pool)
        .await?;
    }
    
    // 插入教师权限
    for perm in teacher_permissions {
        sqlx::query(
            "INSERT INTO permissions (role, permission, priority) VALUES ($1, $2, $3) 
             ON CONFLICT (role, permission) DO NOTHING"
        )
        .bind("teacher")
        .bind(perm)
        .bind(5)
        .execute(pool)
        .await?;
    }
    
    // 插入学生权限
    for perm in student_permissions {
        sqlx::query(
            "INSERT INTO permissions (role, permission, priority) VALUES ($1, $2, $3) 
             ON CONFLICT (role, permission) DO NOTHING"
        )
        .bind("student")
        .bind(perm)
        .bind(0)
        .execute(pool)
        .await?;
    }
    
    // 插入家长权限
    for perm in parent_permissions {
        sqlx::query(
            "INSERT INTO permissions (role, permission, priority) VALUES ($1, $2, $3) 
             ON CONFLICT (role, permission) DO NOTHING"
        )
        .bind("parent")
        .bind(perm)
        .bind(0)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}