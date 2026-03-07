use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Utc};

/// 格式化日期为YYYY-MM-DD字符串
#[allow(dead_code)]
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// 格式化为时间HH:MM:SS字符串
#[allow(dead_code)]
pub fn format_time(time: &NaiveTime) -> String {
    time.format("%H:%M:%S").to_string()
}

/// 格式化为日期时间为RFC3339字符串
#[allow(dead_code)]
pub fn format_datetime(datetime: &NaiveDateTime) -> String {
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// 解析YYYY-MM-DD日期字符串
#[allow(dead_code)]
pub fn parse_date(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("日期格式错误，应为YYYY-MM-DD: {}", e))
}

/// 解析HH:MM:SS时间字符串
#[allow(dead_code)]
pub fn parse_time(time_str: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(time_str, "%H:%M:%S")
        .map_err(|e| format!("时间格式错误，应为HH:MM:SS: {}", e))
}

/// 解析日期时间字符串（支持多种格式）
#[allow(dead_code)]
pub fn parse_datetime(datetime_str: &str) -> Result<NaiveDateTime, String> {
    // 尝试RFC3339格式
    if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%SZ") {
        return Ok(dt);
    }
    // 尝试YYYY-MM-DD HH:MM:SS格式
    if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    Err("日期时间格式错误".to_string())
}

/// 获取当前日期字符串
#[allow(dead_code)]
pub fn current_date_string() -> String {
    Utc::now().date_naive().format("%Y-%m-%d").to_string()
}

/// 获取当前时间字符串
#[allow(dead_code)]
pub fn current_time_string() -> String {
    Utc::now().time().format("%H:%M:%S").to_string()
}

/// 获取当前日期时间字符串（RFC3339格式）
#[allow(dead_code)]
pub fn current_datetime_string() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
