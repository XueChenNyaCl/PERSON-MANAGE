use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info, warn};

use super::client::RedisClient;
use super::error::Result;
use crate::core::config::MonitorConfig;

#[derive(Debug, Clone)]
pub struct RedisMetrics {
    pub connected: bool,
    pub used_memory: u64,
    pub max_memory: u64,
    pub memory_usage_percent: f64,
    pub connected_clients: u32,
    pub uptime_in_seconds: u64,
    pub total_commands_processed: u64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
    pub hit_rate: f64,
}

impl Default for RedisMetrics {
    fn default() -> Self {
        Self {
            connected: false,
            used_memory: 0,
            max_memory: 0,
            memory_usage_percent: 0.0,
            connected_clients: 0,
            uptime_in_seconds: 0,
            total_commands_processed: 0,
            keyspace_hits: 0,
            keyspace_misses: 0,
            hit_rate: 0.0,
        }
    }
}

pub struct RedisMonitor {
    client: RedisClient,
    config: MonitorConfig,
    last_metrics: Arc<RwLock<RedisMetrics>>,
    running: Arc<RwLock<bool>>,
}

impl RedisMonitor {
    pub fn new(client: RedisClient, config: MonitorConfig) -> Self {
        Self {
            client,
            config,
            last_metrics: Arc::new(RwLock::new(RedisMetrics::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn check_status(&self) -> Result<RedisMetrics> {
        if !self.client.is_connected().await {
            let metrics = RedisMetrics {
                connected: false,
                ..Default::default()
            };
            let mut last = self.last_metrics.write().await;
            *last = metrics.clone();
            return Ok(metrics);
        }

        let info_memory = self.client.info("memory").await.unwrap_or_default();
        let info_stats = self.client.info("stats").await.unwrap_or_default();
        let info_clients = self.client.info("clients").await.unwrap_or_default();
        let info_server = self.client.info("server").await.unwrap_or_default();

        let used_memory = Self::parse_info_u64(&info_memory, "used_memory:");
        let max_memory = Self::parse_info_u64(&info_memory, "maxmemory:");
        let memory_usage_percent = if max_memory > 0 {
            (used_memory as f64 / max_memory as f64) * 100.0
        } else {
            0.0
        };

        let connected_clients = Self::parse_info_u32(&info_clients, "connected_clients:");
        let uptime_in_seconds = Self::parse_info_u64(&info_server, "uptime_in_seconds:");
        let total_commands_processed =
            Self::parse_info_u64(&info_stats, "total_commands_processed:");
        let keyspace_hits = Self::parse_info_u64(&info_stats, "keyspace_hits:");
        let keyspace_misses = Self::parse_info_u64(&info_stats, "keyspace_misses:");

        let hit_rate = if keyspace_hits + keyspace_misses > 0 {
            (keyspace_hits as f64 / (keyspace_hits + keyspace_misses) as f64) * 100.0
        } else {
            0.0
        };

        let metrics = RedisMetrics {
            connected: true,
            used_memory,
            max_memory,
            memory_usage_percent,
            connected_clients,
            uptime_in_seconds,
            total_commands_processed,
            keyspace_hits,
            keyspace_misses,
            hit_rate,
        };

        let mut last = self.last_metrics.write().await;
        *last = metrics.clone();
        drop(last);

        self.check_alerts(&metrics).await;

        Ok(metrics)
    }

    async fn check_alerts(&self, metrics: &RedisMetrics) {
        if metrics.memory_usage_percent > self.config.memory_alert_threshold as f64 {
            warn!(
                "Redis memory usage is high: {:.1}% (threshold: {}%)",
                metrics.memory_usage_percent, self.config.memory_alert_threshold
            );
        }

        if metrics.connected_clients > 100 {
            warn!(
                "Redis has many connected clients: {}",
                metrics.connected_clients
            );
        }

        if metrics.hit_rate < 50.0 && metrics.keyspace_hits + metrics.keyspace_misses > 1000 {
            warn!("Redis cache hit rate is low: {:.1}%", metrics.hit_rate);
        }
    }

    #[allow(dead_code)]
    pub async fn get_last_metrics(&self) -> RedisMetrics {
        self.last_metrics.read().await.clone()
    }

    pub fn start_monitoring(self: Arc<Self>) {
        let interval_secs = self.config.interval_secs;
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            info!("Started Redis monitoring (interval: {}s)", interval_secs);

            {
                let mut r = running.write().await;
                *r = true;
            }

            loop {
                ticker.tick().await;

                {
                    let r = running.read().await;
                    if !*r {
                        break;
                    }
                }

                match self.check_status().await {
                    Ok(metrics) => {
                        if metrics.connected {
                            info!(
                                "Redis status: memory={:.1}%, clients={}, hit_rate={:.1}%",
                                metrics.memory_usage_percent,
                                metrics.connected_clients,
                                metrics.hit_rate
                            );
                        } else {
                            error!("Redis is not connected");
                        }
                    }
                    Err(e) => {
                        error!("Failed to check Redis status: {}", e);
                    }
                }
            }

            info!("Redis monitoring stopped");
        });
    }

    #[allow(dead_code)]
    pub async fn stop_monitoring(&self) {
        let mut r = self.running.write().await;
        *r = false;
    }

    #[allow(dead_code)]
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn immediate_check(&self) -> Result<RedisMetrics> {
        info!("Performing immediate Redis status check");
        self.check_status().await
    }

    fn parse_info_u64(info: &str, key: &str) -> u64 {
        info.lines()
            .find(|line| line.starts_with(key))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|val| val.trim().parse().ok())
            .unwrap_or(0)
    }

    fn parse_info_u32(info: &str, key: &str) -> u32 {
        info.lines()
            .find(|line| line.starts_with(key))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|val| val.trim().parse().ok())
            .unwrap_or(0)
    }
}
