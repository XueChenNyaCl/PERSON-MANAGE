use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::Value;
use uuid::Uuid;

use crate::core::error::AppError;

/// 字段验证规则
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FieldRule {
    pub field_name: &'static str,
    pub required: bool,
    pub field_type: FieldType,
    pub max_length: Option<usize>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub allowed_values: Option<Vec<&'static str>>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FieldType {
    String,
    Integer,
    Boolean,
    Date,
    Time,
    DateTime,
    Uuid,
}

/// 验证规则集合
#[allow(dead_code)]
pub struct ValidationRules;

#[allow(dead_code)]
impl ValidationRules {
    /// create_person操作的验证规则
    pub fn create_person_rules() -> Vec<FieldRule> {
        vec![
            FieldRule {
                field_name: "name",
                required: true,
                field_type: FieldType::String,
                max_length: Some(100),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "gender",
                required: true,
                field_type: FieldType::Integer,
                max_length: None,
                min_value: Some(0),
                max_value: Some(2),
                allowed_values: None,
            },
            FieldRule {
                field_name: "type",
                required: true,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: Some(vec!["student", "teacher", "parent"]),
            },
            FieldRule {
                field_name: "birthday",
                required: false,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "phone",
                required: false,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "email",
                required: false,
                field_type: FieldType::String,
                max_length: Some(100),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "student_no",
                required: false,
                field_type: FieldType::String,
                max_length: Some(50),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "class_id",
                required: false,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "enrollment_date",
                required: false,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "employee_no",
                required: false,
                field_type: FieldType::String,
                max_length: Some(50),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "department_id",
                required: false,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "title",
                required: false,
                field_type: FieldType::String,
                max_length: Some(50),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "hire_date",
                required: false,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
        ]
    }

    /// create_attendance操作的验证规则
    pub fn create_attendance_rules() -> Vec<FieldRule> {
        vec![
            FieldRule {
                field_name: "person_id",
                required: true,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "date",
                required: true,
                field_type: FieldType::Date,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "status",
                required: true,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: Some(vec!["present", "absent", "late", "early_leave", "excused"]),
            },
            FieldRule {
                field_name: "time",
                required: false,
                field_type: FieldType::Time,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "remark",
                required: false,
                field_type: FieldType::String,
                max_length: Some(500),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
        ]
    }

    /// create_score操作的验证规则
    pub fn create_score_rules() -> Vec<FieldRule> {
        vec![
            FieldRule {
                field_name: "person_id",
                required: true,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "score_type",
                required: true,
                field_type: FieldType::String,
                max_length: Some(20),
                min_value: None,
                max_value: None,
                allowed_values: Some(vec!["personal", "group", "class", "dormitory"]),
            },
            FieldRule {
                field_name: "value",
                required: true,
                field_type: FieldType::Integer,
                max_length: None,
                min_value: Some(0),
                max_value: Some(100),
                allowed_values: None,
            },
            FieldRule {
                field_name: "reason",
                required: true,
                field_type: FieldType::String,
                max_length: Some(500),
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
            FieldRule {
                field_name: "group_id",
                required: false,
                field_type: FieldType::Uuid,
                max_length: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
            },
        ]
    }
}

/// AI操作参数验证器
#[allow(dead_code)]
pub struct AIActionValidator;

#[allow(dead_code)]
impl AIActionValidator {
    /// 通用字段验证函数
    pub fn validate_field(value: &Value, rule: &FieldRule) -> Result<(), AppError> {
        let field_value = value.get(rule.field_name);

        // 检查必填字段
        if rule.required && field_value.is_none() {
            return Err(AppError::InvalidInput(format!(
                "缺少必填字段: {}",
                rule.field_name
            )));
        }

        let Some(field_value) = field_value else {
            return Ok(());
        };

        // 如果字段是null或空字符串，也跳过（非必填）
        if field_value.is_null()
            || (field_value.is_string() && field_value.as_str().unwrap_or("").is_empty())
        {
            if rule.required {
                return Err(AppError::InvalidInput(format!(
                    "必填字段不能为空: {}",
                    rule.field_name
                )));
            }
            return Ok(());
        }

        // 根据字段类型进行验证
        match rule.field_type {
            FieldType::String => {
                let s = field_value.as_str().ok_or_else(|| {
                    AppError::InvalidInput(format!("字段 {} 必须是字符串类型", rule.field_name))
                })?;

                if let Some(max_len) = rule.max_length {
                    if s.len() > max_len {
                        return Err(AppError::InvalidInput(format!(
                            "字段 {} 长度不能超过 {} 个字符",
                            rule.field_name, max_len
                        )));
                    }
                }

                if let Some(allowed) = &rule.allowed_values {
                    if !allowed.contains(&s) {
                        return Err(AppError::InvalidInput(format!(
                            "字段 {} 的值无效，允许的值: {:?}",
                            rule.field_name, allowed
                        )));
                    }
                }
            }

            FieldType::Integer => {
                let n = field_value.as_i64().ok_or_else(|| {
                    AppError::InvalidInput(format!("字段 {} 必须是整数类型", rule.field_name))
                })?;

                if let Some(min) = rule.min_value {
                    if n < min {
                        return Err(AppError::InvalidInput(format!(
                            "字段 {} 的值不能小于 {}",
                            rule.field_name, min
                        )));
                    }
                }

                if let Some(max) = rule.max_value {
                    if n > max {
                        return Err(AppError::InvalidInput(format!(
                            "字段 {} 的值不能大于 {}",
                            rule.field_name, max
                        )));
                    }
                }
            }

            FieldType::Boolean => {
                if !field_value.is_boolean() {
                    return Err(AppError::InvalidInput(format!(
                        "字段 {} 必须是布尔类型",
                        rule.field_name
                    )));
                }
            }

            FieldType::Date => {
                let s = field_value.as_str().ok_or_else(|| {
                    AppError::InvalidInput(format!("字段 {} 必须是字符串类型", rule.field_name))
                })?;

                NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
                    AppError::InvalidInput(format!(
                        "字段 {} 日期格式无效，应为YYYY-MM-DD: {}",
                        rule.field_name, s
                    ))
                })?;
            }

            FieldType::Time => {
                let s = field_value.as_str().ok_or_else(|| {
                    AppError::InvalidInput(format!("字段 {} 必须是字符串类型", rule.field_name))
                })?;

                NaiveTime::parse_from_str(s, "%H:%M:%S").map_err(|_| {
                    AppError::InvalidInput(format!(
                        "字段 {} 时间格式无效，应为HH:MM:SS: {}",
                        rule.field_name, s
                    ))
                })?;
            }

            FieldType::DateTime => {
                let s = field_value.as_str().ok_or_else(|| {
                    AppError::InvalidInput(format!("字段 {} 必须是字符串类型", rule.field_name))
                })?;

                let _ = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                    .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                    .map_err(|_| {
                        AppError::InvalidInput(format!(
                            "字段 {} 日期时间格式无效: {}",
                            rule.field_name, s
                        ))
                    })?;
            }

            FieldType::Uuid => {
                let s = field_value.as_str().ok_or_else(|| {
                    AppError::InvalidInput(format!("字段 {} 必须是字符串类型", rule.field_name))
                })?;

                Uuid::parse_str(s).map_err(|_| {
                    AppError::InvalidInput(format!("字段 {} UUID格式无效: {}", rule.field_name, s))
                })?;
            }
        }

        Ok(())
    }

    /// 验证创建人员参数
    pub fn validate_create_person(params: &Value) -> Result<(), AppError> {
        let rules = ValidationRules::create_person_rules();

        for rule in &rules {
            Self::validate_field(params, rule)?;
        }

        // 条件必填验证
        let type_ = params
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("缺少type字段".to_string()))?;

        match type_ {
            "student" => {
                let student_no = params
                    .get("student_no")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty());

                if student_no.is_none() {
                    return Err(AppError::InvalidInput(
                        "学生缺少必填字段: student_no".to_string(),
                    ));
                }
            }
            "teacher" => {
                let employee_no = params
                    .get("employee_no")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty());

                if employee_no.is_none() {
                    return Err(AppError::InvalidInput(
                        "教师缺少必填字段: employee_no".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 验证创建考勤参数
    pub fn validate_create_attendance(params: &Value) -> Result<(), AppError> {
        let rules = ValidationRules::create_attendance_rules();

        for rule in &rules {
            Self::validate_field(params, rule)?;
        }

        Ok(())
    }

    /// 验证创建成绩参数
    pub fn validate_create_score(params: &Value) -> Result<(), AppError> {
        let rules = ValidationRules::create_score_rules();

        for rule in &rules {
            Self::validate_field(params, rule)?;
        }

        Ok(())
    }
}
