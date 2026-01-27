// SPDX-License-Identifier: MIT OR Apache-2.0

//! Retry policies and backoff strategies for resilient API calls.
//!
//! This module provides configurable retry behavior for handling transient failures
//! in gRPC communication with Talos nodes.
//!
//! # Example
//!
//! ```
//! use talos_api_rs::runtime::{RetryConfig, ExponentialBackoff};
//! use std::time::Duration;
//!
//! let retry = RetryConfig::builder()
//!     .max_retries(3)
//!     .backoff(ExponentialBackoff::new(Duration::from_millis(100)))
//!     .build();
//! ```

use std::time::Duration;

/// Defines a backoff strategy for retry delays.
pub trait BackoffStrategy: Clone + Send + Sync + 'static {
    /// Calculate the delay before the next retry attempt.
    ///
    /// # Arguments
    /// * `attempt` - The current attempt number (0-indexed)
    fn delay(&self, attempt: u32) -> Duration;
}

// =============================================================================
// No Backoff
// =============================================================================

/// No delay between retries.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoBackoff;

impl NoBackoff {
    /// Create a new no-backoff strategy.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl BackoffStrategy for NoBackoff {
    fn delay(&self, _attempt: u32) -> Duration {
        Duration::ZERO
    }
}

// =============================================================================
// Fixed Backoff
// =============================================================================

/// Fixed delay between retries.
#[derive(Debug, Clone, Copy)]
pub struct FixedBackoff {
    delay: Duration,
}

impl FixedBackoff {
    /// Create a new fixed backoff strategy.
    #[must_use]
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }

    /// Create a fixed backoff with delay in milliseconds.
    #[must_use]
    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }

    /// Create a fixed backoff with delay in seconds.
    #[must_use]
    pub fn from_secs(secs: u64) -> Self {
        Self::new(Duration::from_secs(secs))
    }
}

impl Default for FixedBackoff {
    fn default() -> Self {
        Self::new(Duration::from_millis(100))
    }
}

impl BackoffStrategy for FixedBackoff {
    fn delay(&self, _attempt: u32) -> Duration {
        self.delay
    }
}

// =============================================================================
// Linear Backoff
// =============================================================================

/// Linear backoff - delay increases linearly with each attempt.
#[derive(Debug, Clone, Copy)]
pub struct LinearBackoff {
    initial_delay: Duration,
    increment: Duration,
    max_delay: Duration,
}

impl LinearBackoff {
    /// Create a new linear backoff strategy.
    #[must_use]
    pub fn new(initial_delay: Duration) -> Self {
        Self {
            initial_delay,
            increment: initial_delay,
            max_delay: Duration::from_secs(30),
        }
    }

    /// Set the increment for each retry.
    #[must_use]
    pub fn with_increment(mut self, increment: Duration) -> Self {
        self.increment = increment;
        self
    }

    /// Set the maximum delay cap.
    #[must_use]
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }
}

impl Default for LinearBackoff {
    fn default() -> Self {
        Self::new(Duration::from_millis(100))
    }
}

impl BackoffStrategy for LinearBackoff {
    fn delay(&self, attempt: u32) -> Duration {
        let delay = self.initial_delay + self.increment * attempt;
        delay.min(self.max_delay)
    }
}

// =============================================================================
// Exponential Backoff
// =============================================================================

/// Exponential backoff - delay doubles with each attempt.
///
/// Optionally includes jitter to prevent thundering herd.
#[derive(Debug, Clone, Copy)]
pub struct ExponentialBackoff {
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
    jitter: bool,
}

impl ExponentialBackoff {
    /// Create a new exponential backoff strategy.
    #[must_use]
    pub fn new(initial_delay: Duration) -> Self {
        Self {
            initial_delay,
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Set the maximum delay cap.
    #[must_use]
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Set the multiplier for exponential growth.
    #[must_use]
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Enable or disable jitter.
    #[must_use]
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(Duration::from_millis(100))
    }
}

impl BackoffStrategy for ExponentialBackoff {
    fn delay(&self, attempt: u32) -> Duration {
        let base_delay =
            self.initial_delay.as_millis() as f64 * self.multiplier.powi(attempt as i32);
        let capped_delay = base_delay.min(self.max_delay.as_millis() as f64);

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter_range = capped_delay * 0.25;
            // Simple deterministic jitter based on attempt number
            let jitter = (attempt as f64 * 0.1).sin().abs() * jitter_range;
            capped_delay + jitter
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay as u64)
    }
}

// =============================================================================
// Retry Policy
// =============================================================================

/// Determines whether a gRPC error should be retried.
pub trait RetryPolicy: Clone + Send + Sync + 'static {
    /// Returns `true` if the operation should be retried for this error.
    fn should_retry(&self, code: tonic::Code) -> bool;
}

/// Default retry policy - retries on transient errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultRetryPolicy;

impl RetryPolicy for DefaultRetryPolicy {
    fn should_retry(&self, code: tonic::Code) -> bool {
        matches!(
            code,
            tonic::Code::Unavailable
                | tonic::Code::Unknown
                | tonic::Code::DeadlineExceeded
                | tonic::Code::ResourceExhausted
                | tonic::Code::Aborted
        )
    }
}

/// Never retry - fail immediately.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoRetryPolicy;

impl RetryPolicy for NoRetryPolicy {
    fn should_retry(&self, _code: tonic::Code) -> bool {
        false
    }
}

/// Custom retry policy based on a list of codes.
#[derive(Debug, Clone)]
pub struct CustomRetryPolicy {
    retry_codes: Vec<tonic::Code>,
}

impl CustomRetryPolicy {
    /// Create a policy that retries on specific codes.
    #[must_use]
    pub fn new(retry_codes: Vec<tonic::Code>) -> Self {
        Self { retry_codes }
    }

    /// Create a policy for network-level errors only.
    #[must_use]
    pub fn network_errors() -> Self {
        Self::new(vec![tonic::Code::Unavailable, tonic::Code::Unknown])
    }
}

impl RetryPolicy for CustomRetryPolicy {
    fn should_retry(&self, code: tonic::Code) -> bool {
        self.retry_codes.contains(&code)
    }
}

// =============================================================================
// Retry Configuration
// =============================================================================

/// Complete retry configuration combining policy and backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig<P: RetryPolicy = DefaultRetryPolicy, B: BackoffStrategy = ExponentialBackoff>
{
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Policy determining which errors to retry.
    pub policy: P,
    /// Backoff strategy for calculating delays.
    pub backoff: B,
    /// Maximum total time for all retries.
    pub total_timeout: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            policy: DefaultRetryPolicy,
            backoff: ExponentialBackoff::default(),
            total_timeout: Some(Duration::from_secs(30)),
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration builder.
    #[must_use]
    pub fn builder() -> RetryConfigBuilder<DefaultRetryPolicy, ExponentialBackoff> {
        RetryConfigBuilder::new()
    }

    /// Disable retries.
    #[must_use]
    pub fn disabled() -> RetryConfig<NoRetryPolicy, NoBackoff> {
        RetryConfig {
            max_retries: 0,
            policy: NoRetryPolicy,
            backoff: NoBackoff,
            total_timeout: None,
        }
    }
}

impl<P: RetryPolicy, B: BackoffStrategy> RetryConfig<P, B> {
    /// Execute an async operation with retry logic.
    pub async fn execute<T, E, F, Fut>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: AsGrpcStatus,
    {
        let start = std::time::Instant::now();
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let code = e.grpc_code();

                    // Check if we should retry
                    if !self.policy.should_retry(code) {
                        return Err(e);
                    }

                    // Check if we've exceeded max retries
                    if attempt >= self.max_retries {
                        return Err(e);
                    }

                    // Check total timeout
                    if let Some(timeout) = self.total_timeout {
                        if start.elapsed() >= timeout {
                            return Err(e);
                        }
                    }

                    // Calculate delay and sleep
                    let delay = self.backoff.delay(attempt);
                    tokio::time::sleep(delay).await;

                    attempt += 1;
                }
            }
        }
    }
}

/// Builder for `RetryConfig`.
#[derive(Debug, Clone)]
pub struct RetryConfigBuilder<P: RetryPolicy, B: BackoffStrategy> {
    max_retries: u32,
    policy: P,
    backoff: B,
    total_timeout: Option<Duration>,
}

impl RetryConfigBuilder<DefaultRetryPolicy, ExponentialBackoff> {
    /// Create a new builder with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            policy: DefaultRetryPolicy,
            backoff: ExponentialBackoff::default(),
            total_timeout: Some(Duration::from_secs(30)),
        }
    }
}

impl Default for RetryConfigBuilder<DefaultRetryPolicy, ExponentialBackoff> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: RetryPolicy, B: BackoffStrategy> RetryConfigBuilder<P, B> {
    /// Set maximum retry attempts.
    #[must_use]
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Set the retry policy.
    #[must_use]
    pub fn policy<P2: RetryPolicy>(self, policy: P2) -> RetryConfigBuilder<P2, B> {
        RetryConfigBuilder {
            max_retries: self.max_retries,
            policy,
            backoff: self.backoff,
            total_timeout: self.total_timeout,
        }
    }

    /// Set the backoff strategy.
    #[must_use]
    pub fn backoff<B2: BackoffStrategy>(self, backoff: B2) -> RetryConfigBuilder<P, B2> {
        RetryConfigBuilder {
            max_retries: self.max_retries,
            policy: self.policy,
            backoff,
            total_timeout: self.total_timeout,
        }
    }

    /// Set the total timeout for all retries.
    #[must_use]
    pub fn total_timeout(mut self, timeout: Duration) -> Self {
        self.total_timeout = Some(timeout);
        self
    }

    /// Disable total timeout.
    #[must_use]
    pub fn no_total_timeout(mut self) -> Self {
        self.total_timeout = None;
        self
    }

    /// Build the configuration.
    #[must_use]
    pub fn build(self) -> RetryConfig<P, B> {
        RetryConfig {
            max_retries: self.max_retries,
            policy: self.policy,
            backoff: self.backoff,
            total_timeout: self.total_timeout,
        }
    }
}

/// Trait for extracting gRPC status codes from errors.
pub trait AsGrpcStatus {
    /// Extract the gRPC status code.
    fn grpc_code(&self) -> tonic::Code;
}

impl AsGrpcStatus for tonic::Status {
    fn grpc_code(&self) -> tonic::Code {
        self.code()
    }
}

impl<T> AsGrpcStatus for Result<T, tonic::Status> {
    fn grpc_code(&self) -> tonic::Code {
        match self {
            Ok(_) => tonic::Code::Ok,
            Err(e) => e.code(),
        }
    }
}

// Implement for our error type
impl AsGrpcStatus for crate::error::TalosError {
    fn grpc_code(&self) -> tonic::Code {
        match self {
            crate::error::TalosError::Api(status) => status.code(),
            crate::error::TalosError::Transport(_) => tonic::Code::Unavailable,
            crate::error::TalosError::Config(_) => tonic::Code::InvalidArgument,
            crate::error::TalosError::Validation(_) => tonic::Code::InvalidArgument,
            crate::error::TalosError::Connection(_) => tonic::Code::Unavailable,
            crate::error::TalosError::CircuitOpen(_) => tonic::Code::Unavailable,
            crate::error::TalosError::Unknown(_) => tonic::Code::Internal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_backoff() {
        let backoff = NoBackoff::new();
        assert_eq!(backoff.delay(0), Duration::ZERO);
        assert_eq!(backoff.delay(5), Duration::ZERO);
        assert_eq!(backoff.delay(100), Duration::ZERO);
    }

    #[test]
    fn test_fixed_backoff() {
        let backoff = FixedBackoff::from_millis(100);
        assert_eq!(backoff.delay(0), Duration::from_millis(100));
        assert_eq!(backoff.delay(5), Duration::from_millis(100));
        assert_eq!(backoff.delay(100), Duration::from_millis(100));
    }

    #[test]
    fn test_linear_backoff() {
        let backoff = LinearBackoff::new(Duration::from_millis(100))
            .with_increment(Duration::from_millis(50))
            .with_max_delay(Duration::from_millis(500));

        assert_eq!(backoff.delay(0), Duration::from_millis(100));
        assert_eq!(backoff.delay(1), Duration::from_millis(150));
        assert_eq!(backoff.delay(2), Duration::from_millis(200));
        assert_eq!(backoff.delay(10), Duration::from_millis(500)); // Capped
    }

    #[test]
    fn test_exponential_backoff() {
        let backoff = ExponentialBackoff::new(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10))
            .with_jitter(false);

        assert_eq!(backoff.delay(0), Duration::from_millis(100));
        assert_eq!(backoff.delay(1), Duration::from_millis(200));
        assert_eq!(backoff.delay(2), Duration::from_millis(400));
        assert_eq!(backoff.delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_exponential_backoff_cap() {
        let backoff = ExponentialBackoff::new(Duration::from_millis(100))
            .with_max_delay(Duration::from_millis(500))
            .with_jitter(false);

        assert_eq!(backoff.delay(5), Duration::from_millis(500)); // Capped at 500ms
    }

    #[test]
    fn test_default_retry_policy() {
        let policy = DefaultRetryPolicy;

        assert!(policy.should_retry(tonic::Code::Unavailable));
        assert!(policy.should_retry(tonic::Code::DeadlineExceeded));
        assert!(policy.should_retry(tonic::Code::ResourceExhausted));
        assert!(policy.should_retry(tonic::Code::Aborted));

        assert!(!policy.should_retry(tonic::Code::InvalidArgument));
        assert!(!policy.should_retry(tonic::Code::NotFound));
        assert!(!policy.should_retry(tonic::Code::PermissionDenied));
        assert!(!policy.should_retry(tonic::Code::AlreadyExists));
    }

    #[test]
    fn test_no_retry_policy() {
        let policy = NoRetryPolicy;

        assert!(!policy.should_retry(tonic::Code::Unavailable));
        assert!(!policy.should_retry(tonic::Code::Unknown));
    }

    #[test]
    fn test_custom_retry_policy() {
        let policy = CustomRetryPolicy::network_errors();

        assert!(policy.should_retry(tonic::Code::Unavailable));
        assert!(policy.should_retry(tonic::Code::Unknown));
        assert!(!policy.should_retry(tonic::Code::DeadlineExceeded));
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::builder()
            .max_retries(5)
            .backoff(FixedBackoff::from_millis(200))
            .total_timeout(Duration::from_secs(60))
            .build();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.total_timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_retry_config_disabled() {
        let config = RetryConfig::disabled();

        assert_eq!(config.max_retries, 0);
        assert_eq!(config.total_timeout, None);
    }

    #[tokio::test]
    async fn test_retry_execute_success() {
        let config = RetryConfig::default();

        let result: Result<i32, tonic::Status> = config.execute(|| async { Ok(42) }).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_execute_transient_failure() {
        let config = RetryConfig::builder()
            .max_retries(3)
            .backoff(NoBackoff::new())
            .build();

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result: Result<i32, tonic::Status> = config
            .execute(|| {
                let count = call_count_clone.clone();
                async move {
                    let n = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if n < 2 {
                        Err(tonic::Status::unavailable("transient"))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_execute_permanent_failure() {
        let config = RetryConfig::builder()
            .max_retries(3)
            .backoff(NoBackoff::new())
            .build();

        let result: Result<i32, tonic::Status> = config
            .execute(|| async { Err(tonic::Status::invalid_argument("bad input")) })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }
}
