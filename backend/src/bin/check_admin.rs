use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use sqlx::types::Uuid;
use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("Checking admin user in database: {}", database_url);
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // 检查admin用户是否存在
    match sqlx::query("SELECT id, username, name, role, is_active FROM persons WHERE username = 'admin'")
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(row)) => {
            let id: Uuid = row.get(0);
            let username: String = row.get(1);
            let name: String = row.get(2);
            let role: String = row.get(3);
            let is_active: bool = row.get(4);
            println!("✅ Admin user found:");
            println!("   ID: {}", id);
            println!("   Username: {}", username);
            println!("   Name: {}", name);
            println!("   Role: {}", role);
            println!("   Is active: {}", is_active);
            
            // 检查密码哈希
            match sqlx::query("SELECT password_hash FROM persons WHERE username = 'admin'")
                .fetch_optional(&pool)
                .await
            {
                Ok(Some(row)) => {
                    let password_hash: Option<String> = row.get(0);
                    match password_hash {
                        Some(hash) => {
                            println!("   Password hash: {}", hash);
                            // 验证哈希是否匹配 'admin'
                            let password = "admin";
                            // 注意：这里只是显示，实际验证需要bcrypt库
                            println!("   Password to verify: '{}'", password);
                            println!("   Note: Use bcrypt::verify(password, &hash) to verify");
                        }
                        None => println!("   ❌ No password hash set for admin"),
                    }
                }
                Ok(None) => println!("   ❌ Could not retrieve password hash"),
                Err(e) => println!("   ❌ Error fetching password hash: {}", e),
            }
        }
        Ok(None) => {
            println!("❌ Admin user not found in persons table");
            // 列出所有用户
            match sqlx::query("SELECT username, name, role FROM persons WHERE username IS NOT NULL LIMIT 10")
                .fetch_all(&pool)
                .await
            {
                Ok(rows) => {
                    println!("   Existing users with username:");
                    for row in rows {
                        let username: Option<String> = row.get(0);
                        let name: String = row.get(1);
                        let role: Option<String> = row.get(2);
                        println!("      - Username: {:?}, Name: {}, Role: {:?}", username, name, role);
                    }
                }
                Err(e) => println!("   ❌ Error listing users: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Error querying admin user: {}", e);
            println!("   Possible: persons table doesn't have username column yet");
        }
    }
    
    // 检查persons表结构
    println!("\nChecking persons table structure:");
    match sqlx::query("SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = 'persons' ORDER BY ordinal_position")
        .fetch_all(&pool)
        .await
    {
        Ok(rows) => {
            println!("   Columns in persons table:");
            for row in rows {
                let column_name: String = row.get(0);
                let data_type: String = row.get(1);
                let is_nullable: String = row.get(2);
                println!("      - {}: {} (nullable: {})", column_name, data_type, is_nullable);
            }
        }
        Err(e) => println!("   ❌ Error checking table structure: {}", e),
    }
    
    // 查询所有人员
    println!("\nChecking all persons in database:");
    match sqlx::query("SELECT id, name, type, username, role FROM persons ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
    {
        Ok(rows) => {
            println!("   Total persons count: {}", rows.len());
            for row in rows {
                let id: Uuid = row.get(0);
                let name: String = row.get(1);
                let person_type: String = row.get(2);
                let username: Option<String> = row.get(3);
                let role: Option<String> = row.get(4);
                println!("      - ID: {}, Name: {}, Type: {}, Username: {:?}, Role: {:?}", id, name, person_type, username, role);
            }
        }
        Err(e) => println!("   ❌ Error querying persons: {}", e),
    }
    
    // 检查students表
    println!("\nChecking students table:");
    match sqlx::query("SELECT COUNT(*) FROM students")
        .fetch_one(&pool)
        .await
    {
        Ok(row) => {
            let count: i64 = row.get(0);
            println!("   Total students count: {}", count);
        }
        Err(e) => println!("   ❌ Error checking students table: {}", e),
    }
    
    // 检查teachers表
    println!("\nChecking teachers table:");
    match sqlx::query("SELECT COUNT(*) FROM teachers")
        .fetch_one(&pool)
        .await
    {
        Ok(row) => {
            let count: i64 = row.get(0);
            println!("   Total teachers count: {}", count);
        }
        Err(e) => println!("   ❌ Error checking teachers table: {}", e),
    }
    
    // 检查parents表
    println!("\nChecking parents table:");
    match sqlx::query("SELECT COUNT(*) FROM parents")
        .fetch_one(&pool)
        .await
    {
        Ok(row) => {
            let count: i64 = row.get(0);
            println!("   Total parents count: {}", count);
        }
        Err(e) => println!("   ❌ Error checking parents table: {}", e),
    }
    
    Ok(())
}