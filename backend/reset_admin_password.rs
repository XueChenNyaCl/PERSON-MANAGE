use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取 .env 文件
    dotenv::from_filename(".env").ok();

    // 获取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL 未设置");

    // 连接数据库
    println!("连接到数据库...");
    let pool = PgPool::connect(&database_url).await?;

    // 生成密码哈希
    let password = "admin";
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    
    println!("生成的密码哈希: {}", password_hash);

    // 更新 admin 用户密码
    let result = sqlx::query(
        "UPDATE persons SET password_hash = $1 WHERE username = 'admin' AND id = '00000000-0000-0000-0000-000000000000'"
    )
    .bind(&password_hash)
    .execute(&pool)
    .await?;

    if result.rows_affected() > 0 {
        println!("✓ Admin 密码已重置为 'admin'");
    } else {
        println!("⚠ 未找到 admin 用户");
    }

    // 验证密码
    let stored_hash: String = sqlx::query_scalar(
        "SELECT password_hash FROM persons WHERE username = 'admin'"
    )
    .fetch_one(&pool)
    .await?;

    let valid = bcrypt::verify("admin", &stored_hash)?;
    println!("密码验证结果: {}", valid);

    Ok(())
}
