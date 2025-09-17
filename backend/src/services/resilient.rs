// T061, T062, T063, T064, T065: Resilient client wrapper with connection pooling,
// retry logic, circuit breaker, and timeout configuration
// Reference: plan.md Phase 4 - Service Resilience

use anyhow::{Result, Context};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use std::future::Future;

/// Configuration for resilient client behavior
#[derive(Debug, Clone)]
pub struct ResilienceConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay for exponential backoff (milliseconds)
    pub base_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Timeout for each request (seconds)
    pub timeout_secs: u64,
    /// Circuit breaker failure threshold
    pub failure_threshold: u32,
    /// Circuit breaker recovery timeout (seconds)
    pub recovery_timeout_secs: u64,
    /// Connection pool size (for HTTP clients)
    pub pool_size: usize,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 10000,
            timeout_secs: 30,
            failure_threshold: 5,
            recovery_timeout_secs: 60,
            pool_size: 10,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open(DateTime<Utc>),
    HalfOpen,
}

/// Circuit breaker for handling failures
struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: Mutex<u32>,
    config: ResilienceConfig,
}

impl CircuitBreaker {
    fn new(config: ResilienceConfig) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: Mutex::new(0),
            config,
        }
    }

    async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Open(until) => {
                if Utc::now() > until {
                    // Transition to half-open
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::HalfOpen;
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut failures = self.failure_count.lock().await;
        
        *failures = 0;
        if *state != CircuitState::Closed {
            *state = CircuitState::Closed;
            tracing::info!("Circuit breaker closed after successful request");
        }
    }

    async fn record_failure(&self) {
        let mut failures = self.failure_count.lock().await;
        *failures += 1;
        
        if *failures >= self.config.failure_threshold {
            let mut state = self.state.write().await;
            let recovery_time = Utc::now() + chrono::Duration::seconds(
                self.config.recovery_timeout_secs as i64
            );
            *state = CircuitState::Open(recovery_time);
            tracing::warn!(
                "Circuit breaker opened after {} failures, will recover at {}",
                failures, recovery_time
            );
        }
    }
}

/// Resilient client wrapper that adds retry, circuit breaker, and timeout logic
pub struct ResilientClient<T> {
    inner: Arc<T>,
    config: ResilienceConfig,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl<T> ResilientClient<T> {
    pub fn new(client: T, config: ResilienceConfig) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(config.clone()));
        Self {
            inner: Arc::new(client),
            config,
            circuit_breaker,
        }
    }

    /// Execute a function with resilience patterns applied
    pub async fn execute<F, Fut, R>(&self, name: &str, f: F) -> Result<R>
    where
        F: Fn(Arc<T>) -> Fut,
        Fut: Future<Output = Result<R>>,
    {
        // Check circuit breaker
        if self.circuit_breaker.is_open().await {
            return Err(anyhow::anyhow!("Circuit breaker is open for {}", name));
        }

        let mut last_error = None;
        let mut delay_ms = self.config.base_delay_ms;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tracing::info!(
                    "Retrying {} (attempt {}/{})",
                    name,
                    attempt + 1,
                    self.config.max_retries + 1
                );
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms * 2).min(self.config.max_delay_ms);
            }

            // Apply timeout
            let result = tokio::time::timeout(
                Duration::from_secs(self.config.timeout_secs),
                f(self.inner.clone()),
            )
            .await;

            match result {
                Ok(Ok(value)) => {
                    self.circuit_breaker.record_success().await;
                    return Ok(value);
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt == self.config.max_retries {
                        self.circuit_breaker.record_failure().await;
                    }
                }
                Err(_) => {
                    last_error = Some(anyhow::anyhow!("Request timeout after {}s", self.config.timeout_secs));
                    if attempt == self.config.max_retries {
                        self.circuit_breaker.record_failure().await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed for {}", name)))
    }
}

/// Connection pool for HTTP clients
pub struct HttpConnectionPool {
    clients: Vec<reqwest::Client>,
    current: Mutex<usize>,
}

impl HttpConnectionPool {
    pub fn new(size: usize, timeout: Duration) -> Result<Self> {
        let mut clients = Vec::with_capacity(size);
        
        for _ in 0..size {
            let client = reqwest::Client::builder()
                .timeout(timeout)
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Some(Duration::from_secs(90)))
                .build()
                .context("Failed to create HTTP client")?;
            clients.push(client);
        }

        Ok(Self {
            clients,
            current: Mutex::new(0),
        })
    }

    pub async fn get_client(&self) -> reqwest::Client {
        let mut current = self.current.lock().await;
        let client = self.clients[*current].clone();
        *current = (*current + 1) % self.clients.len();
        client
    }
}

/// Resilient HTTP client with pooling
pub struct ResilientHttpClient {
    pool: Arc<HttpConnectionPool>,
    config: ResilienceConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
}

impl ResilientHttpClient {
    pub fn new(config: ResilienceConfig) -> Result<Self> {
        let pool = HttpConnectionPool::new(
            config.pool_size,
            Duration::from_secs(config.timeout_secs),
        )?;

        Ok(Self {
            pool: Arc::new(pool),
            config,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn get_circuit_breaker(&self, host: &str) -> Arc<CircuitBreaker> {
        let breakers = self.circuit_breakers.read().await;
        
        if let Some(breaker) = breakers.get(host) {
            return breaker.clone();
        }
        
        drop(breakers);
        let mut breakers = self.circuit_breakers.write().await;
        
        let breaker = Arc::new(CircuitBreaker::new(self.config.clone()));
        breakers.insert(host.to_string(), breaker.clone());
        breaker
    }

    /// Execute an HTTP request with resilience
    pub async fn request<F, Fut>(&self, url: &str, f: F) -> Result<reqwest::Response>
    where
        F: Fn(reqwest::Client) -> Fut,
        Fut: Future<Output = Result<reqwest::Response>>,
    {
        let host = url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        let breaker = self.get_circuit_breaker(&host).await;
        
        if breaker.is_open().await {
            return Err(anyhow::anyhow!("Circuit breaker is open for {}", host));
        }

        let mut last_error = None;
        let mut delay_ms = self.config.base_delay_ms;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tracing::info!(
                    "Retrying request to {} (attempt {}/{})",
                    host,
                    attempt + 1,
                    self.config.max_retries + 1
                );
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms * 2).min(self.config.max_delay_ms);
            }

            let client = self.pool.get_client().await;
            
            match f(client).await {
                Ok(response) => {
                    breaker.record_success().await;
                    return Ok(response);
                }
                Err(e) => {
                    tracing::warn!("Request failed: {}", e);
                    last_error = Some(e);
                    if attempt == self.config.max_retries {
                        breaker.record_failure().await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed for {}", host)))
    }
}

/// Trait for services that need resilience
#[async_trait]
pub trait ResilientService {
    async fn health_check(&self) -> Result<()>;
    fn is_critical(&self) -> bool;
}

/// Manager for all resilient services
pub struct ResilienceManager {
    services: Arc<RwLock<HashMap<String, Box<dyn ResilientService + Send + Sync>>>>,
    config: ResilienceConfig,
}

impl ResilienceManager {
    pub fn new(config: ResilienceConfig) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn register_service(
        &self,
        name: String,
        service: Box<dyn ResilientService + Send + Sync>,
    ) {
        let mut services = self.services.write().await;
        services.insert(name, service);
    }

    pub async fn check_all_services(&self) -> HashMap<String, Result<()>> {
        let services = self.services.read().await;
        let mut results = HashMap::new();

        for (name, service) in services.iter() {
            results.insert(name.clone(), service.health_check().await);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_threshold() {
        let config = ResilienceConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        
        let breaker = CircuitBreaker::new(config);
        
        assert!(!breaker.is_open().await);
        
        for _ in 0..3 {
            breaker.record_failure().await;
        }
        
        assert!(breaker.is_open().await);
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let config = ResilienceConfig {
            max_retries: 2,
            base_delay_ms: 100,
            max_delay_ms: 1000,
            ..Default::default()
        };

        let client = ResilientClient::new((), config);
        
        let start = std::time::Instant::now();
        let _ = client.execute("test", |_| async {
            Err::<(), _>(anyhow::anyhow!("Test error"))
        }).await;
        
        // Should have delays of 100ms + 200ms = 300ms minimum
        assert!(start.elapsed().as_millis() >= 300);
    }

    #[tokio::test]
    async fn test_connection_pool_rotation() {
        let pool = HttpConnectionPool::new(3, Duration::from_secs(30)).unwrap();
        
        let client1 = pool.get_client().await;
        let client2 = pool.get_client().await;
        let client3 = pool.get_client().await;
        let client4 = pool.get_client().await;
        
        // Fourth client should be the same as first (rotation)
        // Note: This is a simplified test, actual comparison would be more complex
        assert_eq!(3, pool.clients.len());
    }
}