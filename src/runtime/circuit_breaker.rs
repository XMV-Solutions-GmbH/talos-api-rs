// SPDX-License-Identifier: MIT OR Apache-2.0

//! Circuit breaker pattern implementation for resilient API calls.
//!
//! The circuit breaker prevents cascading failures by temporarily stopping
//! requests to failing endpoints and allowing them to recover.
//!
//! # States
//!
//! - **Closed**: Normal operation, requests pass through
//! - **Open**: Requests immediately fail (endpoint is down)
//! - **Half-Open**: Testing if endpoint has recovered
//!
//! # Example
//!
//! ```ignore
//! use talos_api::runtime::{CircuitBreaker, CircuitBreakerConfig};
//!
//! let config = CircuitBreakerConfig::default()
//!     .with_failure_threshold(5)
//!     .with_reset_timeout(Duration::from_secs(30));
//!
//! let breaker = CircuitBreaker::new(config);
//!
//! // Execute with circuit breaker protection
//! let result = breaker.call(|| async {
//!     client.version().await
//! }).await;
//! ```

use crate::error::{Result, TalosError};
use std::future::Future;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through normally.
    Closed,
    /// Circuit is open, requests fail immediately.
    Open,
    /// Circuit is half-open, testing if service has recovered.
    HalfOpen,
}

/// Configuration for the circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit.
    pub failure_threshold: usize,
    /// Number of successes in half-open state before closing.
    pub success_threshold: usize,
    /// Duration to wait before transitioning from open to half-open.
    pub reset_timeout: Duration,
    /// Maximum number of requests allowed in half-open state.
    pub half_open_max_requests: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(30),
            half_open_max_requests: 3,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new circuit breaker configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the failure threshold.
    #[must_use]
    pub fn with_failure_threshold(mut self, threshold: usize) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the success threshold for recovery.
    #[must_use]
    pub fn with_success_threshold(mut self, threshold: usize) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set the reset timeout.
    #[must_use]
    pub fn with_reset_timeout(mut self, timeout: Duration) -> Self {
        self.reset_timeout = timeout;
        self
    }

    /// Set the maximum half-open requests.
    #[must_use]
    pub fn with_half_open_max_requests(mut self, max: usize) -> Self {
        self.half_open_max_requests = max;
        self
    }
}

/// Circuit breaker for protecting against cascading failures.
///
/// The circuit breaker tracks failures and opens the circuit when the
/// failure threshold is reached, preventing further requests until
/// the reset timeout has elapsed.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failure_count: AtomicUsize,
    success_count: AtomicUsize,
    half_open_requests: AtomicUsize,
    last_failure_time: RwLock<Option<Instant>>,
    opened_at: RwLock<Option<Instant>>,
    total_calls: AtomicU64,
    total_failures: AtomicU64,
    total_rejections: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            half_open_requests: AtomicUsize::new(0),
            last_failure_time: RwLock::new(None),
            opened_at: RwLock::new(None),
            total_calls: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            total_rejections: AtomicU64::new(0),
        }
    }

    /// Create a circuit breaker with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Get the current circuit state.
    pub async fn state(&self) -> CircuitState {
        // Check if we should transition from open to half-open
        let current_state = *self.state.read().await;
        if current_state == CircuitState::Open {
            if let Some(opened_at) = *self.opened_at.read().await {
                if opened_at.elapsed() >= self.config.reset_timeout {
                    // Transition to half-open
                    let mut state = self.state.write().await;
                    if *state == CircuitState::Open {
                        *state = CircuitState::HalfOpen;
                        self.half_open_requests.store(0, Ordering::Relaxed);
                        self.success_count.store(0, Ordering::Relaxed);
                    }
                    return CircuitState::HalfOpen;
                }
            }
        }
        current_state
    }

    /// Check if the circuit allows requests.
    pub async fn can_execute(&self) -> bool {
        match self.state().await {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => {
                let current = self.half_open_requests.load(Ordering::Relaxed);
                current < self.config.half_open_max_requests
            }
        }
    }

    /// Execute an async operation with circuit breaker protection.
    ///
    /// # Errors
    ///
    /// Returns `TalosError::CircuitOpen` if the circuit is open.
    /// Returns the operation's error if it fails.
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        // Check if we can execute
        if !self.can_execute().await {
            self.total_rejections.fetch_add(1, Ordering::Relaxed);
            return Err(TalosError::CircuitOpen(format!(
                "Circuit breaker is open, will retry after {:?}",
                self.time_until_retry().await
            )));
        }

        // Track half-open requests
        let current_state = self.state().await;
        if current_state == CircuitState::HalfOpen {
            self.half_open_requests.fetch_add(1, Ordering::Relaxed);
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    /// Record a successful operation.
    async fn on_success(&self) {
        let state = *self.state.read().await;
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if successes >= self.config.success_threshold {
                    // Close the circuit
                    let mut state = self.state.write().await;
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // Should not happen, but reset anyway
                self.failure_count.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Record a failed operation.
    async fn on_failure(&self) {
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        *self.last_failure_time.write().await = Some(Instant::now());

        let state = *self.state.read().await;
        match state {
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failures >= self.config.failure_threshold {
                    // Open the circuit
                    self.open_circuit().await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                self.open_circuit().await;
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Open the circuit.
    async fn open_circuit(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Open;
        *self.opened_at.write().await = Some(Instant::now());
    }

    /// Manually reset the circuit breaker to closed state.
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.half_open_requests.store(0, Ordering::Relaxed);
        *self.opened_at.write().await = None;
    }

    /// Get the time until the circuit can retry (if open).
    pub async fn time_until_retry(&self) -> Option<Duration> {
        if *self.state.read().await != CircuitState::Open {
            return None;
        }

        self.opened_at.read().await.map(|opened| {
            let elapsed = opened.elapsed();
            if elapsed >= self.config.reset_timeout {
                Duration::ZERO
            } else {
                self.config.reset_timeout - elapsed
            }
        })
    }

    /// Get the current failure count.
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get total number of calls.
    #[must_use]
    pub fn total_calls(&self) -> u64 {
        self.total_calls.load(Ordering::Relaxed)
    }

    /// Get total number of failures.
    #[must_use]
    pub fn total_failures(&self) -> u64 {
        self.total_failures.load(Ordering::Relaxed)
    }

    /// Get total number of rejections (circuit open).
    #[must_use]
    pub fn total_rejections(&self) -> u64 {
        self.total_rejections.load(Ordering::Relaxed)
    }

    /// Get failure rate (0.0 to 1.0).
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        let total = self.total_calls.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let failures = self.total_failures.load(Ordering::Relaxed);
        failures as f64 / total as f64
    }

    /// Get the circuit breaker configuration.
    #[must_use]
    pub fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 2);
        assert_eq!(config.reset_timeout, Duration::from_secs(30));
        assert_eq!(config.half_open_max_requests, 3);
    }

    #[test]
    fn test_circuit_breaker_config_builder() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(10)
            .with_success_threshold(5)
            .with_reset_timeout(Duration::from_secs(60))
            .with_half_open_max_requests(5);

        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.success_threshold, 5);
        assert_eq!(config.reset_timeout, Duration::from_secs(60));
        assert_eq!(config.half_open_max_requests, 5);
    }

    #[tokio::test]
    async fn test_circuit_breaker_initial_state() {
        let breaker = CircuitBreaker::with_defaults();
        assert_eq!(breaker.state().await, CircuitState::Closed);
        assert!(breaker.can_execute().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig::new().with_failure_threshold(3);
        let breaker = CircuitBreaker::new(config);

        // Simulate failures
        for _ in 0..3 {
            let _ = breaker
                .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
                .await;
        }

        assert_eq!(breaker.state().await, CircuitState::Open);
        assert!(!breaker.can_execute().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_rejects_when_open() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(2)
            .with_reset_timeout(Duration::from_secs(60));
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
                .await;
        }

        // Try to execute - should be rejected
        let result = breaker
            .call(|| async { Ok::<_, TalosError>("success") })
            .await;

        assert!(matches!(result, Err(TalosError::CircuitOpen(_))));
        assert_eq!(breaker.total_rejections(), 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets_failures() {
        let config = CircuitBreakerConfig::new().with_failure_threshold(3);
        let breaker = CircuitBreaker::new(config);

        // Some failures
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
                .await;
        }
        assert_eq!(breaker.failure_count(), 2);

        // Success resets
        let _ = breaker.call(|| async { Ok::<_, TalosError>("ok") }).await;
        assert_eq!(breaker.failure_count(), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig::new().with_failure_threshold(2);
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
                .await;
        }
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Manual reset
        breaker.reset().await;
        assert_eq!(breaker.state().await, CircuitState::Closed);
        assert!(breaker.can_execute().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_transition() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(2)
            .with_reset_timeout(Duration::from_millis(50));
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
                .await;
        }
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Wait for reset timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should be half-open now
        assert_eq!(breaker.state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_success_in_half_open() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(2)
            .with_success_threshold(2)
            .with_reset_timeout(Duration::from_millis(10));
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
                .await;
        }

        // Wait for half-open
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(breaker.state().await, CircuitState::HalfOpen);

        // Successes in half-open
        for _ in 0..2 {
            let _ = breaker.call(|| async { Ok::<_, TalosError>("ok") }).await;
        }

        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_rate() {
        let breaker = CircuitBreaker::with_defaults();

        assert_eq!(breaker.failure_rate(), 0.0);

        // 4 successes, 1 failure
        for _ in 0..4 {
            let _ = breaker.call(|| async { Ok::<_, TalosError>("ok") }).await;
        }
        let _ = breaker
            .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
            .await;

        assert!((breaker.failure_rate() - 0.2).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_circuit_breaker_time_until_retry() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(2)
            .with_reset_timeout(Duration::from_secs(30));
        let breaker = CircuitBreaker::new(config);

        // Closed state - no retry time
        assert!(breaker.time_until_retry().await.is_none());

        // Open the circuit
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(TalosError::Connection("test".to_string())) })
                .await;
        }

        // Should have retry time
        let retry_time = breaker.time_until_retry().await;
        assert!(retry_time.is_some());
        assert!(retry_time.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_circuit_state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_ne!(CircuitState::Closed, CircuitState::Open);
        assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
    }
}
