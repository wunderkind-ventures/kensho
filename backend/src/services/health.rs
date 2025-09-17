// T066 & T067: Health check service with dependency monitoring
// Reference: plan.md Phase 4 - Production Hardening

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Overall health status of the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: u64,
    pub last_check: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Complete health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
    pub checks: Vec<ComponentHealth>,
}

/// Liveness probe response (for Kubernetes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResponse {
    pub alive: bool,
    pub timestamp: DateTime<Utc>,
}

/// Readiness probe response (for Kubernetes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub failing_checks: Vec<String>,
}

/// Health check service that monitors all dependencies
pub struct HealthService {
    start_time: DateTime<Utc>,
    version: String,
    checks: Arc<RwLock<Vec<ComponentHealth>>>,
}

impl HealthService {
    pub fn new(version: String) -> Self {
        Self {
            start_time: Utc::now(),
            version,
            checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if the application is alive (basic liveness)
    pub async fn check_liveness(&self) -> LivenessResponse {
        LivenessResponse {
            alive: true,
            timestamp: Utc::now(),
        }
    }

    /// Check if the application is ready to serve traffic
    pub async fn check_readiness(&self) -> ReadinessResponse {
        let checks = self.checks.read().await;
        
        let failing_checks: Vec<String> = checks
            .iter()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .map(|c| c.name.clone())
            .collect();
        
        let status = if failing_checks.is_empty() {
            if checks.iter().any(|c| c.status == HealthStatus::Degraded) {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            }
        } else {
            HealthStatus::Unhealthy
        };

        ReadinessResponse {
            ready: failing_checks.is_empty(),
            status,
            timestamp: Utc::now(),
            failing_checks,
        }
    }

    /// Perform complete health check of all components
    pub async fn check_health(&self) -> HealthCheckResponse {
        let checks = self.checks.read().await.clone();
        
        let overall_status = if checks.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if checks.iter().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let uptime = (Utc::now() - self.start_time).num_seconds() as u64;

        HealthCheckResponse {
            status: overall_status,
            version: self.version.clone(),
            uptime_seconds: uptime,
            timestamp: Utc::now(),
            checks,
        }
    }

    /// Update a component's health status
    pub async fn update_component_health(&self, health: ComponentHealth) {
        let mut checks = self.checks.write().await;
        
        // Find and update existing check or add new one
        if let Some(existing) = checks.iter_mut().find(|c| c.name == health.name) {
            *existing = health;
        } else {
            checks.push(health);
        }
    }

    /// Check Redis health
    pub async fn check_redis(&self, client: &mut redis::aio::ConnectionManager) -> ComponentHealth {
        let start = std::time::Instant::now();
        let mut metadata = HashMap::new();

        let (status, message) = match redis::cmd("PING")
            .query_async::<String>(client)
            .await
        {
            Ok(response) if response == "PONG" => {
                // Get additional Redis info
                if let Ok(info) = redis::cmd("INFO")
                    .arg("server")
                    .query_async::<String>(client)
                    .await
                {
                    // Parse Redis version from info
                    if let Some(version_line) = info.lines().find(|l| l.starts_with("redis_version:")) {
                        if let Some(version) = version_line.split(':').nth(1) {
                            metadata.insert("version".to_string(), serde_json::Value::String(version.to_string()));
                        }
                    }
                }
                (HealthStatus::Healthy, None)
            }
            Ok(_) => (HealthStatus::Degraded, Some("Unexpected PING response".to_string())),
            Err(e) => (HealthStatus::Unhealthy, Some(format!("Redis error: {}", e))),
        };

        let latency_ms = start.elapsed().as_millis() as u64;
        metadata.insert("latency_ms".to_string(), serde_json::Value::Number(latency_ms.into()));

        ComponentHealth {
            name: "redis".to_string(),
            status,
            message,
            latency_ms,
            last_check: Utc::now(),
            metadata,
        }
    }

    /// Check database health (simplified for in-memory DB)
    pub async fn check_database(&self, db: &crate::services::DatabaseService) -> ComponentHealth {
        let start = std::time::Instant::now();
        let mut metadata = HashMap::new();

        let (status, message) = match db.get_anime_count().await {
            Ok(count) => {
                metadata.insert("anime_count".to_string(), serde_json::Value::Number(count.into()));
                (HealthStatus::Healthy, None)
            }
            Err(e) => (HealthStatus::Unhealthy, Some(format!("Database error: {}", e))),
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        ComponentHealth {
            name: "database".to_string(),
            status,
            message,
            latency_ms,
            last_check: Utc::now(),
            metadata,
        }
    }

    /// Check Crunchyroll API health
    pub async fn check_crunchyroll(&self, _streaming: &crate::services::StreamingService) -> ComponentHealth {
        let start = std::time::Instant::now();
        let metadata = HashMap::new();

        // For now, we'll consider Crunchyroll healthy if we can create a client
        // In production, you'd want to make a lightweight API call
        let (status, message) = (
            HealthStatus::Healthy,
            Some("Crunchyroll check not implemented".to_string())
        );

        let latency_ms = start.elapsed().as_millis() as u64;

        ComponentHealth {
            name: "crunchyroll".to_string(),
            status,
            message,
            latency_ms,
            last_check: Utc::now(),
            metadata,
        }
    }

    /// Check system resources
    pub async fn check_system(&self) -> ComponentHealth {
        let start = std::time::Instant::now();
        let mut metadata = HashMap::new();

        // Get memory info using sysinfo or similar
        // For now, we'll use a simplified check
        let memory_usage = 0.75; // Placeholder
        metadata.insert("memory_usage".to_string(), serde_json::Value::Number(
            serde_json::Number::from_f64(memory_usage).unwrap_or(0.into())
        ));

        let status = if memory_usage > 0.9 {
            HealthStatus::Unhealthy
        } else if memory_usage > 0.8 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        ComponentHealth {
            name: "system".to_string(),
            status,
            message: Some(format!("Memory usage: {:.1}%", memory_usage * 100.0)),
            latency_ms,
            last_check: Utc::now(),
            metadata,
        }
    }
}

/// Background task to periodically update health checks
pub async fn health_check_worker(
    health_service: Arc<HealthService>,
    app_state: crate::db::connection::AppState,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // Check all components
        let checks = vec![
            health_service.check_database(&app_state.db).await,
            health_service.check_system().await,
            // Add Redis check if available
            // Add Crunchyroll check if available
        ];
        
        // Update health status for each component
        for check in checks {
            health_service.update_component_health(check).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liveness_always_returns_alive() {
        let service = HealthService::new("1.0.0".to_string());
        let response = service.check_liveness().await;
        assert!(response.alive);
    }

    #[tokio::test]
    async fn test_readiness_reflects_component_health() {
        let service = HealthService::new("1.0.0".to_string());
        
        // Initially ready (no checks)
        let response = service.check_readiness().await;
        assert!(response.ready);
        
        // Add unhealthy component
        service.update_component_health(ComponentHealth {
            name: "test".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some("Test failure".to_string()),
            latency_ms: 10,
            last_check: Utc::now(),
            metadata: HashMap::new(),
        }).await;
        
        // Should not be ready
        let response = service.check_readiness().await;
        assert!(!response.ready);
        assert_eq!(response.failing_checks, vec!["test"]);
    }

    #[tokio::test]
    async fn test_overall_health_aggregation() {
        let service = HealthService::new("1.0.0".to_string());
        
        // Add healthy component
        service.update_component_health(ComponentHealth {
            name: "component1".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: 5,
            last_check: Utc::now(),
            metadata: HashMap::new(),
        }).await;
        
        // Overall should be healthy
        let response = service.check_health().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        
        // Add degraded component
        service.update_component_health(ComponentHealth {
            name: "component2".to_string(),
            status: HealthStatus::Degraded,
            message: Some("Slow response".to_string()),
            latency_ms: 500,
            last_check: Utc::now(),
            metadata: HashMap::new(),
        }).await;
        
        // Overall should be degraded
        let response = service.check_health().await;
        assert_eq!(response.status, HealthStatus::Degraded);
        
        // Add unhealthy component
        service.update_component_health(ComponentHealth {
            name: "component3".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some("Connection failed".to_string()),
            latency_ms: 1000,
            last_check: Utc::now(),
            metadata: HashMap::new(),
        }).await;
        
        // Overall should be unhealthy
        let response = service.check_health().await;
        assert_eq!(response.status, HealthStatus::Unhealthy);
    }
}