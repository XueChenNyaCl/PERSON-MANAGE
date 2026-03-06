use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// 权限模板项
#[derive(Debug, Deserialize, Serialize)]
struct PermissionTemplateItem {
    permission: String,
    priority: i32,
}

/// 权限模板
#[derive(Debug, Deserialize, Serialize)]
struct PermissionTemplate {
    permissions: Vec<PermissionTemplateItem>,
}

impl PermissionTemplate {
    /// 从YAML文件加载权限模板
    fn from_yaml_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let yaml_content = fs::read_to_string(file_path)?;
        Self::from_yaml_str(&yaml_content)
    }
    
    /// 从YAML字符串加载权限模板
    fn from_yaml_str(yaml_content: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 预处理YAML内容，处理通配符权限
        // YAML将*解释为别名，需要处理这种情况
        let processed_content = yaml_content
            .lines()
            .map(|line| {
                if line.trim().starts_with("- permission:") {
                    // 提取permission值
                    if let Some((_, value)) = line.split_once(":") {
                        let value = value.trim();
                        // 如果值以*开头且不是用引号包围的，添加引号
                        if value.starts_with('*') && !value.starts_with('"') && !value.starts_with('\'') {
                            return line.replace(&format!(": {}", value), &format!(": \"{}\"", value));
                        }
                    }
                }
                line.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let template: PermissionTemplate = serde_yaml::from_str(&processed_content)?;
        Ok(template)
    }
}

/// 拆分SQL命令，处理分号和注释
fn split_sql_commands(content: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut current_command = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_quote_tag = String::new();
    
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        
        // 跳过纯注释行和空行
        if (trimmed.starts_with("--") && !trimmed.contains(";")) || trimmed.is_empty() {
            i += 1;
            continue;
        }
        
        current_command.push_str(line);
        current_command.push('\n');
        
        // 检查是否是 $tag$ 开始
        if !in_dollar_quote {
            if let Some(start) = line.find("$") {
                if let Some(end) = line[start+1..].find("$") {
                    let tag = &line[start+1..start+1+end];
                    if !tag.is_empty() && tag.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        in_dollar_quote = true;
                        dollar_quote_tag = format!("${}$", tag);
                    }
                }
            }
        } else if line.contains(&dollar_quote_tag) {
            // 找到结束标记
            in_dollar_quote = false;
            dollar_quote_tag.clear();
        }
        
        // 如果不在 dollar quote 中，且行以分号结尾，则完成一个命令
        if !in_dollar_quote && trimmed.ends_with(';') && !trimmed.starts_with("--") {
            let cmd = current_command.trim().to_string();
            if !cmd.is_empty() {
                commands.push(cmd);
            }
            current_command.clear();
        }
        
        i += 1;
    }
    
    // 处理最后可能剩余的命令
    if !current_command.trim().is_empty() {
        commands.push(current_command.trim().to_string());
    }
    
    commands
}

/// 从YAML模板文件加载权限并初始化到数据库
async fn init_permissions_from_templates(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 定义角色和对应的模板文件
    let roles = vec!["admin", "teacher", "student", "parent"];
    
    for role in roles {
        let template_path = format!("templates/permissions/{}.yaml", role);
        
        match PermissionTemplate::from_yaml_file(&template_path) {
            Ok(template) => {
                println!("加载 {} 权限模板: {} 个权限", role, template.permissions.len());
                
                // 清空该角色的现有权限
                sqlx::query("DELETE FROM permissions WHERE role = $1")
                    .bind(role)
                    .execute(pool)
                    .await?;
                
                // 插入模板中的权限
                for item in &template.permissions {
                    // 处理否定权限（以-开头）
                    let (permission_str, value) = if item.permission.starts_with('-') {
                        (&item.permission[1..], false)
                    } else {
                        (item.permission.as_str(), true)
                    };
                    
                    sqlx::query(
                        "INSERT INTO permissions (role, permission, value, priority) VALUES ($1, $2, $3, $4) 
                         ON CONFLICT (role, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority"
                    )
                    .bind(role)
                    .bind(permission_str)
                    .bind(value)
                    .bind(item.priority)
                    .execute(pool)
                    .await?;
                    
                    let action = if value { "允许" } else { "拒绝" };
                    println!("  [{}] {} - 优先级: {}", action, permission_str, item.priority);
                }
                
                println!("{} 权限初始化完成!\n", role);
            }
            Err(e) => {
                eprintln!("警告: 无法加载 {} 权限模板: {}", role, e);
            }
        }
    }
    
    Ok(())
}

/// 运行基础迁移
async fn run_basic_migration(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 读取迁移文件
    let migration_content = fs::read_to_string("migrations/001_initial_schema.sql").expect("无法读取迁移文件");
    
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
        sqlx::query(command).execute(pool).await?;
    }
    
    println!("数据库迁移成功!");
    
    Ok(())
}

/// 运行权限迁移
async fn run_permission_migration(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 读取权限迁移文件
    let migration_content = fs::read_to_string("migrations/001_initial_schema.sql")
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
        match sqlx::query(command).execute(pool).await {
            Ok(_) => {},
            Err(e) => {
                // 如果表已存在，忽略错误
                println!("警告: SQL执行可能失败: {}", e);
            }
        }
    }
    
    // 执行额外的迁移：添加value字段
    println!("检查并添加value字段...");
    let alter_table_sql = r#"
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
    "#;
    match sqlx::query(alter_table_sql).execute(pool).await {
        Ok(_) => println!("value字段检查完成"),
        Err(e) => println!("警告: 添加value字段可能失败: {}", e),
    }
    
    println!("权限数据库迁移成功!");
    
    // 从YAML模板文件加载权限
    println!("从YAML模板文件初始化权限...");
    init_permissions_from_templates(pool).await?;
    
    println!("权限初始化完成!");
    
    Ok(())
}

/// 运行AI权限迁移
async fn run_ai_permission_migration(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Applying AI permissions migration...");

    // 手动执行权限插入，使用 ON CONFLICT 处理已存在的情况
    // value 列是布尔类型，使用 true/false 而不是 "true"/"false"
    let permissions = vec![
        // 管理员权限
        ("admin", "ai.view", true, 10),
        ("admin", "ai.chat", true, 10),
        ("admin", "ai.analyze", true, 10),
        ("admin", "ai.settings", true, 10),
        ("admin", "ai.*", true, 5),
        // 教师权限
        ("teacher", "ai.view", true, 10),
        ("teacher", "ai.chat", true, 10),
        ("teacher", "ai.analyze", true, 10),
        // 学生权限
        ("student", "ai.view", true, 7),
        ("student", "ai.chat", true, 7),
        ("student", "ai.analyze", true, 7),
        // 家长权限
        ("parent", "ai.view", true, 10),
        ("parent", "ai.chat", true, 10),
        ("parent", "ai.analyze", true, 10),
    ];

    for (role, permission, value, priority) in permissions {
        match sqlx::query(
            "INSERT INTO permissions (role, permission, value, priority) 
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (role, permission) 
             DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority"
        )
        .bind(role)
        .bind(permission)
        .bind(value)
        .bind(priority)
        .execute(pool)
        .await
        {
            Ok(_) => println!("✓ Added/Updated permission: {}.{} (priority: {})", role, permission, priority),
            Err(e) => println!("✗ Failed to add permission {}.{}: {}", role, permission, e),
        }
    }

    println!("\nAI permissions migration completed!");
    
    // 验证权限是否添加成功
    println!("\nVerifying permissions...");
    let rows = sqlx::query_as::<_, (String, String, bool, i32)>(
        "SELECT role, permission, value, priority FROM permissions WHERE permission LIKE 'ai.%' ORDER BY role, permission"
    )
    .fetch_all(pool)
    .await?;

    println!("\nCurrent AI permissions in database:");
    println!("{:<10} {:<15} {:<6} {:<10}", "Role", "Permission", "Value", "Priority");
    println!("{}", "-".repeat(50));
    for (role, permission, value, priority) in rows {
        println!("{:<10} {:<15} {:<6} {:<10}", role, permission, value, priority);
    }

    Ok(())
}

/// 检查教师权限
async fn check_teacher_permissions(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 查询所有班级和班主任
    println!("\n=== 班级和班主任信息 ===");
    let classes = sqlx::query!("SELECT c.id, c.name, c.teacher_id, p.name as teacher_name FROM classes c LEFT JOIN persons p ON c.teacher_id = p.id")
        .fetch_all(pool)
        .await?;
    
    for class in classes {
        println!("班级: {} (ID: {})", class.name, class.id);
        if let Some(teacher_id) = class.teacher_id {
            println!("  班主任ID: {}", teacher_id);
            println!("  班主任姓名: {:?}", class.teacher_name);
            
            // 获取班级ID后6位
            let id_str = class.id.to_string().replace("-", "");
            let class_suffix: String = id_str.chars().rev().take(6).collect::<String>().chars().rev().collect();
            println!("  班级后缀: {}", class_suffix);
            
            // 查询该老师的权限
            let permissions = sqlx::query!("SELECT permission, value, priority FROM user_permissions WHERE user_id = $1", teacher_id)
                .fetch_all(pool)
                .await?;
            
            if permissions.is_empty() {
                println!("  该老师没有特定权限");
            } else {
                println!("  该老师的权限:");
                for perm in permissions {
                    println!("    - {} (value: {:?}, priority: {:?})", perm.permission, perm.value, perm.priority);
                }
            }
        } else {
            println!("  暂无班主任");
        }
        println!();
    }
    
    // 查询所有教师用户
    println!("\n=== 所有教师用户 ===");
    let teachers = sqlx::query!("SELECT id, username, name, role FROM persons WHERE role = 'teacher' OR type = 'teacher'")
        .fetch_all(pool)
        .await?;
    
    for teacher in teachers {
        println!("教师: {:?} (ID: {}, 用户名: {:?}, 角色: {:?})", teacher.name, teacher.id, teacher.username, teacher.role);
    }
    
    Ok(())
}

/// 修复教师权限
async fn fix_teacher_permissions(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 查询所有有班主任但没有班级特定权限的班级
    println!("\n=== 检查并修复班主任权限 ===\n");
    
    let classes = sqlx::query!(
        "SELECT c.id, c.name, c.teacher_id, p.name as teacher_name 
         FROM classes c 
         LEFT JOIN persons p ON c.teacher_id = p.id
         WHERE c.teacher_id IS NOT NULL"
    )
        .fetch_all(pool)
        .await?;
    
    for class in classes {
        let teacher_id = class.teacher_id.unwrap();
        let class_id = class.id.unwrap();
        let class_name = class.name.unwrap_or_else(|| "未知班级".to_string());
        
        println!("班级: {} (ID: {})", class_name, class_id);
        println!("  班主任: {:?} (ID: {})", class.teacher_name, teacher_id);
        
        // 获取班级ID后6位
        let id_str = class_id.to_string().replace("-", "");
        let class_suffix: String = id_str.chars().rev().take(6).collect::<String>().chars().rev().collect();
        println!("  班级后缀: {}", class_suffix);
        
        // 检查是否已经有班级特定权限
        let has_class_perm = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_permissions WHERE user_id = $1 AND permission LIKE $2"
        )
        .bind(teacher_id)
        .bind(format!("%.{}", class_suffix))
        .fetch_one(pool)
        .await?;
        
        if has_class_perm > 0 {
            println!("  ✓ 已有班级特定权限，跳过");
        } else {
            println!("  ✗ 没有班级特定权限，正在植入...");
            
            // 植入班级特定权限
            let permissions = vec![
                format!("class.{}", class_suffix), // 班级通用管理权限
                format!("group.view.{}", class_suffix),
                format!("group.create.{}", class_suffix),
                format!("group.update.{}", class_suffix),
                format!("group.delete.{}", class_suffix),
                format!("group.update.member.{}", class_suffix),
                format!("group.update.score.{}", class_suffix),
            ];
            
            for permission in permissions {
                sqlx::query(
                    "INSERT INTO user_permissions (user_id, permission, value, priority) 
                     VALUES ($1, $2, $3, $4) 
                     ON CONFLICT (user_id, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority"
                )
                .bind(teacher_id)
                .bind(&permission)
                .bind(true)
                .bind(20)
                .execute(pool)
                .await?;
                
                println!("    已植入: {}", permission);
            }
        }
        println!();
    }
    
    println!("权限修复完成！");
    
    Ok(())
}

/// 运行所有迁移
async fn run_all_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 确保 _sqlx_migrations 表存在
    println!("检查迁移表...");
    if let Err(e) = sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            success BOOLEAN NOT NULL,
            checksum BYTEA NOT NULL,
            execution_time BIGINT NOT NULL
        )
    "#).execute(pool).await {
        println!("警告: 创建迁移表失败 (可能已存在): {}", e);
    }
    
    // 定义所有迁移文件（按顺序）
    let migrations = vec![
        (1, "001_initial_schema.sql", "initial schema with all tables and data"),
    ];
    
    for (version, filename, description) in migrations {
        // 检查迁移是否已经执行
        let already_executed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version = $1 AND success = true)"
        )
        .bind(version as i64)
        .fetch_one(pool)
        .await?;
        
        if already_executed {
            println!("迁移 {} ({}) 已执行，跳过", version, description);
            continue;
        }
        
        let filepath = format!("migrations/{}", filename);
        let path = Path::new(&filepath);
        
        if !path.exists() {
            println!("警告: 迁移文件 {} 不存在，跳过", filepath);
            continue;
        }
        
        println!("执行迁移 {}: {}", version, description);
        
        // 读取迁移文件
        let migration_content = fs::read_to_string(&filepath)?;
        
        // 拆分SQL命令
        let commands = split_sql_commands(&migration_content);
        
        // 逐条执行SQL命令（不使用事务，因为某些命令可能失败但不影响其他）
        let mut all_success = true;
        for (i, cmd) in commands.iter().enumerate() {
            if cmd.trim().is_empty() {
                continue;
            }
            
            let cmd_preview: String = cmd.chars().take(60).collect();
            print!("  执行 SQL {}/{}: {}... ", i + 1, commands.len(), cmd_preview);
            
            match sqlx::query(cmd).execute(pool).await {
                Ok(_) => println!("OK"),
                Err(e) => {
                    let err_msg = e.to_string();
                    // 检查是否是"已存在"类型的错误
                    if err_msg.contains("already exists") || 
                       err_msg.contains("42710") ||  // duplicate object
                       err_msg.contains("42P07") ||  // duplicate table
                       err_msg.contains("42P16") {   // duplicate column
                        println!("已存在，跳过");
                    } else {
                        println!("错误: {}", e);
                        all_success = false;
                        // 继续执行其他命令
                    }
                }
            }
        }
        
        // 记录迁移
        if all_success {
            if let Err(e) = sqlx::query(r#"
                INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
                VALUES ($1, $2, true, '\x00'::bytea, 0)
                ON CONFLICT (version) DO UPDATE SET 
                    success = true,
                    installed_on = NOW()
            "#)
            .bind(version as i64)
            .bind(description)
            .execute(pool).await {
                println!("警告: 记录迁移失败: {}", e);
            }
            println!("迁移 {} 执行成功!", version);
        } else {
            println!("迁移 {} 执行完成（部分命令失败）", version);
        }
    }
    
    println!("\n所有数据库迁移完成!");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取 .env 文件
    dotenv::from_filename(".env").ok();
    
    // 获取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL 未设置");
    
    // 连接数据库
    println!("连接到数据库...");
    let pool = PgPool::connect(&database_url).await?;
    
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("请指定要执行的迁移类型:");
        println!("  basic    - 运行基础迁移");
        println!("  permission - 运行权限迁移");
        println!("  ai       - 运行AI权限迁移");
        println!("  all      - 运行所有迁移");
        println!("  check    - 检查教师权限");
        println!("  fix      - 修复教师权限");
        return Ok(());
    }
    
    match args[1].as_str() {
        "basic" => run_basic_migration(&pool).await?,
        "permission" => run_permission_migration(&pool).await?,
        "ai" => run_ai_permission_migration(&pool).await?,
        "all" => run_all_migrations(&pool).await?,
        "check" => check_teacher_permissions(&pool).await?,
        "fix" => fix_teacher_permissions(&pool).await?,
        _ => {
            println!("未知的迁移类型: {}", args[1]);
            println!("可用的迁移类型: basic, permission, ai, all, check, fix");
        }
    }
    
    Ok(())
}
