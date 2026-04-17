use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::api::routes::AppState;
use crate::core::error::AppError;

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exp as i32);
    format!("{:.2} {}", value, UNITS[exp])
}

#[derive(Debug, Serialize)]
pub struct MonitorStatusResponse {
    pub postgresql: DatabaseStatus,
    pub redis: RedisStatus,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct DatabaseStatus {
    pub connected: bool,
    pub status: String,
    pub message: String,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct RedisStatus {
    pub connected: bool,
    pub status: String,
    pub message: String,
    pub memory: Option<RedisMemoryInfo>,
    pub stats: Option<RedisStatsInfo>,
}

#[derive(Debug, Serialize)]
pub struct RedisMemoryInfo {
    pub used_memory: u64,
    pub max_memory: u64,
    pub usage_percent: f64,
    pub used_memory_human: String,
    pub max_memory_human: String,
}

#[derive(Debug, Serialize)]
pub struct RedisStatsInfo {
    pub connected_clients: u32,
    pub total_commands_processed: u64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
    pub hit_rate: f64,
    pub uptime_in_seconds: u64,
}

pub async fn get_monitor_status(
    State(state): State<AppState>,
) -> Result<Json<MonitorStatusResponse>, AppError> {
    let _start_time = std::time::Instant::now();

    let postgresql = if let Some(pool) = &state.pool {
        let pg_start = std::time::Instant::now();
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => {
                let elapsed = pg_start.elapsed().as_millis() as u64;
                DatabaseStatus {
                    connected: true,
                    status: "connected".to_string(),
                    message: "PostgreSQL connection is active".to_string(),
                    response_time_ms: Some(elapsed),
                }
            }
            Err(e) => DatabaseStatus {
                connected: false,
                status: "error".to_string(),
                message: format!("PostgreSQL query failed: {}", e),
                response_time_ms: None,
            },
        }
    } else {
        DatabaseStatus {
            connected: false,
            status: "not_initialized".to_string(),
            message: "PostgreSQL pool not initialized".to_string(),
            response_time_ms: None,
        }
    };

    let redis = if let Some(monitor) = &state.redis_monitor {
        // 实时检查 Redis 状态，而不是使用缓存的指标
        match monitor.immediate_check().await {
            Ok(metrics) => {
                if metrics.connected {
                    let memory = Some(RedisMemoryInfo {
                        used_memory: metrics.used_memory,
                        max_memory: metrics.max_memory,
                        usage_percent: metrics.memory_usage_percent,
                        used_memory_human: format_bytes(metrics.used_memory),
                        max_memory_human: format_bytes(metrics.max_memory),
                    });

                    let stats = Some(RedisStatsInfo {
                        connected_clients: metrics.connected_clients,
                        total_commands_processed: metrics.total_commands_processed,
                        keyspace_hits: metrics.keyspace_hits,
                        keyspace_misses: metrics.keyspace_misses,
                        hit_rate: metrics.hit_rate,
                        uptime_in_seconds: metrics.uptime_in_seconds,
                    });

                    RedisStatus {
                        connected: true,
                        status: "connected".to_string(),
                        message: format!(
                            "Redis connection is active (memory: {:.1}%)",
                            metrics.memory_usage_percent
                        ),
                        memory,
                        stats,
                    }
                } else {
                    RedisStatus {
                        connected: false,
                        status: "disconnected".to_string(),
                        message: "Redis is not connected".to_string(),
                        memory: None,
                        stats: None,
                    }
                }
            }
            Err(e) => RedisStatus {
                connected: false,
                status: "error".to_string(),
                message: format!("Failed to check Redis status: {}", e),
                memory: None,
                stats: None,
            },
        }
    } else {
        RedisStatus {
            connected: false,
            status: "not_initialized".to_string(),
            message: "Redis monitor not initialized".to_string(),
            memory: None,
            stats: None,
        }
    };

    let response = MonitorStatusResponse {
        postgresql,
        redis,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct BufferStatusResponse {
    pub enabled: bool,
    pub queue_length: usize,
    pub last_flush_time: Option<String>,
    pub total_flushed: u64,
    pub total_failed: u64,
}

pub async fn get_buffer_status(
    State(state): State<AppState>,
) -> Result<Json<BufferStatusResponse>, AppError> {
    if let Some(db_service) = &state.db_service {
        let service_status = db_service.status().await;

        let response = BufferStatusResponse {
            enabled: service_status.redis_connected,
            queue_length: service_status.buffer_size,
            last_flush_time: None,
            total_flushed: 0,
            total_failed: 0,
        };

        Ok(Json(response))
    } else {
        Ok(Json(BufferStatusResponse {
            enabled: false,
            queue_length: 0,
            last_flush_time: None,
            total_flushed: 0,
            total_failed: 0,
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct FlushBufferRequest {
    pub confirm: bool,
}

#[derive(Debug, Serialize)]
pub struct FlushBufferResponse {
    pub success: bool,
    pub message: String,
    pub flushed_count: Option<u32>,
}

pub async fn flush_buffer(
    State(state): State<AppState>,
    Json(payload): Json<FlushBufferRequest>,
) -> Result<Json<FlushBufferResponse>, AppError> {
    if !payload.confirm {
        return Ok(Json(FlushBufferResponse {
            success: false,
            message: "Confirmation required".to_string(),
            flushed_count: None,
        }));
    }

    if let Some(db_service) = &state.db_service {
        match db_service.flush_buffer().await {
            Ok(_) => Ok(Json(FlushBufferResponse {
                success: true,
                message: "Buffer flushed successfully".to_string(),
                flushed_count: None,
            })),
            Err(e) => Ok(Json(FlushBufferResponse {
                success: false,
                message: format!("Failed to flush buffer: {}", e),
                flushed_count: None,
            })),
        }
    } else {
        Ok(Json(FlushBufferResponse {
            success: false,
            message: "Database service not available".to_string(),
            flushed_count: None,
        }))
    }
}
