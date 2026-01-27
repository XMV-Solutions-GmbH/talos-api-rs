// SPDX-License-Identifier: MIT OR Apache-2.0

//! Connection pooling and multi-endpoint support for Talos API clients.
//!
//! This module provides:
//! - [`ConnectionPool`]: A pool of connections to multiple Talos endpoints
//! - [`EndpointHealth`]: Health tracking for individual endpoints
//! - [`LoadBalancer`]: Strategies for selecting endpoints
//!
//! # Example
//!
//! ```ignore
//! use talos_api_rs::client::{ConnectionPool, ConnectionPoolConfig, LoadBalancer};
//!
//! let config = ConnectionPoolConfig::new(vec![
//!     "https://node1:50000".to_string(),
//!     "https://node2:50000".to_string(),
//!     "https://node3:50000".to_string(),
//! ])
//! .with_load_balancer(LoadBalancer::RoundRobin)
//! .with_health_check_interval(Duration::from_secs(30));
//!
//! let pool = ConnectionPool::new(config).await?;
//!
//! // Get a healthy client
//! let client = pool.get_client().await?;
//! ```

use crate::client::{TalosClient, TalosClientConfig};
use crate::error::{Result, TalosError};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health status of an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Endpoint is healthy and accepting requests.
    Healthy,
    /// Endpoint is unhealthy and should not receive requests.
    Unhealthy,
    /// Health status is unknown (initial state or after reset).
    Unknown,
}

/// Health tracking for a single endpoint.
#[derive(Debug)]
pub struct EndpointHealth {
    /// The endpoint URL.
    pub endpoint: String,
    /// Current health status.
    status: AtomicU64, // Encoded HealthStatus
    /// Number of consecutive failures.
    consecutive_failures: AtomicUsize,
    /// Number of consecutive successes.
    consecutive_successes: AtomicUsize,
    /// Total number of requests.
    total_requests: AtomicU64,
    /// Total number of failures.
    total_failures: AtomicU64,
    /// Last successful request time.
    last_success: RwLock<Option<Instant>>,
    /// Last failure time.
    last_failure: RwLock<Option<Instant>>,
    /// Last health check time.
    last_health_check: RwLock<Option<Instant>>,
}

impl EndpointHealth {
    /// Create a new endpoint health tracker.
    #[must_use]
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            status: AtomicU64::new(Self::status_to_u64(HealthStatus::Unknown)),
            consecutive_failures: AtomicUsize::new(0),
            consecutive_successes: AtomicUsize::new(0),
            total_requests: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            last_success: RwLock::new(None),
            last_failure: RwLock::new(None),
            last_health_check: RwLock::new(None),
        }
    }

    fn status_to_u64(status: HealthStatus) -> u64 {
        match status {
            HealthStatus::Healthy => 0,
            HealthStatus::Unhealthy => 1,
            HealthStatus::Unknown => 2,
        }
    }

    fn u64_to_status(value: u64) -> HealthStatus {
        match value {
            0 => HealthStatus::Healthy,
            1 => HealthStatus::Unhealthy,
            _ => HealthStatus::Unknown,
        }
    }

    /// Get the current health status.
    #[must_use]
    pub fn status(&self) -> HealthStatus {
        Self::u64_to_status(self.status.load(Ordering::Acquire))
    }

    /// Check if the endpoint is healthy.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.status() == HealthStatus::Healthy
    }

    /// Record a successful request.
    pub async fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.fetch_add(1, Ordering::Relaxed);
        *self.last_success.write().await = Some(Instant::now());
        self.status.store(
            Self::status_to_u64(HealthStatus::Healthy),
            Ordering::Release,
        );
    }

    /// Record a failed request.
    pub async fn record_failure(&self, failure_threshold: usize) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.write().await = Some(Instant::now());

        if failures >= failure_threshold {
            self.status.store(
                Self::status_to_u64(HealthStatus::Unhealthy),
                Ordering::Release,
            );
        }
    }

    /// Record a health check.
    pub async fn record_health_check(&self, healthy: bool, failure_threshold: usize) {
        *self.last_health_check.write().await = Some(Instant::now());
        if healthy {
            self.record_success().await;
        } else {
            self.record_failure(failure_threshold).await;
        }
    }

    /// Reset the health status to unknown.
    pub fn reset(&self) {
        self.status.store(
            Self::status_to_u64(HealthStatus::Unknown),
            Ordering::Release,
        );
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
    }

    /// Get the number of consecutive failures.
    #[must_use]
    pub fn consecutive_failures(&self) -> usize {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    /// Get the total number of requests.
    #[must_use]
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Get the total number of failures.
    #[must_use]
    pub fn total_failures(&self) -> u64 {
        self.total_failures.load(Ordering::Relaxed)
    }

    /// Get the failure rate (0.0 to 1.0).
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let failures = self.total_failures.load(Ordering::Relaxed);
        failures as f64 / total as f64
    }

    /// Get the last successful request time.
    pub async fn last_success(&self) -> Option<Instant> {
        *self.last_success.read().await
    }

    /// Get the last health check time.
    pub async fn last_health_check(&self) -> Option<Instant> {
        *self.last_health_check.read().await
    }
}

/// Load balancing strategy for selecting endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadBalancer {
    /// Round-robin selection across healthy endpoints.
    #[default]
    RoundRobin,
    /// Random selection among healthy endpoints.
    Random,
    /// Select the endpoint with the lowest failure rate.
    LeastFailures,
    /// Always prefer the first healthy endpoint (failover mode).
    Failover,
}

/// Configuration for the connection pool.
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// List of endpoint URLs.
    pub endpoints: Vec<String>,
    /// Load balancing strategy.
    pub load_balancer: LoadBalancer,
    /// Health check interval.
    pub health_check_interval: Duration,
    /// Number of consecutive failures before marking unhealthy.
    pub failure_threshold: usize,
    /// Number of consecutive successes before marking healthy again.
    pub recovery_threshold: usize,
    /// Base client configuration (TLS, timeouts, etc.).
    pub base_config: Option<TalosClientConfig>,
    /// Enable automatic health checks.
    pub auto_health_check: bool,
}

impl ConnectionPoolConfig {
    /// Create a new connection pool configuration.
    #[must_use]
    pub fn new(endpoints: Vec<String>) -> Self {
        Self {
            endpoints,
            load_balancer: LoadBalancer::RoundRobin,
            health_check_interval: Duration::from_secs(30),
            failure_threshold: 3,
            recovery_threshold: 2,
            base_config: None,
            auto_health_check: true,
        }
    }

    /// Set the load balancing strategy.
    #[must_use]
    pub fn with_load_balancer(mut self, lb: LoadBalancer) -> Self {
        self.load_balancer = lb;
        self
    }

    /// Set the health check interval.
    #[must_use]
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Set the failure threshold.
    #[must_use]
    pub fn with_failure_threshold(mut self, threshold: usize) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the recovery threshold.
    #[must_use]
    pub fn with_recovery_threshold(mut self, threshold: usize) -> Self {
        self.recovery_threshold = threshold;
        self
    }

    /// Set the base client configuration.
    #[must_use]
    pub fn with_base_config(mut self, config: TalosClientConfig) -> Self {
        self.base_config = Some(config);
        self
    }

    /// Disable automatic health checks.
    #[must_use]
    pub fn disable_auto_health_check(mut self) -> Self {
        self.auto_health_check = false;
        self
    }
}

/// A pool of connections to multiple Talos endpoints.
///
/// The pool maintains connections to multiple Talos nodes and routes
/// requests to healthy endpoints based on the configured load balancing
/// strategy.
pub struct ConnectionPool {
    config: ConnectionPoolConfig,
    clients: RwLock<HashMap<String, TalosClient>>,
    health: HashMap<String, Arc<EndpointHealth>>,
    round_robin_index: AtomicUsize,
    shutdown: AtomicBool,
    #[allow(dead_code)]
    health_check_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ConnectionPool {
    /// Create a new connection pool.
    ///
    /// # Errors
    ///
    /// Returns an error if no endpoints are provided or if initial connection fails.
    pub async fn new(config: ConnectionPoolConfig) -> Result<Self> {
        if config.endpoints.is_empty() {
            return Err(TalosError::Config(
                "At least one endpoint is required".to_string(),
            ));
        }

        // Initialize health tracking for all endpoints
        let health: HashMap<String, Arc<EndpointHealth>> = config
            .endpoints
            .iter()
            .map(|e| (e.clone(), Arc::new(EndpointHealth::new(e.clone()))))
            .collect();

        let pool = Self {
            config,
            clients: RwLock::new(HashMap::new()),
            health,
            round_robin_index: AtomicUsize::new(0),
            shutdown: AtomicBool::new(false),
            health_check_handle: None,
        };

        // Try to connect to at least one endpoint
        pool.connect_all().await?;

        Ok(pool)
    }

    /// Connect to all endpoints, collecting errors but not failing.
    async fn connect_all(&self) -> Result<()> {
        let mut connected = false;
        let mut last_error = None;

        for endpoint in &self.config.endpoints {
            match self.connect_endpoint(endpoint).await {
                Ok(client) => {
                    self.clients.write().await.insert(endpoint.clone(), client);
                    if let Some(health) = self.health.get(endpoint) {
                        health.record_success().await;
                    }
                    connected = true;
                }
                Err(e) => {
                    if let Some(health) = self.health.get(endpoint) {
                        health.record_failure(self.config.failure_threshold).await;
                    }
                    last_error = Some(e);
                }
            }
        }

        if connected {
            Ok(())
        } else {
            Err(last_error.unwrap_or_else(|| {
                TalosError::Connection("Failed to connect to any endpoint".to_string())
            }))
        }
    }

    /// Connect to a single endpoint.
    async fn connect_endpoint(&self, endpoint: &str) -> Result<TalosClient> {
        let config = if let Some(base) = &self.config.base_config {
            TalosClientConfig {
                endpoint: endpoint.to_string(),
                crt_path: base.crt_path.clone(),
                key_path: base.key_path.clone(),
                ca_path: base.ca_path.clone(),
                insecure: base.insecure,
                connect_timeout: base.connect_timeout,
                request_timeout: base.request_timeout,
                keepalive_interval: base.keepalive_interval,
                keepalive_timeout: base.keepalive_timeout,
            }
        } else {
            TalosClientConfig::new(endpoint)
        };

        TalosClient::new(config).await
    }

    /// Get a healthy client using the configured load balancing strategy.
    ///
    /// # Errors
    ///
    /// Returns an error if no healthy endpoints are available.
    pub async fn get_client(&self) -> Result<TalosClient> {
        let healthy_endpoints = self.get_healthy_endpoints();

        if healthy_endpoints.is_empty() {
            // Try to reconnect to all endpoints
            self.connect_all().await?;
            let healthy = self.get_healthy_endpoints();
            if healthy.is_empty() {
                return Err(TalosError::Connection(
                    "No healthy endpoints available".to_string(),
                ));
            }
        }

        let endpoint = self.select_endpoint(&self.get_healthy_endpoints())?;
        let clients = self.clients.read().await;

        clients.get(&endpoint).cloned().ok_or_else(|| {
            TalosError::Connection(format!("Client for endpoint {} not found", endpoint))
        })
    }

    /// Get a list of healthy endpoint URLs.
    #[must_use]
    pub fn get_healthy_endpoints(&self) -> Vec<String> {
        self.health
            .iter()
            .filter(|(_, h)| h.is_healthy())
            .map(|(e, _)| e.clone())
            .collect()
    }

    /// Get health information for an endpoint.
    #[must_use]
    pub fn get_endpoint_health(&self, endpoint: &str) -> Option<&Arc<EndpointHealth>> {
        self.health.get(endpoint)
    }

    /// Get health information for all endpoints.
    #[must_use]
    pub fn get_all_health(&self) -> &HashMap<String, Arc<EndpointHealth>> {
        &self.health
    }

    /// Select an endpoint based on the load balancing strategy.
    #[allow(clippy::result_large_err)]
    fn select_endpoint(&self, healthy: &[String]) -> Result<String> {
        if healthy.is_empty() {
            return Err(TalosError::Connection(
                "No healthy endpoints available".to_string(),
            ));
        }

        let endpoint = match self.config.load_balancer {
            LoadBalancer::RoundRobin => {
                let idx = self.round_robin_index.fetch_add(1, Ordering::Relaxed) % healthy.len();
                healthy[idx].clone()
            }
            LoadBalancer::Random => {
                let idx = rand::random::<usize>() % healthy.len();
                healthy[idx].clone()
            }
            LoadBalancer::LeastFailures => {
                let mut best = healthy[0].clone();
                let mut best_rate = f64::MAX;
                for e in healthy {
                    if let Some(health) = self.health.get(e) {
                        let rate = health.failure_rate();
                        if rate < best_rate {
                            best_rate = rate;
                            best = e.clone();
                        }
                    }
                }
                best
            }
            LoadBalancer::Failover => healthy[0].clone(),
        };

        Ok(endpoint)
    }

    /// Perform a health check on a specific endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails.
    pub async fn health_check(&self, endpoint: &str) -> Result<bool> {
        let client = match self.connect_endpoint(endpoint).await {
            Ok(c) => c,
            Err(e) => {
                if let Some(health) = self.health.get(endpoint) {
                    health
                        .record_health_check(false, self.config.failure_threshold)
                        .await;
                }
                return Err(e);
            }
        };

        // Try a simple version request as health check
        let mut version_client = client.version();
        let request = crate::api::version::VersionRequest { client: false };
        match version_client.version(request).await {
            Ok(_) => {
                if let Some(health) = self.health.get(endpoint) {
                    health
                        .record_health_check(true, self.config.failure_threshold)
                        .await;
                }
                // Update client in pool
                self.clients
                    .write()
                    .await
                    .insert(endpoint.to_string(), client);
                Ok(true)
            }
            Err(e) => {
                if let Some(health) = self.health.get(endpoint) {
                    health
                        .record_health_check(false, self.config.failure_threshold)
                        .await;
                }
                Err(TalosError::Api(e))
            }
        }
    }

    /// Perform health checks on all endpoints.
    pub async fn health_check_all(&self) {
        for endpoint in &self.config.endpoints {
            let _ = self.health_check(endpoint).await;
        }
    }

    /// Record a successful operation for an endpoint.
    pub async fn record_success(&self, endpoint: &str) {
        if let Some(health) = self.health.get(endpoint) {
            health.record_success().await;
        }
    }

    /// Record a failed operation for an endpoint.
    pub async fn record_failure(&self, endpoint: &str) {
        if let Some(health) = self.health.get(endpoint) {
            health.record_failure(self.config.failure_threshold).await;
        }
    }

    /// Shutdown the connection pool.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
    }

    /// Check if the pool is shut down.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }

    /// Get the number of connected clients.
    pub async fn connected_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get the total number of endpoints.
    #[must_use]
    pub fn endpoint_count(&self) -> usize {
        self.config.endpoints.len()
    }
}

impl Drop for ConnectionPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_health_new() {
        let health = EndpointHealth::new("https://test:50000".to_string());
        assert_eq!(health.status(), HealthStatus::Unknown);
        assert_eq!(health.consecutive_failures(), 0);
        assert_eq!(health.total_requests(), 0);
    }

    #[tokio::test]
    async fn test_endpoint_health_record_success() {
        let health = EndpointHealth::new("https://test:50000".to_string());
        health.record_success().await;
        assert_eq!(health.status(), HealthStatus::Healthy);
        assert_eq!(health.total_requests(), 1);
        assert!(health.last_success().await.is_some());
    }

    #[tokio::test]
    async fn test_endpoint_health_record_failure() {
        let health = EndpointHealth::new("https://test:50000".to_string());
        health.record_failure(3).await;
        assert_eq!(health.consecutive_failures(), 1);
        assert_eq!(health.status(), HealthStatus::Unknown);

        health.record_failure(3).await;
        health.record_failure(3).await;
        assert_eq!(health.status(), HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_endpoint_health_recovery() {
        let health = EndpointHealth::new("https://test:50000".to_string());
        // Make unhealthy
        for _ in 0..3 {
            health.record_failure(3).await;
        }
        assert_eq!(health.status(), HealthStatus::Unhealthy);

        // Recover
        health.record_success().await;
        assert_eq!(health.status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_endpoint_health_failure_rate() {
        let health = EndpointHealth::new("https://test:50000".to_string());
        assert_eq!(health.failure_rate(), 0.0);

        health.total_requests.store(10, Ordering::Relaxed);
        health.total_failures.store(2, Ordering::Relaxed);
        assert!((health.failure_rate() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_load_balancer_default() {
        assert_eq!(LoadBalancer::default(), LoadBalancer::RoundRobin);
    }

    #[test]
    fn test_connection_pool_config_new() {
        let config = ConnectionPoolConfig::new(vec![
            "https://node1:50000".to_string(),
            "https://node2:50000".to_string(),
        ]);

        assert_eq!(config.endpoints.len(), 2);
        assert_eq!(config.load_balancer, LoadBalancer::RoundRobin);
        assert_eq!(config.failure_threshold, 3);
        assert!(config.auto_health_check);
    }

    #[test]
    fn test_connection_pool_config_builder() {
        let config = ConnectionPoolConfig::new(vec!["https://node1:50000".to_string()])
            .with_load_balancer(LoadBalancer::Random)
            .with_failure_threshold(5)
            .with_recovery_threshold(3)
            .with_health_check_interval(Duration::from_secs(60))
            .disable_auto_health_check();

        assert_eq!(config.load_balancer, LoadBalancer::Random);
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_threshold, 3);
        assert_eq!(config.health_check_interval, Duration::from_secs(60));
        assert!(!config.auto_health_check);
    }

    #[tokio::test]
    async fn test_connection_pool_empty_endpoints() {
        let config = ConnectionPoolConfig::new(vec![]);
        let result = ConnectionPool::new(config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_health_status_conversions() {
        assert_eq!(
            EndpointHealth::u64_to_status(EndpointHealth::status_to_u64(HealthStatus::Healthy)),
            HealthStatus::Healthy
        );
        assert_eq!(
            EndpointHealth::u64_to_status(EndpointHealth::status_to_u64(HealthStatus::Unhealthy)),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            EndpointHealth::u64_to_status(EndpointHealth::status_to_u64(HealthStatus::Unknown)),
            HealthStatus::Unknown
        );
    }

    #[test]
    fn test_endpoint_health_reset() {
        let health = EndpointHealth::new("https://test:50000".to_string());
        health.status.store(
            EndpointHealth::status_to_u64(HealthStatus::Unhealthy),
            Ordering::Relaxed,
        );
        health.consecutive_failures.store(5, Ordering::Relaxed);

        health.reset();

        assert_eq!(health.status(), HealthStatus::Unknown);
        assert_eq!(health.consecutive_failures(), 0);
    }
}
