use sqlx::PgPool;
use std::env;
use std::fs;
use std::path::PathBuf;

fn search_roots() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    Ok(vec![
        cwd.clone(),
        cwd.join("backend"),
        manifest_dir.clone(),
        manifest_dir.join(".."),
    ])
}

/// 解析迁移文件路径，兼容从仓库根目录或 backend 目录启动
fn resolve_migration_path(filename: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let roots = search_roots()?;
    let mut checked_paths = Vec::new();

    for root in roots {
        let path = root.join("migrations").join(filename);
        checked_paths.push(path.display().to_string());
        if path.exists() {
            return Ok(path);
        }
    }

    Err(format!(
        "无法定位迁移文件 {}，已检查路径:\n{}",
        filename,
        checked_paths.join("\n")
    )
    .into())
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

        // 检查是否是 $tag$ 或 $$ 开始
        if !in_dollar_quote {
            if let Some(tag) = find_dollar_quote_delimiter(line) {
                // 若同一行出现偶数次分隔符，说明同一行已闭合，无需进入 dollar quote 模式
                let occurrences = line.matches(&tag).count();
                if occurrences % 2 == 1 {
                    in_dollar_quote = true;
                    dollar_quote_tag = tag;
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

/// 在一行 SQL 中查找第一个不在单引号字符串里的 dollar-quote 分隔符（如 $$ / $func$）
fn find_dollar_quote_delimiter(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut in_single_quote = false;

    while i < bytes.len() {
        let ch = bytes[i] as char;

        if ch == '\'' {
            // SQL 单引号转义：''
            if in_single_quote && i + 1 < bytes.len() && bytes[i + 1] as char == '\'' {
                i += 2;
                continue;
            }
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if !in_single_quote && ch == '$' {
            let start = i;
            i += 1;

            // 允许空 tag（即 $$）或字母数字下划线 tag（如 $func$）
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c == '$' {
                    let tag = &line[start + 1..i];
                    if tag.is_empty() || tag.chars().all(|t| t.is_ascii_alphanumeric() || t == '_')
                    {
                        return Some(format!("${}$", tag));
                    }
                    break;
                }
                if !(c.is_ascii_alphanumeric() || c == '_') {
                    break;
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    None
}

fn is_ignorable_migration_error(err: &sqlx::Error) -> bool {
    let err_msg = err.to_string();
    err_msg.contains("already exists")
        || err_msg.contains("42710") // duplicate object
        || err_msg.contains("42P07") // duplicate table
        || err_msg.contains("42P16") // duplicate column
}

/// 运行完整迁移
async fn run_migration(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 读取迁移文件
    let migration_path = resolve_migration_path("001_complete_schema.sql")?;
    println!("使用迁移文件: {}", migration_path.display());
    let migration_content = fs::read_to_string(&migration_path)?;

    // 拆分成单独的 SQL 命令并执行（支持 dollar-quote）
    let commands = split_sql_commands(&migration_content);
    println!("共 {} 个 SQL 命令需要执行\n", commands.len());

    // 执行所有命令
    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for (i, command) in commands.iter().enumerate() {
        let cmd_preview: String = command.chars().take(60).collect();
        print!("[{:>3}/{}] {}... ", i + 1, commands.len(), cmd_preview);

        match sqlx::query(command).execute(pool).await {
            Ok(_) => {
                println!("OK");
                success_count += 1;
            }
            Err(e) => {
                if is_ignorable_migration_error(&e) {
                    println!("已存在，跳过");
                    skip_count += 1;
                } else {
                    println!("错误: {}", e);
                    error_count += 1;
                }
            }
        }
    }

    println!("\n========================================");
    println!("迁移完成!");
    println!("  成功: {}", success_count);
    println!("  跳过: {}", skip_count);
    println!("  失败: {}", error_count);
    println!("========================================");

    if error_count > 0 {
        return Err(format!("迁移完成，但有 {} 个错误", error_count).into());
    }

    Ok(())
}

/// 检查数据库连接和基本状态
async fn check_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 数据库状态检查 ===\n");

    // 检查表数量
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'"
    )
    .fetch_one(pool)
    .await?;
    println!("数据库表数量: {}", table_count);

    // 检查人员数量
    let person_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM persons")
        .fetch_one(pool)
        .await?;
    println!("人员记录数量: {}", person_count);

    // 检查班级数量
    let class_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM classes")
        .fetch_one(pool)
        .await?;
    println!("班级记录数量: {}", class_count);

    // 检查权限数量
    let permission_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM permissions")
        .fetch_one(pool)
        .await?;
    println!("权限记录数量: {}", permission_count);

    // 检查特殊用户数量
    let special_user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM special_users")
        .fetch_one(pool)
        .await?;
    println!("特殊用户数量: {}", special_user_count);

    println!("\n数据库状态检查完成!");
    Ok(())
}

/// 重置数据库（删除所有表并重新迁移）
async fn reset_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n!!! 警告: 这将删除所有数据 !!!");
    println!("正在重置数据库...\n");

    // 删除所有表（按照依赖顺序）
    let drop_tables = vec![
        "operation_logs",
        "special_users",
        "chat_messages",
        "chat_conversation_members",
        "chat_conversations",
        "scores",
        "notices",
        "attendances",
        "group_score_records",
        "group_members",
        "class_groups",
        "user_permissions",
        "permissions",
        "teacher_class",
        "student_parent",
        "parents",
        "students",
        "teachers",
        "classes",
        "departments",
        "ai_settings",
        "persons",
    ];

    for table in drop_tables {
        let sql = format!("DROP TABLE IF EXISTS {} CASCADE", table);
        match sqlx::query(&sql).execute(pool).await {
            Ok(_) => println!("已删除表: {}", table),
            Err(e) => println!("删除表 {} 失败: {}", table, e),
        }
    }

    // 删除函数
    let _ = sqlx::query("DROP FUNCTION IF EXISTS update_updated_at_column()")
        .execute(pool)
        .await;
    println!("已删除触发器函数");

    println!("\n数据库重置完成，开始重新迁移...\n");

    // 重新运行迁移
    run_migration(pool).await
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
        println!("学校管理系统数据库迁移工具\n");
        println!("用法: cargo run --bin run_migration <命令>\n");
        println!("可用命令:");
        println!("  migrate  - 运行数据库迁移（创建/更新表结构）");
        println!("  check    - 检查数据库状态");
        println!("  reset    - 重置数据库（删除所有数据并重新迁移）");
        println!("  help     - 显示此帮助信息");
        return Ok(());
    }

    match args[1].as_str() {
        "migrate" => run_migration(&pool).await?,
        "check" => check_database(&pool).await?,
        "reset" => reset_database(&pool).await?,
        "help" | "--help" | "-h" => {
            println!("学校管理系统数据库迁移工具\n");
            println!("用法: cargo run --bin run_migration <命令>\n");
            println!("可用命令:");
            println!("  migrate  - 运行数据库迁移（创建/更新表结构）");
            println!("  check    - 检查数据库状态");
            println!("  reset    - 重置数据库（删除所有数据并重新迁移）");
            println!("  help     - 显示此帮助信息");
        }
        _ => {
            println!("未知命令: {}", args[1]);
            println!("使用 'help' 查看可用命令");
        }
    }

    Ok(())
}
