use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::redis::RedisClient;
use crate::models::special_user::OperatorInfo;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
        }
    }
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub operator_type: String,
    pub operator_name: String,
    pub action: String,
    pub details: String,
}

impl LogEntry {
    pub fn to_console_string(&self) -> String {
        format!(
            "[{}][{}][{}][{}][{}]",
            self.timestamp, self.level, self.operator_name, self.action, self.details
        )
    }

    pub fn to_file_string(&self) -> String {
        format!(
            "[{}][{}][{}][{}][{}]\n",
            self.timestamp, self.level, self.operator_name, self.action, self.details
        )
    }

    pub fn to_redis_json(&self) -> String {
        format!(
            r#"{{"timestamp":"{}","level":"{}","operator_type":"{}","operator_name":"{}","action":"{}","details":"{}"}}"#,
            self.timestamp,
            self.level.as_str(),
            self.operator_type,
            self.operator_name,
            self.action,
            self.details.replace('"', "\\\"")
        )
    }
}

/// 操作日志管理器
pub struct OperationLogger {
    redis_client: Option<Arc<RedisClient>>,
    log_dir: PathBuf,
    current_file: Arc<Mutex<Option<std::fs::File>>>,
    current_date: Arc<Mutex<String>>,
}

impl OperationLogger {
    /// 创建新的操作日志管理器
    pub fn new(redis_client: Option<Arc<RedisClient>>) -> Self {
        let log_dir = PathBuf::from("./log");

        // 确保日志目录存在
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory: {}", e);
        }

        Self {
            redis_client,
            log_dir,
            current_file: Arc::new(Mutex::new(None)),
            current_date: Arc::new(Mutex::new(String::new())),
        }
    }

    /// 获取当前日志文件路径
    fn get_log_file_path(&self, date: &str) -> PathBuf {
        self.log_dir.join(format!("operation-{}.log", date))
    }

    /// 获取或创建当前日志文件
    async fn get_current_file(&self) -> Option<std::fs::File> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let mut current_date = self.current_date.lock().await;
        let mut current_file = self.current_file.lock().await;

        // 如果日期变化或文件未打开，创建新文件
        if *current_date != today || current_file.is_none() {
            *current_date = today.clone();

            let file_path = self.get_log_file_path(&today);
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
            {
                Ok(file) => {
                    *current_file = Some(file);
                }
                Err(e) => {
                    eprintln!("Failed to open log file: {}", e);
                    return None;
                }
            }
        }

        // 克隆文件句柄
        current_file.as_ref().map(|f| {
            // 重新打开文件用于写入
            let file_path = self.get_log_file_path(&today);
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .ok()
        })?
    }

    /// 记录操作日志
    pub async fn log(
        &self,
        level: LogLevel,
        operator: &OperatorInfo,
        action: &str,
        details: impl Into<String>,
    ) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let details_str: String = details.into();

        let entry = LogEntry {
            timestamp,
            level,
            operator_type: operator.operator_type.clone(),
            operator_name: operator.operator_name.clone(),
            action: action.to_string(),
            details: details_str,
        };

        // 1. 输出到控制台
        println!("{}", entry.to_console_string());

        // 2. 写入文件
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.get_log_file_path(&Utc::now().format("%Y-%m-%d").to_string()))
        {
            let _ = file.write_all(entry.to_file_string().as_bytes());
        }

        // 3. 写入 Redis (List 结构，保留最近 10000 条)
        if let Some(redis) = &self.redis_client {
            let key = format!("logs:{}", Utc::now().format("%Y-%m-%d"));
            let value = entry.to_redis_json();

            // 使用 LPUSH 添加到列表头部
            let _ = redis.lpush(&key, &value).await;

            // 限制列表长度，保留最近 10000 条
            let _ = redis
                .execute_with_retry(|mut conn| {
                    let key = key.clone();
                    Box::pin(async move {
                        redis::cmd("LTRIM")
                            .arg(&key)
                            .arg(0)
                            .arg(9999)
                            .query_async::<_, ()>(&mut conn)
                            .await
                    })
                })
                .await;

            // 设置过期时间 30 天
            let _ = redis.expire(&key, 30 * 24 * 3600).await;
        }
    }

    /// 记录 Info 级别日志
    pub async fn info(
        &self,
        operator: &OperatorInfo,
        action: &str,
        details: impl Into<String>,
    ) {
        self.log(LogLevel::Info, operator, action, details).await;
    }

    /// 记录 Warning 级别日志
    pub async fn warning(
        &self,
        operator: &OperatorInfo,
        action: &str,
        details: impl Into<String>,
    ) {
        self.log(LogLevel::Warning, operator, action, details).await;
    }

    /// 记录 Error 级别日志
    pub async fn error(
        &self,
        operator: &OperatorInfo,
        action: &str,
        details: impl Into<String>,
    ) {
        self.log(LogLevel::Error, operator, action, details).await;
    }

    /// 记录系统操作（Info 级别）
    pub async fn log_system(&self, action: &str, details: impl Into<String>) {
        let operator = OperatorInfo::system();
        self.info(&operator, action, details).await;
    }

    /// 记录系统操作（Error 级别）
    pub async fn log_system_error(&self, action: &str, details: impl Into<String>) {
        let operator = OperatorInfo::system();
        self.error(&operator, action, details).await;
    }

    /// 记录用户操作（Info 级别）
    pub async fn log_user(
        &self,
        user_id: uuid::Uuid,
        identifier: &str,
        action: &str,
        details: impl Into<String>,
    ) {
        let operator = OperatorInfo::user(user_id, identifier);
        self.info(&operator, action, details).await;
    }

    /// 记录 Admin 操作（Info 级别）
    pub async fn log_admin(
        &self,
        user_id: uuid::Uuid,
        username: &str,
        action: &str,
        details: impl Into<String>,
    ) {
        let operator = OperatorInfo::admin(user_id, username);
        self.info(&operator, action, details).await;
    }

    /// 记录 ChatAI 操作
    pub async fn log_chatai(&self, user_id: &str, action: &str, details: impl Into<String>) {
        let operator = OperatorInfo::chatai(user_id);
        self.info(&operator, action, details).await;
    }

    /// 从 Redis 获取日志列表
    pub async fn get_logs_from_redis(
        &self,
        date: &str,
        start: isize,
        end: isize,
    ) -> Vec<String> {
        if let Some(redis) = &self.redis_client {
            let key = format!("logs:{}", date);
            match redis
                .execute_with_retry(|mut conn| {
                    let key = key.clone();
                    Box::pin(async move {
                        redis::cmd("LRANGE")
                            .arg(&key)
                            .arg(start)
                            .arg(end)
                            .query_async::<_, Vec<String>>(&mut conn)
                            .await
                    })
                })
                .await
            {
                Ok(logs) => logs,
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }

    /// 从文件获取日志
    pub fn get_logs_from_file(&self, date: &str) -> Vec<String> {
        let file_path = self.get_log_file_path(date);
        match fs::read_to_string(&file_path) {
            Ok(content) => content.lines().map(|s| s.to_string()).collect(),
            Err(_) => Vec::new(),
        }
    }
}

/// 全局日志记录器
use std::sync::OnceLock;
static GLOBAL_LOGGER: OnceLock<OperationLogger> = OnceLock::new();

/// 初始化全局日志记录器
pub fn init_global_logger(redis_client: Option<Arc<RedisClient>>) {
    let _ = GLOBAL_LOGGER.set(OperationLogger::new(redis_client));
}

/// 获取全局日志记录器
pub fn get_global_logger() -> &'static OperationLogger {
    GLOBAL_LOGGER.get().expect("Global logger not initialized")
}

/// 宏：记录系统 Info 日志
#[macro_export]
macro_rules! log_system_info {
    ($action:expr, $details:expr) => {
        if let Some(logger) = $crate::core::operation_logger::GLOBAL_LOGGER.get() {
            let _ = logger.log_system($action, $details);
        }
    };
}

/// 宏：记录系统 Error 日志
#[macro_export]
macro_rules! log_system_error {
    ($action:expr, $details:expr) => {
        if let Some(logger) = $crate::core::operation_logger::GLOBAL_LOGGER.get() {
            let _ = logger.log_system_error($action, $details);
        }
    };
}

/// 宏：记录用户操作
#[macro_export]
macro_rules! log_user {
    ($user_id:expr, $identifier:expr, $action:expr, $details:expr) => {
        if let Some(logger) = $crate::core::operation_logger::GLOBAL_LOGGER.get() {
            let _ = logger.log_user($user_id, $identifier, $action, $details);
        }
    };
}

/// 宏：记录 Admin 操作
#[macro_export]
macro_rules! log_admin {
    ($user_id:expr, $username:expr, $action:expr, $details:expr) => {
        if let Some(logger) = $crate::core::operation_logger::GLOBAL_LOGGER.get() {
            let _ = logger.log_admin($user_id, $username, $action, $details);
        }
    };
}
