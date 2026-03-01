use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;

// ========== 数据实体定义 ==========

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClassInfo {
    pub id: Uuid,
    pub name: String,
    pub grade: i16,
    pub teacher_id: Option<Uuid>,
    pub teacher_name: Option<String>,
    pub student_count: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassDetailInfo {
    pub basic_info: ClassInfo,
    pub students: Vec<StudentInfo>,
    pub teachers: Vec<TeacherInfo>,
    pub groups: Vec<GroupInfo>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupInfo {
    pub id: Uuid,
    pub name: String,
    pub class_id: Uuid,
    pub class_name: String,
    pub member_count: i64,
    pub total_score: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupDetailInfo {
    pub basic_info: GroupInfo,
    pub members: Vec<GroupMemberInfo>,
    pub score_records: Vec<ScoreRecord>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupMemberInfo {
    pub id: Uuid,
    pub name: String,
    pub student_no: Option<String>,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScoreRecord {
    pub id: Uuid,
    pub score: i32,
    pub reason: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub operator_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DepartmentInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub teacher_count: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentDetailInfo {
    pub basic_info: DepartmentInfo,
    pub teachers: Vec<TeacherInfo>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudentInfo {
    pub id: Uuid,
    pub name: String,
    pub student_no: String,
    pub class_id: Option<Uuid>,
    pub class_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeacherInfo {
    pub id: Uuid,
    pub name: String,
    pub employee_no: String,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

// ========== 数据查询请求/响应 ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct DataQueryRequest {
    /// 查询类型: class_list, class_detail, group_list, group_detail, 
    ///          department_list, department_detail, overview
    pub query_type: String,
    /// 可选的ID参数（用于详情查询），可以是UUID或名称
    pub id: Option<String>,
    /// 是否返回Markdown格式
    pub format_as_markdown: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataQueryResponse {
    /// 查询结果（JSON格式或Markdown字符串）
    pub data: String,
    /// 数据类型: json, markdown
    pub data_type: String,
    /// 用户权限列表
    pub user_permissions: Vec<String>,
}

// ========== 数据服务层 ==========

pub struct ClassDataService {
    pool: PgPool,
}

impl ClassDataService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有班级列表
    pub async fn get_all_classes(&self) -> Result<Vec<ClassInfo>, AppError> {
        let classes = sqlx::query_as::<_, ClassInfo>(
            r#"
            SELECT 
                c.id,
                c.name,
                c.grade,
                c.teacher_id,
                p.name as teacher_name,
                COUNT(DISTINCT s.person_id) as student_count,
                c.created_at
            FROM classes c
            LEFT JOIN persons p ON c.teacher_id = p.id
            LEFT JOIN students s ON c.id = s.class_id
            GROUP BY c.id, c.name, c.grade, c.teacher_id, p.name, c.created_at
            ORDER BY c.grade, c.name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("获取班级列表失败: {}", e)))?;

        Ok(classes)
    }

    /// 获取班级详细信息
    pub async fn get_class_detail(&self, class_id: Uuid) -> Result<ClassDetailInfo, AppError> {
        // 获取基本信息
        let basic_info = sqlx::query_as::<_, ClassInfo>(
            r#"
            SELECT 
                c.id,
                c.name,
                c.grade,
                c.teacher_id,
                p.name as teacher_name,
                COUNT(DISTINCT s.person_id) as student_count,
                c.created_at
            FROM classes c
            LEFT JOIN persons p ON c.teacher_id = p.id
            LEFT JOIN students s ON c.id = s.class_id
            WHERE c.id = $1
            GROUP BY c.id, c.name, c.grade, c.teacher_id, p.name, c.created_at
            "#,
        )
        .bind(class_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("获取班级信息失败: {}", e)))?
        .ok_or(AppError::NotFound)?;

        // 获取学生列表
        let students = sqlx::query_as::<_, StudentInfo>(
            r#"
            SELECT 
                p.id,
                p.name,
                s.student_no,
                c.id as class_id,
                c.name as class_name,
                p.email,
                p.phone
            FROM persons p
            JOIN students s ON p.id = s.person_id
            JOIN classes c ON s.class_id = c.id
            WHERE c.id = $1
            ORDER BY s.student_no
            "#,
        )
        .bind(class_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // 获取教师列表（通过teacher_class关联表）
        let teachers = sqlx::query_as::<_, TeacherInfo>(
            r#"
            SELECT 
                p.id,
                p.name,
                t.employee_no,
                d.id as department_id,
                d.name as department_name,
                p.email,
                p.phone
            FROM persons p
            JOIN teachers t ON p.id = t.person_id
            JOIN teacher_class tc ON t.person_id = tc.teacher_id
            LEFT JOIN departments d ON t.department_id = d.id
            WHERE tc.class_id = $1
            ORDER BY p.name
            "#,
        )
        .bind(class_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // 获取小组列表（使用class_groups表）
        let groups = sqlx::query_as::<_, GroupInfo>(
            r#"
            SELECT 
                cg.id,
                cg.name,
                cg.class_id,
                c.name as class_name,
                COUNT(DISTINCT gm.person_id) as member_count,
                COALESCE(SUM(gsr.score_change), 0) as total_score
            FROM class_groups cg
            JOIN classes c ON cg.class_id = c.id
            LEFT JOIN group_members gm ON cg.id = gm.group_id
            LEFT JOIN group_score_records gsr ON cg.id = gsr.group_id
            WHERE cg.class_id = $1
            GROUP BY cg.id, cg.name, cg.class_id, c.name
            ORDER BY cg.name
            "#,
        )
        .bind(class_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(ClassDetailInfo {
            basic_info,
            students,
            teachers,
            groups,
        })
    }
}

pub struct GroupDataService {
    pool: PgPool,
}

impl GroupDataService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有小组列表
    pub async fn get_all_groups(&self) -> Result<Vec<GroupInfo>, AppError> {
        let groups = sqlx::query_as::<_, GroupInfo>(
            r#"
            SELECT 
                cg.id,
                cg.name,
                cg.class_id,
                c.name as class_name,
                COUNT(DISTINCT gm.person_id) as member_count,
                COALESCE(SUM(gsr.score_change), 0) as total_score
            FROM class_groups cg
            JOIN classes c ON cg.class_id = c.id
            LEFT JOIN group_members gm ON cg.id = gm.group_id
            LEFT JOIN group_score_records gsr ON cg.id = gsr.group_id
            GROUP BY cg.id, cg.name, cg.class_id, c.name
            ORDER BY c.name, cg.name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("获取小组列表失败: {}", e)))?;

        Ok(groups)
    }

    /// 获取小组详细信息
    pub async fn get_group_detail(&self, group_id: Uuid) -> Result<GroupDetailInfo, AppError> {
        // 获取基本信息
        let basic_info = sqlx::query_as::<_, GroupInfo>(
            r#"
            SELECT 
                cg.id,
                cg.name,
                cg.class_id,
                c.name as class_name,
                COUNT(DISTINCT gm.person_id) as member_count,
                COALESCE(SUM(gsr.score_change), 0) as total_score
            FROM class_groups cg
            JOIN classes c ON cg.class_id = c.id
            LEFT JOIN group_members gm ON cg.id = gm.group_id
            LEFT JOIN group_score_records gsr ON cg.id = gsr.group_id
            WHERE cg.id = $1
            GROUP BY cg.id, cg.name, cg.class_id, c.name
            "#,
        )
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("获取小组信息失败: {}", e)))?
        .ok_or(AppError::NotFound)?;

        // 获取成员列表
        let members = sqlx::query_as::<_, GroupMemberInfo>(
            r#"
            SELECT 
                p.id,
                p.name,
                s.student_no,
                '成员' as role
            FROM group_members gm
            JOIN persons p ON gm.person_id = p.id
            LEFT JOIN students s ON p.id = s.person_id
            WHERE gm.group_id = $1
            ORDER BY p.name
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // 获取积分记录
        let score_records = sqlx::query_as::<_, ScoreRecord>(
            r#"
            SELECT 
                gsr.id,
                gsr.score_change as score,
                gsr.reason,
                gsr.created_at,
                p.name as operator_name
            FROM group_score_records gsr
            LEFT JOIN persons p ON gsr.created_by = p.id
            WHERE gsr.group_id = $1
            ORDER BY gsr.created_at DESC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(GroupDetailInfo {
            basic_info,
            members,
            score_records,
        })
    }
}

pub struct DepartmentDataService {
    pool: PgPool,
}

impl DepartmentDataService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有部门列表
    pub async fn get_all_departments(&self) -> Result<Vec<DepartmentInfo>, AppError> {
        let departments = sqlx::query_as::<_, DepartmentInfo>(
            r#"
            SELECT 
                d.id,
                d.name,
                NULL as description,
                COUNT(DISTINCT t.person_id) as teacher_count,
                d.created_at
            FROM departments d
            LEFT JOIN teachers t ON d.id = t.department_id
            GROUP BY d.id, d.name, d.created_at
            ORDER BY d.name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("获取部门列表失败: {}", e)))?;

        Ok(departments)
    }

    /// 获取部门详细信息
    pub async fn get_department_detail(&self, dept_id: Uuid) -> Result<DepartmentDetailInfo, AppError> {
        // 获取基本信息
        let basic_info = sqlx::query_as::<_, DepartmentInfo>(
            r#"
            SELECT 
                d.id,
                d.name,
                NULL as description,
                COUNT(DISTINCT t.person_id) as teacher_count,
                d.created_at
            FROM departments d
            LEFT JOIN teachers t ON d.id = t.department_id
            WHERE d.id = $1
            GROUP BY d.id, d.name, d.created_at
            "#,
        )
        .bind(dept_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("获取部门信息失败: {}", e)))?
        .ok_or(AppError::NotFound)?;

        // 获取教师列表
        let teachers = sqlx::query_as::<_, TeacherInfo>(
            r#"
            SELECT 
                p.id,
                p.name,
                t.employee_no,
                d.id as department_id,
                d.name as department_name,
                p.email,
                p.phone
            FROM persons p
            JOIN teachers t ON p.id = t.person_id
            LEFT JOIN departments d ON t.department_id = d.id
            WHERE t.department_id = $1
            ORDER BY p.name
            "#,
        )
        .bind(dept_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(DepartmentDetailInfo {
            basic_info,
            teachers,
        })
    }
}

// ========== Markdown格式化器 ==========

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    /// 格式化班级列表为Markdown
    pub fn format_class_list(classes: &[ClassInfo]) -> String {
        let mut markdown = String::new();
        
        markdown.push_str("# 班级列表\n\n");
        
        if classes.is_empty() {
            markdown.push_str("*暂无班级数据*\n");
            return markdown;
        }

        // 按年级分组
        let mut grade_groups: std::collections::HashMap<i16, Vec<&ClassInfo>> = std::collections::HashMap::new();
        for class in classes {
            grade_groups.entry(class.grade).or_default().push(class);
        }

        let mut grades: Vec<i16> = grade_groups.keys().copied().collect();
        grades.sort();

        for grade in grades {
            markdown.push_str(&format!("## {}年级\n\n", grade));
            markdown.push_str("| 班级名称 | 班主任 | 学生人数 |\n");
            markdown.push_str("|---------|--------|----------|\n");

            if let Some(classes_in_grade) = grade_groups.get(&grade) {
                for class in classes_in_grade {
                    let teacher_name = class.teacher_name.as_deref().unwrap_or("未设置");
                    markdown.push_str(&format!(
                        "| {} | {} | {} |\n",
                        class.name,
                        teacher_name,
                        class.student_count
                    ));
                }
            }
            markdown.push('\n');
        }

        markdown
    }

    /// 格式化班级详情为Markdown
    pub fn format_class_detail(detail: &ClassDetailInfo) -> String {
        let mut markdown = String::new();
        let class = &detail.basic_info;

        markdown.push_str(&format!("# {}（{}年级）\n\n", class.name, class.grade));
        
        // 基本信息
        markdown.push_str("## 基本信息\n\n");
        markdown.push_str(&format!("- **班级名称**: {}\n", class.name));
        markdown.push_str(&format!("- **年级**: {}年级\n", class.grade));
        markdown.push_str(&format!(
            "- **班主任**: {}\n",
            class.teacher_name.as_deref().unwrap_or("未设置")
        ));
        markdown.push_str(&format!("- **学生人数**: {}\n", class.student_count));
        markdown.push('\n');

        // 教师列表
        markdown.push_str("## 任课教师\n\n");
        if detail.teachers.is_empty() {
            markdown.push_str("*暂无教师信息*\n\n");
        } else {
            markdown.push_str("| 姓名 | 工号 | 部门 |\n");
            markdown.push_str("|------|------|------|\n");
            for teacher in &detail.teachers {
                markdown.push_str(&format!(
                    "| {} | {} | {} |\n",
                    teacher.name,
                    teacher.employee_no,
                    teacher.department_name.as_deref().unwrap_or("未分配")
                ));
            }
            markdown.push('\n');
        }

        // 学生列表
        markdown.push_str("## 学生名单\n\n");
        if detail.students.is_empty() {
            markdown.push_str("*暂无学生信息*\n\n");
        } else {
            markdown.push_str("| 学号 | 姓名 |\n");
            markdown.push_str("|------|------|\n");
            for student in &detail.students {
                markdown.push_str(&format!(
                    "| {} | {} |\n",
                    student.student_no,
                    student.name
                ));
            }
            markdown.push('\n');
        }

        // 小组列表
        markdown.push_str("## 学习小组\n\n");
        if detail.groups.is_empty() {
            markdown.push_str("*暂无小组信息*\n\n");
        } else {
            markdown.push_str("| 小组名称 | 成员人数 | 总积分 |\n");
            markdown.push_str("|---------|----------|--------|\n");
            for group in &detail.groups {
                markdown.push_str(&format!(
                    "| {} | {} | {} |\n",
                    group.name,
                    group.member_count,
                    group.total_score.unwrap_or(0)
                ));
            }
            markdown.push('\n');
        }

        markdown
    }

    /// 格式化小组列表为Markdown
    pub fn format_group_list(groups: &[GroupInfo]) -> String {
        let mut markdown = String::new();
        
        markdown.push_str("# 小组列表\n\n");
        
        if groups.is_empty() {
            markdown.push_str("*暂无小组数据*\n");
            return markdown;
        }

        // 按班级分组
        let mut class_groups: std::collections::HashMap<String, Vec<&GroupInfo>> = std::collections::HashMap::new();
        for group in groups {
            class_groups.entry(group.class_name.clone()).or_default().push(group);
        }

        for (class_name, groups_in_class) in class_groups {
            markdown.push_str(&format!("## {}\n\n", class_name));
            markdown.push_str("| 小组名称 | 成员人数 | 总积分 |\n");
            markdown.push_str("|---------|----------|--------|\n");

            for group in groups_in_class {
                markdown.push_str(&format!(
                    "| {} | {} | {} |\n",
                    group.name,
                    group.member_count,
                    group.total_score.unwrap_or(0)
                ));
            }
            markdown.push('\n');
        }

        markdown
    }

    /// 格式化小组详情为Markdown
    pub fn format_group_detail(detail: &GroupDetailInfo) -> String {
        let mut markdown = String::new();
        let group = &detail.basic_info;

        markdown.push_str(&format!("# {}\n\n", group.name));
        
        // 基本信息
        markdown.push_str("## 基本信息\n\n");
        markdown.push_str(&format!("- **小组名称**: {}\n", group.name));
        markdown.push_str(&format!("- **所属班级**: {}\n", group.class_name));
        markdown.push_str(&format!("- **成员人数**: {}\n", group.member_count));
        markdown.push_str(&format!("- **总积分**: {}\n", group.total_score.unwrap_or(0)));
        markdown.push('\n');

        // 成员列表
        markdown.push_str("## 成员列表\n\n");
        if detail.members.is_empty() {
            markdown.push_str("*暂无成员*\n\n");
        } else {
            markdown.push_str("| 学号 | 姓名 | 角色 |\n");
            markdown.push_str("|------|------|------|\n");
            for member in &detail.members {
                markdown.push_str(&format!(
                    "| {} | {} | {} |\n",
                    member.student_no.as_deref().unwrap_or("-"),
                    member.name,
                    member.role
                ));
            }
            markdown.push('\n');
        }

        // 积分记录
        markdown.push_str("## 积分记录\n\n");
        if detail.score_records.is_empty() {
            markdown.push_str("*暂无积分记录*\n\n");
        } else {
            markdown.push_str("| 积分 | 原因 | 操作人 | 时间 |\n");
            markdown.push_str("|------|------|--------|------|\n");
            for record in &detail.score_records {
                markdown.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    record.score,
                    record.reason,
                    record.operator_name.as_deref().unwrap_or("系统"),
                    record.created_at.format("%Y-%m-%d %H:%M")
                ));
            }
            markdown.push('\n');
        }

        markdown
    }

    /// 格式化部门列表为Markdown
    pub fn format_department_list(departments: &[DepartmentInfo]) -> String {
        let mut markdown = String::new();
        
        markdown.push_str("# 部门列表\n\n");
        
        if departments.is_empty() {
            markdown.push_str("*暂无部门数据*\n");
            return markdown;
        }

        markdown.push_str("| 部门名称 | 教师人数 |\n");
        markdown.push_str("|---------|----------|\n");

        for dept in departments {
            markdown.push_str(&format!(
                "| {} | {} |\n",
                dept.name,
                dept.teacher_count
            ));
        }

        markdown
    }

    /// 格式化部门详情为Markdown
    pub fn format_department_detail(detail: &DepartmentDetailInfo) -> String {
        let mut markdown = String::new();
        let dept = &detail.basic_info;

        markdown.push_str(&format!("# {}\n\n", dept.name));
        
        // 基本信息
        markdown.push_str("## 基本信息\n\n");
        markdown.push_str(&format!("- **部门名称**: {}\n", dept.name));
        markdown.push_str(&format!("- **教师人数**: {}\n", dept.teacher_count));
        markdown.push('\n');

        // 教师列表
        markdown.push_str("## 教师名单\n\n");
        if detail.teachers.is_empty() {
            markdown.push_str("*暂无教师信息*\n\n");
        } else {
            markdown.push_str("| 姓名 | 工号 | 邮箱 | 电话 |\n");
            markdown.push_str("|------|------|------|------|\n");
            for teacher in &detail.teachers {
                markdown.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    teacher.name,
                    teacher.employee_no,
                    teacher.email.as_deref().unwrap_or("-"),
                    teacher.phone.as_deref().unwrap_or("-")
                ));
            }
            markdown.push('\n');
        }

        markdown
    }

    /// 格式化统计概览为Markdown
    pub fn format_overview(
        classes: &[ClassInfo],
        groups: &[GroupInfo],
        departments: &[DepartmentInfo],
    ) -> String {
        let mut markdown = String::new();

        markdown.push_str("# 学校数据概览\n\n");

        // 统计卡片
        markdown.push_str("## 数据统计\n\n");
        markdown.push_str(&format!("- **班级总数**: {}\n", classes.len()));
        markdown.push_str(&format!("- **小组总数**: {}\n", groups.len()));
        markdown.push_str(&format!("- **部门总数**: {}\n", departments.len()));
        
        let total_students: i64 = classes.iter().map(|c| c.student_count).sum();
        markdown.push_str(&format!("- **学生总数**: {}\n", total_students));
        
        let total_teachers: i64 = departments.iter().map(|d| d.teacher_count).sum();
        markdown.push_str(&format!("- **教师总数**: {}\n", total_teachers));
        
        markdown.push('\n');

        // 年级分布
        if !classes.is_empty() {
            markdown.push_str("## 年级分布\n\n");
            let mut grade_counts: std::collections::HashMap<i16, i64> = std::collections::HashMap::new();
            for class in classes {
                *grade_counts.entry(class.grade).or_default() += class.student_count;
            }
            
            let mut grades: Vec<i16> = grade_counts.keys().copied().collect();
            grades.sort();

            markdown.push_str("| 年级 | 学生人数 |\n");
            markdown.push_str("|------|----------|\n");
            for grade in grades {
                markdown.push_str(&format!("| {}年级 | {} |\n", grade, grade_counts[&grade]));
            }
            markdown.push('\n');
        }

        markdown
    }
}

// ========== 数据查询服务 ==========

pub struct DataQueryService {
    pool: PgPool,
}

impl DataQueryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn execute_query(&self, req: DataQueryRequest) -> Result<String, AppError> {
        match req.query_type.as_str() {
            "class_list" => {
                let service = ClassDataService::new(self.pool.clone());
                let classes = service.get_all_classes().await?;
                Ok(MarkdownFormatter::format_class_list(&classes))
            }
            "class_detail" => {
                let class_id_str = req.id.as_deref().ok_or(AppError::InvalidInput("缺少班级ID".to_string()))?;
                
                // 首先尝试作为UUID解析
                let class_id = if let Ok(uuid) = Uuid::parse_str(class_id_str) {
                    // 检查UUID是否存在
                    let exists: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM classes WHERE id = $1)"
                    )
                    .bind(uuid)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| AppError::Database(e))?;
                    
                    if exists {
                        uuid
                    } else {
                        return Err(AppError::InvalidInput(format!("班级不存在: {}", class_id_str)));
                    }
                } else {
                    // 按名称搜索班级
                    let classes: Vec<(Uuid, String, String)> = sqlx::query_as(
                        "SELECT id, name, grade FROM classes WHERE name = $1 OR name ILIKE $1 ORDER BY name"
                    )
                    .bind(class_id_str)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| AppError::Database(e))?;
                    
                    if classes.is_empty() {
                        return Err(AppError::InvalidInput(format!("未找到名为 '{}' 的班级", class_id_str)));
                    }
                    
                    if classes.len() > 1 {
                        // 有多个同名班级，返回错误信息让AI询问用户
                        let candidates_info: Vec<String> = classes
                            .iter()
                            .map(|c| format!("{} (年级: {})", c.1, c.2))
                            .collect();
                        return Err(AppError::InvalidInput(format!(
                            "找到多个名为 '{}' 的班级: {}。请指定具体是哪一个（例如提供年级信息）",
                            class_id_str,
                            candidates_info.join(", ")
                        )));
                    }
                    
                    // 只有一个班级，使用其ID
                    classes[0].0
                };
                
                let service = ClassDataService::new(self.pool.clone());
                let detail = service.get_class_detail(class_id).await?;
                Ok(MarkdownFormatter::format_class_detail(&detail))
            }
            "group_list" => {
                let service = GroupDataService::new(self.pool.clone());
                let groups = service.get_all_groups().await?;
                Ok(MarkdownFormatter::format_group_list(&groups))
            }
            "group_detail" => {
                let group_id_str = req.id.as_deref().ok_or(AppError::InvalidInput("缺少小组ID".to_string()))?;
                
                // 首先尝试作为UUID解析
                let group_id = if let Ok(uuid) = Uuid::parse_str(group_id_str) {
                    // 检查UUID是否存在
                    let exists: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM class_groups WHERE id = $1)"
                    )
                    .bind(uuid)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| AppError::Database(e))?;
                    
                    if exists {
                        uuid
                    } else {
                        return Err(AppError::InvalidInput(format!("小组不存在: {}", group_id_str)));
                    }
                } else {
                    // 按名称搜索小组
                    let groups: Vec<(Uuid, String, String, i32)> = sqlx::query_as(
                        r#"SELECT 
                            cg.id, 
                            cg.name, 
                            c.name as class_name,
                            cg.score
                        FROM class_groups cg
                        JOIN classes c ON cg.class_id = c.id
                        WHERE cg.name = $1 OR cg.name ILIKE $1
                        ORDER BY cg.name"#,
                    )
                    .bind(&group_id_str)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| AppError::Database(e))?;
                    
                    if groups.is_empty() {
                        return Err(AppError::InvalidInput(format!("未找到名为 '{}' 的小组", group_id_str)));
                    }
                    
                    if groups.len() > 1 {
                        // 有多个同名小组，返回错误信息让AI询问用户
                        let candidates_info: Vec<String> = groups
                            .iter()
                            .map(|g| format!("{} (班级: {}, 积分: {})", g.1, g.2, g.3))
                            .collect();
                        return Err(AppError::InvalidInput(format!(
                            "找到多个名为 '{}' 的小组: {}。请指定具体是哪一个（例如提供班级信息）",
                            group_id_str,
                            candidates_info.join(", ")
                        )));
                    }
                    
                    // 只有一个小组，使用其ID
                    groups[0].0
                };
                
                let service = GroupDataService::new(self.pool.clone());
                let detail = service.get_group_detail(group_id).await?;
                Ok(MarkdownFormatter::format_group_detail(&detail))
            }
            "department_list" => {
                let service = DepartmentDataService::new(self.pool.clone());
                let departments = service.get_all_departments().await?;
                Ok(MarkdownFormatter::format_department_list(&departments))
            }
            "department_detail" => {
                let dept_id_str = req.id.as_deref().ok_or(AppError::InvalidInput("缺少部门ID".to_string()))?;
                
                // 首先尝试作为UUID解析
                let dept_id = if let Ok(uuid) = Uuid::parse_str(dept_id_str) {
                    // 检查UUID是否存在
                    let exists: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM departments WHERE id = $1)"
                    )
                    .bind(uuid)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| AppError::Database(e))?;
                    
                    if exists {
                        uuid
                    } else {
                        return Err(AppError::InvalidInput(format!("部门不存在: {}", dept_id_str)));
                    }
                } else {
                    // 按名称搜索部门
                    let departments: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
                        "SELECT id, name, description FROM departments WHERE name = $1 OR name ILIKE $1 ORDER BY name"
                    )
                    .bind(dept_id_str)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| AppError::Database(e))?;
                    
                    if departments.is_empty() {
                        return Err(AppError::InvalidInput(format!("未找到名为 '{}' 的部门", dept_id_str)));
                    }
                    
                    if departments.len() > 1 {
                        // 有多个同名部门，返回错误信息让AI询问用户
                        let candidates_info: Vec<String> = departments
                            .iter()
                            .map(|d| format!("{} ({})", d.1, d.2.as_deref().unwrap_or("无描述")))
                            .collect();
                        return Err(AppError::InvalidInput(format!(
                            "找到多个名为 '{}' 的部门: {}。请指定具体是哪一个",
                            dept_id_str,
                            candidates_info.join(", ")
                        )));
                    }
                    
                    // 只有一个部门，使用其ID
                    departments[0].0
                };
                
                let service = DepartmentDataService::new(self.pool.clone());
                let detail = service.get_department_detail(dept_id).await?;
                Ok(MarkdownFormatter::format_department_detail(&detail))
            }
            "overview" => {
                let class_service = ClassDataService::new(self.pool.clone());
                let group_service = GroupDataService::new(self.pool.clone());
                let dept_service = DepartmentDataService::new(self.pool.clone());
                
                let classes = class_service.get_all_classes().await.unwrap_or_default();
                let groups = group_service.get_all_groups().await.unwrap_or_default();
                let departments = dept_service.get_all_departments().await.unwrap_or_default();
                
                Ok(MarkdownFormatter::format_overview(&classes, &groups, &departments))
            }
            _ => Err(AppError::InvalidInput(format!("未知的查询类型: {}", req.query_type))),
        }
    }
}

// ========== API处理函数 ==========

/// 数据查询API - 供AI调用
pub async fn query_data(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<DataQueryRequest>,
) -> Result<Json<DataQueryResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 获取用户权限
    let permission_manager = PermissionManager::new(pool.clone());
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;

    // 根据查询类型执行相应操作
    let result = match req.query_type.as_str() {
        "class_list" => {
            // 检查权限
            if !user_permissions.iter().any(|p| p == "class.view" || p == "class.*") {
                return Err(AppError::Auth("没有查看班级的权限".to_string()));
            }
            
            let service = ClassDataService::new(pool.clone());
            let classes = service.get_all_classes().await?;
            
            if req.format_as_markdown {
                MarkdownFormatter::format_class_list(&classes)
            } else {
                serde_json::to_string(&classes).unwrap_or_default()
            }
        }
        "class_detail" => {
            let class_id_str = req.id.as_deref().ok_or(AppError::InvalidInput("缺少班级ID".to_string()))?;
            
            if !user_permissions.iter().any(|p| p == "class.view" || p == "class.*") {
                return Err(AppError::Auth("没有查看班级的权限".to_string()));
            }
            
            // 首先尝试作为UUID解析
            let class_id = if let Ok(uuid) = Uuid::parse_str(class_id_str) {
                // 检查UUID是否存在
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM classes WHERE id = $1)"
                )
                .bind(uuid)
                .fetch_one(&pool)
                .await
                .map_err(|e| AppError::Database(e))?;
                
                if exists {
                    uuid
                } else {
                    return Err(AppError::InvalidInput(format!("班级不存在: {}", class_id_str)));
                }
            } else {
                // 按名称搜索班级
                let classes: Vec<(Uuid, String, String)> = sqlx::query_as(
                    "SELECT id, name, grade FROM classes WHERE name = $1 OR name ILIKE $1 ORDER BY name"
                )
                .bind(class_id_str)
                .fetch_all(&pool)
                .await
                .map_err(|e| AppError::Database(e))?;
                
                if classes.is_empty() {
                    return Err(AppError::InvalidInput(format!("未找到名为 '{}' 的班级", class_id_str)));
                }
                
                if classes.len() > 1 {
                    // 有多个同名班级，返回错误信息让AI询问用户
                    let candidates_info: Vec<String> = classes
                        .iter()
                        .map(|c| format!("{} (年级: {})", c.1, c.2))
                        .collect();
                    return Err(AppError::InvalidInput(format!(
                        "找到多个名为 '{}' 的班级: {}。请指定具体是哪一个（例如提供年级信息）",
                        class_id_str,
                        candidates_info.join(", ")
                    )));
                }
                
                // 只有一个班级，使用其ID
                classes[0].0
            };
            
            let service = ClassDataService::new(pool.clone());
            let detail = service.get_class_detail(class_id).await?;
            
            if req.format_as_markdown {
                MarkdownFormatter::format_class_detail(&detail)
            } else {
                serde_json::to_string(&detail).unwrap_or_default()
            }
        }
        "group_list" => {
            if !user_permissions.iter().any(|p| p == "group.view" || p == "group.*") {
                return Err(AppError::Auth("没有查看小组的权限".to_string()));
            }
            
            let service = GroupDataService::new(pool.clone());
            let groups = service.get_all_groups().await?;
            
            if req.format_as_markdown {
                MarkdownFormatter::format_group_list(&groups)
            } else {
                serde_json::to_string(&groups).unwrap_or_default()
            }
        }
        "group_detail" => {
            let group_id_str = req.id.as_deref().ok_or(AppError::InvalidInput("缺少小组ID".to_string()))?;
            
            if !user_permissions.iter().any(|p| p == "group.view" || p == "group.*") {
                return Err(AppError::Auth("没有查看小组的权限".to_string()));
            }
            
            // 首先尝试作为UUID解析
            let group_id = if let Ok(uuid) = Uuid::parse_str(group_id_str) {
                // 检查UUID是否存在
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM class_groups WHERE id = $1)"
                )
                .bind(uuid)
                .fetch_one(&pool)
                .await
                .map_err(|e| AppError::Database(e))?;
                
                if exists {
                    uuid
                } else {
                    return Err(AppError::InvalidInput(format!("小组不存在: {}", group_id_str)));
                }
            } else {
                // 按名称搜索小组
                let groups: Vec<(Uuid, String, String, i32)> = sqlx::query_as(
                    r#"SELECT 
                        cg.id, 
                        cg.name, 
                        c.name as class_name,
                        cg.score
                    FROM class_groups cg
                    JOIN classes c ON cg.class_id = c.id
                    WHERE cg.name = $1 OR cg.name ILIKE $1
                    ORDER BY cg.name"#,
                )
                .bind(group_id_str)
                .fetch_all(&pool)
                .await
                .map_err(|e| AppError::Database(e))?;
                
                if groups.is_empty() {
                    return Err(AppError::InvalidInput(format!("未找到名为 '{}' 的小组", group_id_str)));
                }
                
                if groups.len() > 1 {
                    // 有多个同名小组，返回错误信息让AI询问用户
                    let candidates_info: Vec<String> = groups
                        .iter()
                        .map(|g| format!("{} (班级: {}, 积分: {})", g.1, g.2, g.3))
                        .collect();
                    return Err(AppError::InvalidInput(format!(
                        "找到多个名为 '{}' 的小组: {}。请指定具体是哪一个（例如提供班级信息）",
                        group_id_str,
                        candidates_info.join(", ")
                    )));
                }
                
                // 只有一个小组，使用其ID
                groups[0].0
            };
            
            let service = GroupDataService::new(pool.clone());
            let detail = service.get_group_detail(group_id).await?;
            
            if req.format_as_markdown {
                MarkdownFormatter::format_group_detail(&detail)
            } else {
                serde_json::to_string(&detail).unwrap_or_default()
            }
        }
        "department_list" => {
            if !user_permissions.iter().any(|p| p == "department.view" || p == "department.*") {
                return Err(AppError::Auth("没有查看部门的权限".to_string()));
            }
            
            let service = DepartmentDataService::new(pool.clone());
            let departments = service.get_all_departments().await?;
            
            if req.format_as_markdown {
                MarkdownFormatter::format_department_list(&departments)
            } else {
                serde_json::to_string(&departments).unwrap_or_default()
            }
        }
        "department_detail" => {
            let dept_id_str = req.id.as_deref().ok_or(AppError::InvalidInput("缺少部门ID".to_string()))?;
            
            if !user_permissions.iter().any(|p| p == "department.view" || p == "department.*") {
                return Err(AppError::Auth("没有查看部门的权限".to_string()));
            }
            
            // 首先尝试作为UUID解析
            let dept_id = if let Ok(uuid) = Uuid::parse_str(dept_id_str) {
                // 检查UUID是否存在
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM departments WHERE id = $1)"
                )
                .bind(uuid)
                .fetch_one(&pool)
                .await
                .map_err(|e| AppError::Database(e))?;
                
                if exists {
                    uuid
                } else {
                    return Err(AppError::InvalidInput(format!("部门不存在: {}", dept_id_str)));
                }
            } else {
                // 按名称搜索部门
                let departments: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
                    "SELECT id, name, description FROM departments WHERE name = $1 OR name ILIKE $1 ORDER BY name"
                )
                .bind(dept_id_str)
                .fetch_all(&pool)
                .await
                .map_err(|e| AppError::Database(e))?;
                
                if departments.is_empty() {
                    return Err(AppError::InvalidInput(format!("未找到名为 '{}' 的部门", dept_id_str)));
                }
                
                if departments.len() > 1 {
                    // 有多个同名部门，返回错误信息让AI询问用户
                    let candidates_info: Vec<String> = departments
                        .iter()
                        .map(|d| format!("{} ({})", d.1, d.2.as_deref().unwrap_or("无描述")))
                        .collect();
                    return Err(AppError::InvalidInput(format!(
                        "找到多个名为 '{}' 的部门: {}。请指定具体是哪一个",
                        dept_id_str,
                        candidates_info.join(", ")
                    )));
                }
                
                // 只有一个部门，使用其ID
                departments[0].0
            };
            
            let service = DepartmentDataService::new(pool.clone());
            let detail = service.get_department_detail(dept_id).await?;
            
            if req.format_as_markdown {
                MarkdownFormatter::format_department_detail(&detail)
            } else {
                serde_json::to_string(&detail).unwrap_or_default()
            }
        }
        "overview" => {
            // 概览需要多个权限
            let class_service = ClassDataService::new(pool.clone());
            let group_service = GroupDataService::new(pool.clone());
            let dept_service = DepartmentDataService::new(pool.clone());
            
            let classes = if user_permissions.iter().any(|p| p == "class.view" || p == "class.*") {
                class_service.get_all_classes().await.unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let groups = if user_permissions.iter().any(|p| p == "group.view" || p == "group.*") {
                group_service.get_all_groups().await.unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let departments = if user_permissions.iter().any(|p| p == "department.view" || p == "department.*") {
                dept_service.get_all_departments().await.unwrap_or_default()
            } else {
                Vec::new()
            };
            
            if req.format_as_markdown {
                MarkdownFormatter::format_overview(&classes, &groups, &departments)
            } else {
                serde_json::to_string(&serde_json::json!({
                    "classes": classes,
                    "groups": groups,
                    "departments": departments,
                })).unwrap_or_default()
            }
        }
        _ => {
            return Err(AppError::InvalidInput(format!("未知的查询类型: {}", req.query_type)));
        }
    };

    Ok(Json(DataQueryResponse {
        data: result,
        data_type: if req.format_as_markdown { "markdown".to_string() } else { "json".to_string() },
        user_permissions,
    }))
}
