use sqlx::PgPool;
use std::fs;
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
        match sqlx::query(command).execute(&pool).await {
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
    match sqlx::query(alter_table_sql).execute(&pool).await {
        Ok(_) => println!("value字段检查完成"),
        Err(e) => println!("警告: 添加value字段可能失败: {}", e),
    }
    
    println!("权限数据库迁移成功!");
    
    // 从YAML模板文件加载权限
    println!("从YAML模板文件初始化权限...");
    init_permissions_from_templates(&pool).await?;
    
    println!("权限初始化完成!");
    
    Ok(())
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
