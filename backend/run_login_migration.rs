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
    
    // 读取迁移文件
    let migration_content = fs::read_to_string("migrations/003_add_login_fields_to_persons.sql")
        .expect("无法读取迁移文件");
    
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
    
    println!("登录字段迁移成功!");
    
    Ok(())
}