# AI Markdown 输出适配开发文档

## 1. 项目概述

### 1.1 目标
设计一个后端程序，专门用于调用数据库中的对应信息，并将信息格式化输出。目的是让AI能够正常调用这些语句来获取数据库的信息，并以Markdown格式输出，便于用户阅读和理解。

### 1.2 核心功能
1. **数据库信息获取**: 根据用户权限获取班级、小组、部门、人员等信息
2. **信息格式化**: 将获取的数据格式化为结构化的Markdown文档
3. **AI调用接口**: 提供标准化的调用语句，让AI能够正确调用数据库
4. **权限控制**: 确保AI只能获取用户权限范围内的信息

### 1.3 技术栈
- **后端**: Rust + Axum + SQLx + PostgreSQL
- **前端**: Vue 3 + TypeScript
- **AI集成**: DeepSeek API / OpenAI API

---

## 2. 数据库信息获取模块设计

### 2.1 数据实体定义

#### 2.1.1 班级信息 (ClassInfo)
```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClassInfo {
    pub id: Uuid,
    pub name: String,
    pub grade: i32,
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
```

#### 2.1.2 小组信息 (GroupInfo)
```rust
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
```

#### 2.1.3 部门信息 (DepartmentInfo)
```rust
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
```

#### 2.1.4 人员信息 (PersonInfo)
```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PersonInfo {
    pub id: Uuid,
    pub name: String,
    pub type_: String,  // student, teacher, parent
    pub email: Option<String>,
    pub phone: Option<String>,
    pub student_no: Option<String>,
    pub employee_no: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentInfo {
    pub id: Uuid,
    pub name: String,
    pub student_no: String,
    pub class_id: Option<Uuid>,
    pub class_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeacherInfo {
    pub id: Uuid,
    pub name: String,
    pub employee_no: String,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}
```

### 2.2 数据获取服务层

#### 2.2.1 班级数据服务 (ClassDataService)
```rust
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
                COUNT(DISTINCT pc.person_id) as student_count,
                c.created_at
            FROM classes c
            LEFT JOIN persons p ON c.teacher_id = p.id
            LEFT JOIN person_class pc ON c.id = pc.class_id AND pc.role = 'student'
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
                COUNT(DISTINCT pc.person_id) as student_count,
                c.created_at
            FROM classes c
            LEFT JOIN persons p ON c.teacher_id = p.id
            LEFT JOIN person_class pc ON c.id = pc.class_id AND pc.role = 'student'
            WHERE c.id = $1
            GROUP BY c.id, c.name, c.grade, c.teacher_id, p.name, c.created_at
            "#,
            class_id
        )
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
            JOIN person_class pc ON p.id = pc.person_id AND pc.role = 'student'
            JOIN classes c ON pc.class_id = c.id
            WHERE c.id = $1
            ORDER BY s.student_no
            "#,
            class_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

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
            JOIN person_class pc ON p.id = pc.person_id AND pc.role = 'teacher'
            LEFT JOIN departments d ON p.department_id = d.id
            WHERE pc.class_id = $1
            ORDER BY p.name
            "#,
            class_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // 获取小组列表
        let groups = sqlx::query_as::<_, GroupInfo>(
            r#"
            SELECT 
                g.id,
                g.name,
                g.class_id,
                c.name as class_name,
                COUNT(DISTINCT gm.person_id) as member_count,
                COALESCE(SUM(gs.score), 0) as total_score
            FROM groups g
            JOIN classes c ON g.class_id = c.id
            LEFT JOIN group_members gm ON g.id = gm.group_id
            LEFT JOIN group_score_records gs ON g.id = gs.group_id
            WHERE g.class_id = $1
            GROUP BY g.id, g.name, g.class_id, c.name
            ORDER BY g.name
            "#,
            class_id
        )
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
```

#### 2.2.2 小组数据服务 (GroupDataService)
```rust
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
                g.id,
                g.name,
                g.class_id,
                c.name as class_name,
                COUNT(DISTINCT gm.person_id) as member_count,
                COALESCE(SUM(gs.score), 0) as total_score
            FROM groups g
            JOIN classes c ON g.class_id = c.id
            LEFT JOIN group_members gm ON g.id = gm.group_id
            LEFT JOIN group_score_records gs ON g.id = gs.group_id
            GROUP BY g.id, g.name, g.class_id, c.name
            ORDER BY c.name, g.name
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
                g.id,
                g.name,
                g.class_id,
                c.name as class_name,
                COUNT(DISTINCT gm.person_id) as member_count,
                COALESCE(SUM(gs.score), 0) as total_score
            FROM groups g
            JOIN classes c ON g.class_id = c.id
            LEFT JOIN group_members gm ON g.id = gm.group_id
            LEFT JOIN group_score_records gs ON g.id = gs.group_id
            WHERE g.id = $1
            GROUP BY g.id, g.name, g.class_id, c.name
            "#,
            group_id
        )
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
                gm.role
            FROM group_members gm
            JOIN persons p ON gm.person_id = p.id
            LEFT JOIN students s ON p.id = s.person_id
            WHERE gm.group_id = $1
            ORDER BY gm.role, p.name
            "#,
            group_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // 获取积分记录
        let score_records = sqlx::query_as::<_, ScoreRecord>(
            r#"
            SELECT 
                gs.id,
                gs.score,
                gs.reason,
                gs.created_at,
                p.name as operator_name
            FROM group_score_records gs
            LEFT JOIN persons p ON gs.operator_id = p.id
            WHERE gs.group_id = $1
            ORDER BY gs.created_at DESC
            "#,
            group_id
        )
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
```

#### 2.2.3 部门数据服务 (DepartmentDataService)
```rust
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
                d.description,
                COUNT(DISTINCT p.id) as teacher_count,
                d.created_at
            FROM departments d
            LEFT JOIN persons p ON d.id = p.department_id AND p.type = 'teacher'
            GROUP BY d.id, d.name, d.description, d.created_at
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
                d.description,
                COUNT(DISTINCT p.id) as teacher_count,
                d.created_at
            FROM departments d
            LEFT JOIN persons p ON d.id = p.department_id AND p.type = 'teacher'
            WHERE d.id = $1
            GROUP BY d.id, d.name, d.description, d.created_at
            "#,
            dept_id
        )
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
            LEFT JOIN departments d ON p.department_id = d.id
            WHERE p.department_id = $1
            ORDER BY p.name
            "#,
            dept_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(DepartmentDetailInfo {
            basic_info,
            teachers,
        })
    }
}
```

---

## 3. Markdown格式化模块设计

### 3.1 Markdown格式化器 (MarkdownFormatter)

```rust
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
        let mut grade_groups: std::collections::HashMap<i32, Vec<&ClassInfo>> = std::collections::HashMap::new();
        for class in classes {
            grade_groups.entry(class.grade).or_default().push(class);
        }

        let mut grades: Vec<i32> = grade_groups.keys().copied().collect();
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

        markdown.push_str("| 部门名称 | 描述 | 教师人数 |\n");
        markdown.push_str("|---------|------|----------|\n");

        for dept in departments {
            markdown.push_str(&format!(
                "| {} | {} | {} |\n",
                dept.name,
                dept.description.as_deref().unwrap_or("-"),
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
        markdown.push_str(&format!(
            "- **描述**: {}\n",
            dept.description.as_deref().unwrap_or("暂无描述")
        ));
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
            let mut grade_counts: std::collections::HashMap<i32, i64> = std::collections::HashMap::new();
            for class in classes {
                *grade_counts.entry(class.grade).or_default() += class.student_count;
            }
            
            let mut grades: Vec<i32> = grade_counts.keys().copied().collect();
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
```

---

## 4. AI调用接口设计

### 4.1 数据查询API

#### 4.1.1 查询请求结构
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DataQueryRequest {
    /// 查询类型: class_list, class_detail, group_list, group_detail, 
    ///          department_list, department_detail, overview
    pub query_type: String,
    /// 可选的ID参数（用于详情查询）
    pub id: Option<Uuid>,
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
```

#### 4.1.2 数据查询API实现
```rust
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
            let class_id = req.id.ok_or(AppError::BadRequest("缺少班级ID".to_string()))?;
            
            if !user_permissions.iter().any(|p| p == "class.view" || p == "class.*") {
                return Err(AppError::Auth("没有查看班级的权限".to_string()));
            }
            
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
            let group_id = req.id.ok_or(AppError::BadRequest("缺少小组ID".to_string()))?;
            
            if !user_permissions.iter().any(|p| p == "group.view" || p == "group.*") {
                return Err(AppError::Auth("没有查看小组的权限".to_string()));
            }
            
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
            let dept_id = req.id.ok_or(AppError::BadRequest("缺少部门ID".to_string()))?;
            
            if !user_permissions.iter().any(|p| p == "department.view" || p == "department.*") {
                return Err(AppError::Auth("没有查看部门的权限".to_string()));
            }
            
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
            return Err(AppError::BadRequest(format!("未知的查询类型: {}", req.query_type)));
        }
    };

    Ok(Json(DataQueryResponse {
        data: result,
        data_type: if req.format_as_markdown { "markdown".to_string() } else { "json".to_string() },
        user_permissions,
    }))
}
```

### 4.2 AI提示词模板

#### 4.2.1 系统提示词 (System Prompt)
```rust
pub struct AIPromptTemplate;

impl AIPromptTemplate {
    /// 获取系统提示词
    pub fn get_system_prompt(user_permissions: &[String]) -> String {
        let permissions_str = user_permissions.join(", ");
        
        format!(r#"
你是一个学校管理系统的AI助手。你的任务是帮助用户查询和分析学校数据。

## 你的权限
用户拥有以下权限: {}

## 可用数据查询功能
你可以通过调用数据查询接口获取以下信息：

### 1. 班级数据
- **查询所有班级**: query_type="class_list"
- **查询班级详情**: query_type="class_detail", id="班级UUID"
- **权限要求**: class.view 或 class.*

### 2. 小组数据
- **查询所有小组**: query_type="group_list"
- **查询小组详情**: query_type="group_detail", id="小组UUID"
- **权限要求**: group.view 或 group.*

### 3. 部门数据
- **查询所有部门**: query_type="department_list"
- **查询部门详情**: query_type="department_detail", id="部门UUID"
- **权限要求**: department.view 或 department.*

### 4. 数据概览
- **查询数据概览**: query_type="overview"
- **权限要求**: 根据具体数据类型需要相应权限

## 调用格式
当需要获取数据时，请使用以下格式：
```
[DATA_QUERY]
query_type: <查询类型>
id: <可选的UUID>
format_as_markdown: true
[/DATA_QUERY]
```

## 回复格式
1. 使用Markdown格式回复
2. 数据以表格形式展示
3. 重要信息使用加粗标注
4. 适当使用列表和标题组织内容

## 注意事项
1. 你只能查询用户权限范围内的数据
2. 如果用户询问没有权限的数据，请礼貌地告知权限不足
3. 数据查询结果会自动转换为Markdown格式
4. 不要编造数据，所有数据必须通过查询接口获取
"#, permissions_str)
    }

    /// 获取数据查询提示词
    pub fn get_data_query_prompt(query_result: &str) -> String {
        format!(r#"
以下是查询到的数据（Markdown格式）：

{}

请根据以上数据回答用户的问题。如果数据为空，请说明没有找到相关数据。
"#, query_result)
    }
}
```

---

## 5. AI对话增强实现

### 5.1 增强的聊天功能

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedChatRequest {
    pub message: String,
    pub conversation_history: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,  // user, assistant, system
    pub content: String,
}

/// 增强版AI聊天 - 支持数据查询
pub async fn enhanced_chat(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<EnhancedChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 获取AI设置
    let settings = sqlx::query_as::<_, AISettings>(
        "SELECT api_key, api_base_url, model, default_prompt, temperature, max_tokens 
         FROM ai_settings 
         ORDER BY id DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    
    // 检查用户权限
    let permission_manager = PermissionManager::new(pool.clone());
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;
    
    if !user_permissions.iter().any(|p| p == "ai.chat") {
        return Err(AppError::Auth("没有 AI 聊天权限".to_string()));
    }
    
    // 检查是否需要数据查询
    let user_message = req.message.clone();
    let data_query_result = if should_query_data(&user_message) {
        // 解析查询意图
        match parse_query_intent(&user_message, &user_permissions) {
            Ok(query_req) => {
                // 执行数据查询
                let service = DataQueryService::new(pool.clone());
                match service.execute_query(query_req).await {
                    Ok(result) => Some(result),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };
    
    // 构建系统提示词
    let system_prompt = if let Some(ref data) = data_query_result {
        format!(
            "{}\n\n{}",
            AIPromptTemplate::get_system_prompt(&user_permissions),
            AIPromptTemplate::get_data_query_prompt(data)
        )
    } else {
        AIPromptTemplate::get_system_prompt(&user_permissions)
    };
    
    // 构建消息列表
    let mut messages = vec![
        AIChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }
    ];
    
    // 添加历史消息
    for msg in req.conversation_history {
        messages.push(AIChatMessage {
            role: msg.role,
            content: msg.content,
        });
    }
    
    // 添加当前用户消息
    messages.push(AIChatMessage {
        role: "user".to_string(),
        content: user_message,
    });
    
    // 调用AI API
    let api_request = AIChatRequest {
        model: settings.model,
        messages,
        temperature: settings.temperature,
        max_tokens: settings.max_tokens,
        stream: false,
    };
    
    let client = Client::new();
    let mut api_url = settings.api_base_url.clone();
    if !api_url.ends_with("/v1/chat/completions") {
        if !api_url.ends_with('/') {
            api_url.push('/');
        }
        api_url.push_str("v1/chat/completions");
    }
    
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .json(&api_request)
        .send()
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("AI API 请求失败: {}", e)))?;
    
    let api_response: AIChatResponse = response
        .json()
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("解析 AI API 响应失败: {}", e)))?;
    
    let reply = api_response.choices
        .first()
        .and_then(|choice| Some(choice.message.content.clone()))
        .unwrap_or_else(|| "AI 没有返回有效回复".to_string());
    
    Ok(Json(ChatResponse { data: reply }))
}

/// 判断是否需要查询数据
fn should_query_data(message: &str) -> bool {
    let keywords = vec![
        "班级", "小组", "部门", "学生", "教师", "人数", "统计",
        "列表", "信息", "数据", "查询", "查看", "显示",
    ];
    
    keywords.iter().any(|&kw| message.contains(kw))
}

/// 解析查询意图
fn parse_query_intent(
    message: &str,
    permissions: &[String],
) -> Result<DataQueryRequest, AppError> {
    let message = message.to_lowercase();
    
    // 班级查询
    if message.contains("班级") {
        if message.contains("所有") || message.contains("列表") || message.contains("全部") {
            return Ok(DataQueryRequest {
                query_type: "class_list".to_string(),
                id: None,
                format_as_markdown: true,
            });
        } else if message.contains("详情") || message.contains("详细") {
            // 这里需要从上下文中获取班级ID，简化处理
            return Err(AppError::BadRequest("请指定班级ID".to_string()));
        }
    }
    
    // 小组查询
    if message.contains("小组") {
        if message.contains("所有") || message.contains("列表") || message.contains("全部") {
            return Ok(DataQueryRequest {
                query_type: "group_list".to_string(),
                id: None,
                format_as_markdown: true,
            });
        }
    }
    
    // 部门查询
    if message.contains("部门") {
        if message.contains("所有") || message.contains("列表") || message.contains("全部") {
            return Ok(DataQueryRequest {
                query_type: "department_list".to_string(),
                id: None,
                format_as_markdown: true,
            });
        }
    }
    
    // 概览查询
    if message.contains("概览") || message.contains("统计") || message.contains("总览") {
        return Ok(DataQueryRequest {
            query_type: "overview".to_string(),
            id: None,
            format_as_markdown: true,
        });
    }
    
    Err(AppError::BadRequest("无法解析查询意图".to_string()))
}
```

### 5.2 数据查询服务

```rust
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
                let class_id = req.id.ok_or(AppError::BadRequest("缺少班级ID".to_string()))?;
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
                let group_id = req.id.ok_or(AppError::BadRequest("缺少小组ID".to_string()))?;
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
                let dept_id = req.id.ok_or(AppError::BadRequest("缺少部门ID".to_string()))?;
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
            _ => Err(AppError::BadRequest(format!("未知的查询类型: {}", req.query_type))),
        }
    }
}
```

---

## 6. 路由配置

### 6.1 添加路由

在 `backend/src/api/routes.rs` 中添加以下路由：

```rust
// AI 数据查询路由
.route("/api/ai/query", post(ai::query_data))
.route("/api/ai/enhanced-chat", post(ai::enhanced_chat))
```

---

## 7. 可能出现的问题及预防措施

### 7.1 数据库查询性能问题

**可能的问题**:
- 大量数据查询导致响应缓慢
- 复杂关联查询导致数据库负载过高

**预防措施**:
1. 添加查询结果缓存机制（如Redis）
2. 限制单次查询返回的数据量
3. 使用数据库索引优化查询性能
4. 添加分页支持

### 7.2 权限检查遗漏

**可能的问题**:
- 数据查询接口未正确检查用户权限
- 权限字符串拼写错误导致检查失败

**预防措施**:
1. 统一使用 `PermissionManager` 进行权限检查
2. 在文档中明确每个接口需要的权限
3. 添加权限检查的单元测试
4. 使用常量定义权限字符串，避免拼写错误

### 7.3 Markdown格式化错误

**可能的问题**:
- 特殊字符导致Markdown格式混乱
- 中文字符在表格中显示错位

**预防措施**:
1. 对特殊字符进行转义处理
2. 使用标准Markdown表格格式
3. 在表格内容中使用 `|` 时进行转义
4. 测试各种数据情况下的格式化效果

### 7.4 AI提示词过长

**可能的问题**:
- 系统提示词过长导致超出AI模型的token限制
- 数据查询结果过大导致请求失败

**预防措施**:
1. 精简系统提示词，只包含必要信息
2. 对数据查询结果进行截断或摘要
3. 监控token使用量，超过限制时进行提示
4. 使用流式响应处理大量数据

### 7.5 数据类型不匹配

**可能的问题**:
- 数据库字段类型与Rust结构体不匹配
- SQL查询返回的NULL值处理不当

**预防措施**:
1. 使用 `Option<T>` 处理可能为NULL的字段
2. 添加 `sqlx::FromRow` 派生宏确保类型匹配
3. 在SQL查询中使用 `COALESCE` 处理NULL值
4. 添加详细的错误日志便于排查问题

### 7.6 并发请求问题

**可能的问题**:
- 大量并发请求导致数据库连接池耗尽
- AI API调用频率限制导致请求失败

**预防措施**:
1. 合理配置数据库连接池大小
2. 添加请求频率限制和队列机制
3. 实现AI API调用的重试机制
4. 使用异步处理提高并发能力

---

## 8. 前端适配

### 8.1 AI对话页面增强

```typescript
// frontend/src/api/ai.ts

export interface EnhancedChatRequest {
  message: string
  conversation_history: ChatMessage[]
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system'
  content: string
}

export interface DataQueryRequest {
  query_type: string
  id?: string
  format_as_markdown: boolean
}

export interface DataQueryResponse {
  data: string
  data_type: 'json' | 'markdown'
  user_permissions: string[]
}

// 增强版聊天API
export const enhancedChat = (request: EnhancedChatRequest) => {
  return api.post<{ data: string }>('/ai/enhanced-chat', request)
}

// 数据查询API
export const queryData = (request: DataQueryRequest) => {
  return api.post<DataQueryResponse>('/ai/query', request)
}
```

### 8.2 Markdown渲染组件

```vue
<!-- frontend/src/components/MarkdownRenderer.vue -->
<template>
  <div class="markdown-content" v-html="renderedContent"></div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { marked } from 'marked'

const props = defineProps<{
  content: string
}>()

const renderedContent = computed(() => {
  return marked(props.content)
})
</script>

<style scoped>
.markdown-content {
  line-height: 1.6;
}

.markdown-content :deep(h1) {
  font-size: 1.5em;
  margin-bottom: 0.5em;
  color: #333;
}

.markdown-content :deep(h2) {
  font-size: 1.3em;
  margin: 0.8em 0 0.4em;
  color: #444;
}

.markdown-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: left;
}

.markdown-content :deep(th) {
  background-color: #f5f5f5;
  font-weight: bold;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}
</style>
```

---

## 9. 测试验证要点

### 9.1 数据查询测试
1. 测试各种查询类型的数据获取
2. 验证权限检查是否正常工作
3. 测试Markdown格式化输出
4. 验证大数据量下的性能

### 9.2 AI对话测试
1. 测试自然语言查询意图识别
2. 验证数据查询结果在AI回复中的展示
3. 测试多轮对话的上下文保持
4. 验证权限不足时的错误提示

### 9.3 边界条件测试
1. 空数据情况下的处理
2. 数据库连接失败时的降级处理
3. AI API调用失败时的错误处理
4. 超长消息的截断处理

---

## 10. 总结

本开发文档详细描述了如何设计一个后端程序来专门调用数据库信息并格式化为Markdown输出，供AI使用。主要包含以下模块：

1. **数据服务层**: 封装数据库查询逻辑，提供班级、小组、部门等数据的获取
2. **Markdown格式化器**: 将数据转换为结构化的Markdown文档
3. **AI调用接口**: 提供标准化的数据查询API，供AI调用
4. **增强对话功能**: 支持自然语言查询意图识别和自动数据获取
5. **权限控制**: 确保AI只能获取用户权限范围内的数据

通过本方案，AI助手可以：
- 理解用户的数据查询需求
- 自动调用相应的数据接口
- 以结构化的Markdown格式展示数据
- 基于数据进行智能分析和回答

同时，文档中详细列出了可能出现的问题及预防措施，确保系统的稳定性和安全性。
