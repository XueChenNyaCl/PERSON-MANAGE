use std::collections::{HashMap, HashSet};
use std::fs;
use sqlx::{PgPool, postgres::PgRow, Row};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// 权限管理器
pub struct PermissionManager {
    pool: PgPool,
}

/// 权限检查结果
#[derive(Debug, PartialEq)]
pub enum PermissionResult {
    Allowed,
    Denied,
    NotSet,
}

impl PermissionManager {
    /// 创建新的权限管理器
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 检查用户是否拥有特定权限
    pub async fn check_permission(&self, user_id: Uuid, permission: &str) -> PermissionResult {
        // 获取用户角色
        let role = match self.get_user_role(user_id).await {
            Ok(role) => role,
            Err(_) => return PermissionResult::NotSet,
        };

        // 获取用户的所有有效权限
        let user_permissions = self.get_user_effective_permissions(user_id, &role).await;

        // 检查权限
        self.evaluate_permission(&user_permissions, permission)
    }

    /// 获取用户角色
    async fn get_user_role(&self, user_id: Uuid) -> Result<String, sqlx::Error> {
        let row = sqlx::query("SELECT role FROM persons WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(row.get("role")),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    /// 获取用户的所有有效权限（包括角色权限和用户特定权限）
    async fn get_user_effective_permissions(&self, user_id: Uuid, role: &str) -> Vec<PermissionNode> {
        let mut permissions = Vec::new();

        // 获取角色权限
        if let Ok(role_perms) = self.get_role_permissions(role).await {
            permissions.extend(role_perms);
        }

        // 获取用户特定权限（覆盖角色权限）
        if let Ok(user_perms) = self.get_user_specific_permissions(user_id).await {
            permissions.extend(user_perms);
        }

        permissions
    }

    /// 获取角色权限
    pub async fn get_role_permissions(&self, role: &str) -> Result<Vec<PermissionNode>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT permission, value, priority FROM permissions 
             WHERE role = $1 
             ORDER BY priority DESC"
        )
        .bind(role)
        .fetch_all(&self.pool)
        .await?;

        let mut permissions = Vec::new();
        for row in rows {
            let permission_str: String = row.get("permission");
            let value: bool = row.get("value");
            let priority: i32 = row.get("priority");
            
            permissions.push(PermissionNode::new(&permission_str, value, priority));
        }

        Ok(permissions)
    }

    /// 获取用户特定权限
    pub async fn get_user_specific_permissions(&self, user_id: Uuid) -> Result<Vec<PermissionNode>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT permission, value, priority FROM user_permissions 
             WHERE user_id = $1 
             ORDER BY priority DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut permissions = Vec::new();
        for row in rows {
            let permission_str: String = row.get("permission");
            let value: bool = row.get("value");
            let priority: i32 = row.get("priority");
            
            let mut node = PermissionNode::from_string(&permission_str, priority);
            node.value = value;
            permissions.push(node);
        }

        Ok(permissions)
    }

    /// 评估权限
    fn evaluate_permission(&self, permissions: &[PermissionNode], target_permission: &str) -> PermissionResult {
        let mut matched_permissions = Vec::new();

        // 查找所有匹配的权限节点
        for node in permissions {
            if node.matches(target_permission) {
                matched_permissions.push(node);
            }
        }

        // 按优先级排序（高优先级在前）
        matched_permissions.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 使用最高优先级的匹配节点
        if let Some(highest_priority) = matched_permissions.first() {
            if highest_priority.value {
                PermissionResult::Allowed
            } else {
                PermissionResult::Denied
            }
        } else {
            PermissionResult::NotSet
        }
    }

    /// 添加角色权限
    pub async fn add_role_permission(&self, role: &str, permission: &str, value: bool, priority: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO permissions (role, permission, value, priority) 
             VALUES ($1, $2, $3, $4) 
             ON CONFLICT (role, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority"
        )
        .bind(role)
        .bind(permission)
        .bind(value)
        .bind(priority)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 移除角色权限
    pub async fn remove_role_permission(&self, role: &str, permission: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM permissions WHERE role = $1 AND permission = $2"
        )
        .bind(role)
        .bind(permission)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 添加用户特定权限
    pub async fn add_user_permission(&self, user_id: Uuid, permission: &str, value: bool, priority: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO user_permissions (user_id, permission, value, priority) 
             VALUES ($1, $2, $3, $4) 
             ON CONFLICT (user_id, permission) DO UPDATE SET value = EXCLUDED.value, priority = EXCLUDED.priority"
        )
        .bind(user_id)
        .bind(permission)
        .bind(value)
        .bind(priority)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 移除用户特定权限
    pub async fn remove_user_permission(&self, user_id: Uuid, permission: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM user_permissions WHERE user_id = $1 AND permission = $2"
        )
        .bind(user_id)
        .bind(permission)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取用户的所有权限（用于登录时返回）
    pub async fn get_user_permissions_list(&self, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
        let role = self.get_user_role(user_id).await?;
        
        // 获取所有允许的权限
        let mut allowed_permissions = HashSet::new();
        let effective_permissions = self.get_user_effective_permissions(user_id, &role).await;
        
        for node in effective_permissions {
            if node.value {
                // 如果是通配符权限，我们需要展开（这里简化处理）
                if node.permission.contains('*') {
                    // 对于通配符权限，我们只返回通配符本身
                    // 实际应用中可能需要更复杂的展开逻辑
                    allowed_permissions.insert(node.permission.clone());
                } else {
                    allowed_permissions.insert(node.permission.clone());
                }
            }
        }
        
        Ok(allowed_permissions.into_iter().collect())
    }

    /// 检查权限，如果拒绝则返回AppError
    pub async fn require_permission(&self, user_id: Uuid, permission: &str) -> Result<(), crate::core::error::AppError> {
        match self.check_permission(user_id, permission).await {
            PermissionResult::Allowed => Ok(()),
            PermissionResult::Denied => Err(crate::core::error::AppError::Auth(
                format!("没有权限执行此操作: {}", permission)
            )),
            PermissionResult::NotSet => Err(crate::core::error::AppError::Auth(
                format!("权限未设置: {}", permission)
            )),
        }
    }
}

/// 权限节点
#[derive(Debug, Clone)]
pub struct PermissionNode {
    pub permission: String,  // 权限字符串，如 "creat.students", "class.*", "-creat.students"
    pub value: bool,         // 权限值：true=允许，false=拒绝
    pub priority: i32,       // 优先级
}

impl PermissionNode {
    /// 创建新的权限节点
    fn new(permission: &str, value: bool, priority: i32) -> Self {
        Self {
            permission: permission.to_string(),
            value,
            priority,
        }
    }

    /// 从字符串创建权限节点（用于从YAML解析，支持-前缀表示否定权限）
    fn from_string(permission_str: &str, priority: i32) -> Self {
        let (value, permission) = if permission_str.starts_with('-') {
            (false, permission_str[1..].to_string())
        } else {
            (true, permission_str.to_string())
        };
        
        Self {
            permission,
            value,
            priority,
        }
    }

    /// 检查权限节点是否匹配目标权限
    fn matches(&self, target_permission: &str) -> bool {
        self.permission_matches_pattern(&self.permission, target_permission)
    }

    /// 通配符匹配逻辑
    fn permission_matches_pattern(&self, pattern: &str, target: &str) -> bool {
        // 简单通配符匹配：* 匹配任意字符序列
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let target_parts: Vec<&str> = target.split('.').collect();

        if pattern_parts.len() != target_parts.len() && !pattern.contains('*') {
            return false;
        }

        let mut pattern_iter = pattern_parts.iter();
        let mut target_iter = target_parts.iter();

        loop {
            match (pattern_iter.next(), target_iter.next()) {
                (Some(&"*"), Some(_)) => continue,
                (Some(&pattern_part), Some(&target_part)) if pattern_part == target_part => continue,
                (Some(&pattern_part), Some(&target_part)) if pattern_part != target_part => return false,
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

/// 工具函数：检查用户权限（简化接口）
pub async fn check_user_permission(pool: &PgPool, user_id: Uuid, permission: &str) -> bool {
    let manager = PermissionManager::new(pool.clone());
    
    match manager.check_permission(user_id, permission).await {
        PermissionResult::Allowed => true,
        _ => false,
    }
}

/// 工具函数：获取用户权限列表
pub async fn get_user_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let manager = PermissionManager::new(pool.clone());
    manager.get_user_permissions_list(user_id).await
}

// ==================== 权限模板系统 ====================

/// 权限模板项
#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionTemplateItem {
    pub permission: String,
    pub priority: i32,
}

/// 权限模板
#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionTemplate {
    pub permissions: Vec<PermissionTemplateItem>,
}

impl PermissionTemplate {
    /// 从YAML文件加载权限模板
    pub fn from_yaml_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let yaml_content = fs::read_to_string(file_path)?;
        let template: PermissionTemplate = serde_yaml::from_str(&yaml_content)?;
        Ok(template)
    }
    
    /// 从YAML字符串加载权限模板
    pub fn from_yaml_str(yaml_content: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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
        
        println!("=== YAML TEMPLATE DEBUG: Processed YAML ===");
        println!("First 5 lines of processed YAML:");
        for (i, line) in processed_content.lines().take(5).enumerate() {
            println!("{}: {}", i+1, line);
        }
        
        let template: PermissionTemplate = serde_yaml::from_str(&processed_content)?;
        Ok(template)
    }
    
    /// 应用模板到指定角色
    pub async fn apply_to_role(&self, pool: &PgPool, role: &str) -> Result<(), sqlx::Error> {
        let manager = PermissionManager::new(pool.clone());
        
        for item in &self.permissions {
            // 处理否定权限（以-开头）
            let (permission_str, value) = if item.permission.starts_with('-') {
                (&item.permission[1..], false)
            } else {
                (item.permission.as_str(), true)
            };
            manager.add_role_permission(role, permission_str, value, item.priority).await?;
        }
        
        Ok(())
    }
    
    /// 应用模板到指定用户（作为用户特定权限）
    pub async fn apply_to_user(&self, pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        // 首先清除用户现有的特定权限（可选，根据需求决定）
        // sqlx::query("DELETE FROM user_permissions WHERE user_id = $1")
        //     .bind(user_id)
        //     .execute(pool)
        //     .await?;
        
        for item in &self.permissions {
            let value = !item.permission.starts_with('-');
            let permission = if item.permission.starts_with('-') {
                &item.permission[1..]
            } else {
                &item.permission
            };
            
            // 添加用户特定权限
            sqlx::query(
                "INSERT INTO user_permissions (user_id, permission, value, priority) 
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (user_id, permission) DO UPDATE SET 
                 value = EXCLUDED.value, priority = EXCLUDED.priority"
            )
            .bind(user_id)
            .bind(permission)
            .bind(value)
            .bind(item.priority)
            .execute(pool)
            .await?;
        }
        
        Ok(())
    }
}

/// 加载默认权限模板
pub fn load_default_template(role: &str) -> Result<PermissionTemplate, Box<dyn std::error::Error + Send + Sync>> {
    let template_path = format!("templates/permissions/{}.yaml", role);
    PermissionTemplate::from_yaml_file(&template_path)
}

/// 为新用户应用角色模板
pub async fn apply_role_template_to_user(pool: &PgPool, user_id: Uuid, role: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match load_default_template(role) {
        Ok(template) => {
            // 应用模板到角色（如果角色权限尚未设置）
            template.apply_to_role(pool, role).await?;
            
            // 也可以选择性地应用到用户特定权限
            // template.apply_to_user(pool, user_id, role).await?;
            
            Ok(())
        }
        Err(e) => {
            // 如果模板文件不存在，使用默认基础权限
            println!("警告: 找不到角色 {} 的权限模板: {}, 使用默认权限", role, e);
            
            // 添加最基本的dashboard.view权限
            let manager = PermissionManager::new(pool.clone());
            manager.add_role_permission(role, "dashboard.view", true, 10).await?;
            
            Ok(())
        }
    }
}