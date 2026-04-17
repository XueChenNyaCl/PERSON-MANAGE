use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    #[allow(dead_code)]
    pub jwt_expires_in: String,
    #[allow(dead_code)]
    pub server_host: String,
    pub server_port: u16,
    #[allow(dead_code)]
    pub ws_path: String,
    #[allow(dead_code)]
    pub plugin_dir: String,
    pub redis: RedisConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
    #[allow(dead_code)]
    pub pool_size: u32,
    pub timeout_secs: u64,
    pub cache: CacheConfig,
    pub buffer: BufferConfig,
    pub rate_limit: RateLimitConfig,
    pub monitor: MonitorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {
    pub default_ttl_secs: u64,
    #[allow(dead_code)]
    pub person_ttl_secs: u64,
    #[allow(dead_code)]
    pub class_ttl_secs: u64,
    #[allow(dead_code)]
    pub score_ttl_secs: u64,
    #[allow(dead_code)]
    pub list_ttl_secs: u64,
    #[allow(dead_code)]
    pub chat_ttl_secs: u64,
    pub ai_chat_ttl_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BufferConfig {
    pub flush_interval_secs: u64,
    pub max_size: usize,
    pub batch_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub qps: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitorConfig {
    pub interval_secs: u64,
    pub memory_alert_threshold: u8,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            password: None,
            pool_size: 10,
            timeout_secs: 5,
            cache: CacheConfig::default(),
            buffer: BufferConfig::default(),
            rate_limit: RateLimitConfig::default(),
            monitor: MonitorConfig::default(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 300,
            person_ttl_secs: 600,
            class_ttl_secs: 300,
            score_ttl_secs: 180,
            list_ttl_secs: 60,
            chat_ttl_secs: 300,
            ai_chat_ttl_secs: 86400, // 24小时
        }
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            flush_interval_secs: 300,
            max_size: 10000,
            batch_size: 100,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self { qps: 50 }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interval_secs: 600,
            memory_alert_threshold: 80,
        }
    }
}

pub fn load_config() -> Result<Config, anyhow::Error> {
    dotenv().ok();

    let redis_config = RedisConfig {
        url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        password: env::var("REDIS_PASSWORD").ok(),
        pool_size: env::var("REDIS_POOL_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()?,
        timeout_secs: env::var("REDIS_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse()?,
        cache: CacheConfig {
            default_ttl_secs: env::var("CACHE_DEFAULT_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()?,
            person_ttl_secs: env::var("CACHE_PERSON_TTL")
                .unwrap_or_else(|_| "600".to_string())
                .parse()?,
            class_ttl_secs: env::var("CACHE_CLASS_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()?,
            score_ttl_secs: env::var("CACHE_SCORE_TTL")
                .unwrap_or_else(|_| "180".to_string())
                .parse()?,
            list_ttl_secs: env::var("CACHE_LIST_TTL")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
            chat_ttl_secs: env::var("CACHE_CHAT_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()?,
            ai_chat_ttl_secs: env::var("CACHE_AI_CHAT_TTL")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()?,
        },
        buffer: BufferConfig {
            flush_interval_secs: env::var("BUFFER_FLUSH_INTERVAL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()?,
            max_size: env::var("BUFFER_MAX_SIZE")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()?,
            batch_size: env::var("BUFFER_BATCH_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
        },
        rate_limit: RateLimitConfig {
            qps: env::var("RATE_LIMIT_QPS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()?,
        },
        monitor: MonitorConfig {
            interval_secs: env::var("MONITOR_INTERVAL")
                .unwrap_or_else(|_| "600".to_string())
                .parse()?,
            memory_alert_threshold: env::var("MEMORY_ALERT_THRESHOLD")
                .unwrap_or_else(|_| "80".to_string())
                .parse()?,
        },
    };

    let config = Config {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        jwt_expires_in: env::var("JWT_EXPIRES_IN").unwrap_or_else(|_| "24h".to_string()),
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        server_port: env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()?,
        ws_path: env::var("WS_PATH").unwrap_or_else(|_| "/ws".to_string()),
        plugin_dir: env::var("PLUGIN_DIR").unwrap_or_else(|_| "plugins".to_string()),
        redis: redis_config,
    };

    Ok(config)
}
