use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub gender: i16, // 0:未知 1:男 2:女
    pub birthday: Option<NaiveDate>,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[sqlx(rename = "type")]
    pub type_: String, // 'student', 'teacher', 'parent'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PersonCreate {
    pub name: String,
    pub gender: i32,
    pub birthday: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub type_: String,
    // 子类型特定字段，根据type_决定哪些字段有效
    pub student_no: Option<String>,
    pub class_id: Option<Uuid>,
    pub enrollment_date: Option<String>,
    pub employee_no: Option<String>,
    pub department_id: Option<Uuid>,
    pub title: Option<String>,
    pub hire_date: Option<String>,
    pub wechat_openid: Option<String>,
    pub occupation: Option<String>,
    // 老师关联的多个班级
    pub classes: Option<Vec<TeacherClassCreate>>,
}

// 老师班级关联创建结构
#[derive(Debug, Deserialize, Serialize)]
pub struct TeacherClassCreate {
    pub class_id: Uuid,
    pub is_main_teacher: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PersonUpdate {
    pub name: Option<String>,
    pub gender: Option<i32>,
    pub birthday: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    // 子类型特定字段
    pub student_no: Option<String>,
    pub class_id: Option<Uuid>,
    pub enrollment_date: Option<String>,
    pub employee_no: Option<String>,
    pub department_id: Option<Uuid>,
    pub title: Option<String>,
    pub hire_date: Option<String>,
    pub wechat_openid: Option<String>,
    pub occupation: Option<String>,
    // 老师关联的多个班级
    pub classes: Option<Vec<TeacherClassCreate>>,
}

// 子类型结构体（用于查询连接结果）
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Student {
    pub person_id: Uuid,
    pub student_no: String,
    pub class_id: Option<Uuid>,
    pub enrollment_date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Teacher {
    pub person_id: Uuid,
    pub employee_no: String,
    pub department_id: Option<Uuid>,
    pub title: Option<String>,
    pub hire_date: Option<NaiveDate>,
}

// 老师与班级的关联关系
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct TeacherClass {
    pub teacher_id: Uuid,
    pub class_id: Uuid,
    pub is_main_teacher: bool,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Parent {
    pub person_id: Uuid,
    pub wechat_openid: Option<String>,
    pub occupation: Option<String>,
}

// 统一响应枚举
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PersonResponse {
    #[serde(rename = "student")]
    Student(StudentResponse),
    #[serde(rename = "teacher")]
    Teacher(TeacherResponse),
    #[serde(rename = "parent")]
    Parent(ParentResponse),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StudentResponse {
    pub id: Uuid,
    pub name: String,
    pub gender: i16,
    pub birthday: Option<NaiveDate>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub student_no: String,
    pub class_id: Option<Uuid>,
    pub class_name: Option<String>,
    pub enrollment_date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeacherClassInfo {
    pub class_id: Uuid,
    pub class_name: String,
    pub is_main_teacher: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeacherResponse {
    pub id: Uuid,
    pub name: String,
    pub gender: i16,
    pub birthday: Option<NaiveDate>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub employee_no: String,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub classes: Vec<TeacherClassInfo>,
    pub title: Option<String>,
    pub hire_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParentResponse {
    pub id: Uuid,
    pub name: String,
    pub gender: i16,
    pub birthday: Option<NaiveDate>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub wechat_openid: Option<String>,
    pub occupation: Option<String>,
}

// 转换实现
impl From<(Person, Option<Student>)> for PersonResponse {
    fn from((person, student): (Person, Option<Student>)) -> Self {
        match person.type_.as_str() {
            "student" if student.is_some() => {
                let student = student.unwrap();
                PersonResponse::Student(StudentResponse {
                    id: person.id,
                    name: person.name,
                    gender: person.gender,
                    birthday: person.birthday,
                    phone: person.phone,
                    email: person.email,
                    student_no: student.student_no,
                    class_id: student.class_id,
                    class_name: None, // 需要额外查询
                    enrollment_date: student.enrollment_date,
                    status: student.status
                })
            }
            "teacher" => {
                // 需要教师信息，暂时返回基础信息
                PersonResponse::Teacher(TeacherResponse {
                    id: person.id,
                    name: person.name,
                    gender: person.gender,
                    birthday: person.birthday,
                    phone: person.phone,
                    email: person.email,
                    employee_no: String::new(),
                    department_id: None,
                    department_name: None,
                    classes: Vec::new(),
                    title: None,
                    hire_date: None
                })
            }
            "parent" => {
                PersonResponse::Parent(ParentResponse {
                    id: person.id,
                    name: person.name,
                    gender: person.gender,
                    birthday: person.birthday,
                    phone: person.phone,
                    email: person.email,
                    wechat_openid: None,
                    occupation: None
                })
            }
            _ => {
                // 默认作为基础人员
                PersonResponse::Teacher(TeacherResponse {
                    id: person.id,
                    name: person.name,
                    gender: person.gender,
                    birthday: person.birthday,
                    phone: person.phone,
                    email: person.email,
                    employee_no: String::new(),
                    department_id: None,
                    department_name: None,
                    classes: Vec::new(),
                    title: None,
                    hire_date: None
                })
            }
        }
    }
}
