use bcrypt::{hash, DEFAULT_COST};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("初始化用户登录凭据...");
    println!("数据库连接: {}", database_url);
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // 检查persons表是否有登录字段
    println!("检查persons表结构...");
    let has_username_column = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'persons' AND column_name = 'username'
        )"
    )
    .fetch_one(&pool)
    .await?;
    
    if !has_username_column {
        println!("❌ persons表缺少username字段，请先运行迁移文件003_add_login_fields_to_persons.sql");
        return Ok(());
    }
    
    // 生成默认密码哈希
    let default_password = "123456";
    let password_hash = hash(default_password, DEFAULT_COST)?;
    println!("默认密码哈希生成完成: {}", default_password);
    
    // 处理学生
    println!("处理学生账号...");
    let students = sqlx::query!(
        r#"
        SELECT s.person_id, s.student_no, p.name 
        FROM students s
        JOIN persons p ON s.person_id = p.id
        WHERE p.username IS NULL OR p.password_hash IS NULL
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    println!("找到 {} 个需要初始化的学生", students.len());
    
    for student in students {
        println!("  学生: {} (学号: {})", student.name, student.student_no);
        
        match sqlx::query!(
            r#"
            UPDATE persons 
            SET username = $1, password_hash = $2, role = 'student', is_active = true
            WHERE id = $3 AND (username IS NULL OR password_hash IS NULL)
            "#,
            student.student_no,
            password_hash,
            student.person_id
        )
        .execute(&pool)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    println!("    ✅ 登录凭据设置成功");
                } else {
                    println!("    ⚠️  登录凭据已存在或更新失败");
                }
            }
            Err(e) => {
                println!("    ❌ 更新失败: {}", e);
            }
        }
    }
    
    // 处理教师
    println!("处理教师账号...");
    let teachers = sqlx::query!(
        r#"
        SELECT t.person_id, t.employee_no, p.name 
        FROM teachers t
        JOIN persons p ON t.person_id = p.id
        WHERE p.username IS NULL OR p.password_hash IS NULL
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    println!("找到 {} 个需要初始化的教师", teachers.len());
    
    for teacher in teachers {
        println!("  教师: {} (工号: {})", teacher.name, teacher.employee_no);
        
        match sqlx::query!(
            r#"
            UPDATE persons 
            SET username = $1, password_hash = $2, role = 'teacher', is_active = true
            WHERE id = $3 AND (username IS NULL OR password_hash IS NULL)
            "#,
            teacher.employee_no,
            password_hash,
            teacher.person_id
        )
        .execute(&pool)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    println!("    ✅ 登录凭据设置成功");
                } else {
                    println!("    ⚠️  登录凭据已存在或更新失败");
                }
            }
            Err(e) => {
                println!("    ❌ 更新失败: {}", e);
            }
        }
    }
    
    // 处理家长（如果没有特殊标识，使用person_id作为用户名）
    println!("处理家长账号...");
    let parents = sqlx::query!(
        r#"
        SELECT p.id, p.name
        FROM persons p
        WHERE p.type = 'parent' 
          AND (p.username IS NULL OR p.password_hash IS NULL)
          AND NOT EXISTS (
            SELECT 1 FROM teachers t WHERE t.person_id = p.id
          )
          AND NOT EXISTS (
            SELECT 1 FROM students s WHERE s.person_id = p.id
          )
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    println!("找到 {} 个需要初始化的家长", parents.len());
    
    for parent in parents {
        // 生成用户名：parent_<id前8位>
        let username = format!("parent_{}", &parent.id.to_string()[..8]);
        println!("  家长: {} (用户名: {})", parent.name, username);
        
        match sqlx::query!(
            r#"
            UPDATE persons 
            SET username = $1, password_hash = $2, role = 'parent', is_active = true
            WHERE id = $3 AND (username IS NULL OR password_hash IS NULL)
            "#,
            username,
            password_hash,
            parent.id
        )
        .execute(&pool)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    println!("    ✅ 登录凭据设置成功");
                } else {
                    println!("    ⚠️  登录凭据已存在或更新失败");
                }
            }
            Err(e) => {
                println!("    ❌ 更新失败: {}", e);
            }
        }
    }
    
    // 统计结果
    println!("\n初始化完成！统计结果：");
    
    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE username IS NOT NULL AND password_hash IS NOT NULL")
        .fetch_one(&pool)
        .await?;
    println!("  已设置登录凭据的用户总数: {}", total_users);
    
    let students_with_creds = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE role = 'student' AND username IS NOT NULL")
        .fetch_one(&pool)
        .await?;
    println!("  学生账号: {}", students_with_creds);
    
    let teachers_with_creds = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE role = 'teacher' AND username IS NOT NULL")
        .fetch_one(&pool)
        .await?;
    println!("  教师账号: {}", teachers_with_creds);
    
    let parents_with_creds = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE role = 'parent' AND username IS NOT NULL")
        .fetch_one(&pool)
        .await?;
    println!("  家长账号: {}", parents_with_creds);
    
    let admins_with_creds = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE role = 'admin' AND username IS NOT NULL")
        .fetch_one(&pool)
        .await?;
    println!("  管理员账号: {}", admins_with_creds);
    
    println!("\n默认登录信息：");
    println!("  用户名: 学号(学生)/工号(教师)/parent_xxxx(家长)");
    println!("  密码: {}", default_password);
    println!("  注意：首次登录后建议修改密码");
    
    Ok(())
}