// SPDX-License-Identifier: MIT OR Apache-2.0

//! Logging interceptor for gRPC requests and responses.
//!
//! Provides structured logging of all API calls with timing, request/response
//! metadata, and error details.
//!
//! # Example
//!
//! ```ignore
//! use talos_api_rs::runtime::{LoggingInterceptor, LogLevel};
//!
//! // Create interceptor with custom log level
//! let interceptor = LoggingInterceptor::new()
//!     .with_level(LogLevel::Debug)
//!     .with_request_body(true)
//!     .with_response_body(true);
//!
//! // Use with TalosClient
//! let client = TalosClient::with_interceptor(config, interceptor).await?;
//! ```

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tonic::service::Interceptor;
use tonic::{Request, Status};
use tracing::{debug, error, info, trace, warn};

/// Log level for the logging interceptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// Trace level - most verbose.
    Trace,
    /// Debug level.
    Debug,
    /// Info level.
    #[default]
    Info,
    /// Warn level.
    Warn,
    /// Error level - only errors.
    Error,
    /// Disabled - no logging.
    Off,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Off => write!(f, "OFF"),
        }
    }
}

/// Configuration for the logging interceptor.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level for successful requests.
    pub success_level: LogLevel,
    /// Log level for failed requests.
    pub error_level: LogLevel,
    /// Whether to log request metadata.
    pub log_metadata: bool,
    /// Whether to log request path.
    pub log_path: bool,
    /// Whether to redact sensitive headers.
    pub redact_sensitive: bool,
    /// List of sensitive header names to redact.
    pub sensitive_headers: Vec<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            success_level: LogLevel::Info,
            error_level: LogLevel::Error,
            log_metadata: true,
            log_path: true,
            redact_sensitive: true,
            sensitive_headers: vec![
                "authorization".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the success log level.
    #[must_use]
    pub fn with_success_level(mut self, level: LogLevel) -> Self {
        self.success_level = level;
        self
    }

    /// Set the error log level.
    #[must_use]
    pub fn with_error_level(mut self, level: LogLevel) -> Self {
        self.error_level = level;
        self
    }

    /// Enable or disable metadata logging.
    #[must_use]
    pub fn with_metadata(mut self, enabled: bool) -> Self {
        self.log_metadata = enabled;
        self
    }

    /// Enable or disable path logging.
    #[must_use]
    pub fn with_path(mut self, enabled: bool) -> Self {
        self.log_path = enabled;
        self
    }

    /// Enable or disable sensitive data redaction.
    #[must_use]
    pub fn with_redaction(mut self, enabled: bool) -> Self {
        self.redact_sensitive = enabled;
        self
    }

    /// Add a sensitive header to redact.
    #[must_use]
    pub fn with_sensitive_header(mut self, header: impl Into<String>) -> Self {
        self.sensitive_headers.push(header.into());
        self
    }

    /// Create a verbose configuration for debugging.
    #[must_use]
    pub fn verbose() -> Self {
        Self {
            success_level: LogLevel::Debug,
            error_level: LogLevel::Error,
            log_metadata: true,
            log_path: true,
            redact_sensitive: true,
            sensitive_headers: vec![
                "authorization".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
        }
    }

    /// Create a quiet configuration for production.
    #[must_use]
    pub fn quiet() -> Self {
        Self {
            success_level: LogLevel::Off,
            error_level: LogLevel::Warn,
            log_metadata: false,
            log_path: true,
            redact_sensitive: true,
            sensitive_headers: vec![
                "authorization".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
        }
    }
}

/// Metrics collected by the logging interceptor.
#[derive(Debug, Default)]
pub struct InterceptorMetrics {
    /// Total number of requests.
    total_requests: AtomicU64,
    /// Number of successful requests.
    successful_requests: AtomicU64,
    /// Number of failed requests.
    failed_requests: AtomicU64,
}

impl InterceptorMetrics {
    /// Create a new metrics instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful request.
    pub fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed request.
    pub fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the total number of requests.
    #[must_use]
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Get the number of successful requests.
    #[must_use]
    pub fn successful_requests(&self) -> u64 {
        self.successful_requests.load(Ordering::Relaxed)
    }

    /// Get the number of failed requests.
    #[must_use]
    pub fn failed_requests(&self) -> u64 {
        self.failed_requests.load(Ordering::Relaxed)
    }

    /// Get the success rate (0.0 to 1.0).
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }
        let successful = self.successful_requests.load(Ordering::Relaxed);
        successful as f64 / total as f64
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
    }
}

/// A gRPC interceptor that logs requests.
///
/// This interceptor logs the start of each request. For complete request/response
/// logging including timing and response status, you should combine this with
/// tower middleware or use the `RequestLogger` wrapper.
#[derive(Clone)]
pub struct LoggingInterceptor {
    config: LoggingConfig,
}

impl LoggingInterceptor {
    /// Create a new logging interceptor with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: LoggingConfig::default(),
        }
    }

    /// Create a logging interceptor with custom configuration.
    #[must_use]
    pub fn with_config(config: LoggingConfig) -> Self {
        Self { config }
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &LoggingConfig {
        &self.config
    }

    fn log_request<T>(&self, request: &Request<T>) {
        if self.config.success_level == LogLevel::Off {
            return;
        }

        let metadata_str = if self.config.log_metadata {
            let mut parts = Vec::new();
            for key_and_value in request.metadata().iter() {
                match key_and_value {
                    tonic::metadata::KeyAndValueRef::Ascii(key, value) => {
                        let key_str = key.as_str();
                        if self.config.redact_sensitive
                            && self
                                .config
                                .sensitive_headers
                                .iter()
                                .any(|h| h.eq_ignore_ascii_case(key_str))
                        {
                            parts.push(format!("{}=[REDACTED]", key_str));
                        } else {
                            parts.push(format!("{}={:?}", key_str, value));
                        }
                    }
                    tonic::metadata::KeyAndValueRef::Binary(key, value) => {
                        let key_str = key.as_str();
                        if self.config.redact_sensitive
                            && self
                                .config
                                .sensitive_headers
                                .iter()
                                .any(|h| h.eq_ignore_ascii_case(key_str))
                        {
                            parts.push(format!("{}=[REDACTED]", key_str));
                        } else {
                            parts.push(format!("{}={:?}", key_str, value));
                        }
                    }
                }
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" metadata=[{}]", parts.join(", "))
            }
        } else {
            String::new()
        };

        match self.config.success_level {
            LogLevel::Trace => {
                trace!(target: "talos_api::grpc", "gRPC request{}", metadata_str);
            }
            LogLevel::Debug => {
                debug!(target: "talos_api::grpc", "gRPC request{}", metadata_str);
            }
            LogLevel::Info => {
                info!(target: "talos_api::grpc", "gRPC request{}", metadata_str);
            }
            LogLevel::Warn => {
                warn!(target: "talos_api::grpc", "gRPC request{}", metadata_str);
            }
            LogLevel::Error => {
                error!(target: "talos_api::grpc", "gRPC request{}", metadata_str);
            }
            LogLevel::Off => {}
        }
    }
}

impl Default for LoggingInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Interceptor for LoggingInterceptor {
    fn call(&mut self, request: Request<()>) -> std::result::Result<Request<()>, Status> {
        self.log_request(&request);
        Ok(request)
    }
}

/// A request logger that tracks timing and logs responses.
///
/// Use this for complete request/response logging with timing information.
#[derive(Debug)]
pub struct RequestLogger {
    config: LoggingConfig,
    metrics: InterceptorMetrics,
}

impl RequestLogger {
    /// Create a new request logger.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: LoggingConfig::default(),
            metrics: InterceptorMetrics::new(),
        }
    }

    /// Create a request logger with custom configuration.
    #[must_use]
    pub fn with_config(config: LoggingConfig) -> Self {
        Self {
            config,
            metrics: InterceptorMetrics::new(),
        }
    }

    /// Get the metrics.
    #[must_use]
    pub fn metrics(&self) -> &InterceptorMetrics {
        &self.metrics
    }

    /// Start tracking a request.
    #[must_use]
    pub fn start(&self, method: &str) -> RequestSpan {
        RequestSpan {
            method: method.to_string(),
            start: Instant::now(),
        }
    }

    /// Finish tracking a request (success).
    pub fn finish_success(&self, span: RequestSpan) {
        self.metrics.record_success();
        let elapsed = span.start.elapsed();

        if self.config.success_level == LogLevel::Off {
            return;
        }

        let msg = format!("gRPC response: {} completed in {:?}", span.method, elapsed);

        match self.config.success_level {
            LogLevel::Trace => trace!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Debug => debug!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Info => info!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Warn => warn!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Error => error!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Off => {}
        }
    }

    /// Finish tracking a request (failure).
    pub fn finish_error(&self, span: RequestSpan, error: &str) {
        self.metrics.record_failure();
        let elapsed = span.start.elapsed();

        if self.config.error_level == LogLevel::Off {
            return;
        }

        let msg = format!(
            "gRPC error: {} failed in {:?}: {}",
            span.method, elapsed, error
        );

        match self.config.error_level {
            LogLevel::Trace => trace!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Debug => debug!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Info => info!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Warn => warn!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Error => error!(target: "talos_api::grpc", "{}", msg),
            LogLevel::Off => {}
        }
    }
}

impl Default for RequestLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// A span representing an in-flight request.
#[derive(Debug)]
pub struct RequestSpan {
    method: String,
    start: Instant,
}

impl RequestSpan {
    /// Get the method name.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get the elapsed time since the request started.
    #[must_use]
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
        assert_eq!(LogLevel::Off.to_string(), "OFF");
    }

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.success_level, LogLevel::Info);
        assert_eq!(config.error_level, LogLevel::Error);
        assert!(config.log_metadata);
        assert!(config.log_path);
        assert!(config.redact_sensitive);
    }

    #[test]
    fn test_logging_config_builder() {
        let config = LoggingConfig::new()
            .with_success_level(LogLevel::Debug)
            .with_error_level(LogLevel::Warn)
            .with_metadata(false)
            .with_path(false)
            .with_redaction(false)
            .with_sensitive_header("x-custom-auth");

        assert_eq!(config.success_level, LogLevel::Debug);
        assert_eq!(config.error_level, LogLevel::Warn);
        assert!(!config.log_metadata);
        assert!(!config.log_path);
        assert!(!config.redact_sensitive);
        assert!(config
            .sensitive_headers
            .contains(&"x-custom-auth".to_string()));
    }

    #[test]
    fn test_logging_config_verbose() {
        let config = LoggingConfig::verbose();
        assert_eq!(config.success_level, LogLevel::Debug);
        assert!(config.log_metadata);
    }

    #[test]
    fn test_logging_config_quiet() {
        let config = LoggingConfig::quiet();
        assert_eq!(config.success_level, LogLevel::Off);
        assert_eq!(config.error_level, LogLevel::Warn);
        assert!(!config.log_metadata);
    }

    #[test]
    fn test_interceptor_metrics() {
        let metrics = InterceptorMetrics::new();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.successful_requests(), 0);
        assert_eq!(metrics.failed_requests(), 0);
        assert_eq!(metrics.success_rate(), 1.0); // No requests = 100% success

        metrics.record_success();
        metrics.record_success();
        metrics.record_failure();

        assert_eq!(metrics.total_requests(), 3);
        assert_eq!(metrics.successful_requests(), 2);
        assert_eq!(metrics.failed_requests(), 1);
        assert!((metrics.success_rate() - 0.666_666_666_666_666_6).abs() < 0.001);
    }

    #[test]
    fn test_interceptor_metrics_reset() {
        let metrics = InterceptorMetrics::new();
        metrics.record_success();
        metrics.record_failure();
        metrics.reset();

        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.successful_requests(), 0);
        assert_eq!(metrics.failed_requests(), 0);
    }

    #[test]
    fn test_logging_interceptor_default() {
        let interceptor = LoggingInterceptor::default();
        assert_eq!(interceptor.config().success_level, LogLevel::Info);
    }

    #[test]
    fn test_request_logger() {
        let logger = RequestLogger::new();
        let span = logger.start("Version");

        assert_eq!(span.method(), "Version");
        assert!(span.elapsed() < std::time::Duration::from_secs(1));

        logger.finish_success(span);
        assert_eq!(logger.metrics().total_requests(), 1);
        assert_eq!(logger.metrics().successful_requests(), 1);
    }

    #[test]
    fn test_request_logger_error() {
        let logger = RequestLogger::with_config(LoggingConfig::quiet());
        let span = logger.start("ApplyConfiguration");

        logger.finish_error(span, "Permission denied");
        assert_eq!(logger.metrics().total_requests(), 1);
        assert_eq!(logger.metrics().failed_requests(), 1);
    }

    #[test]
    fn test_request_span() {
        let span = RequestSpan {
            method: "test".to_string(),
            start: Instant::now(),
        };

        assert_eq!(span.method(), "test");
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(span.elapsed() >= std::time::Duration::from_millis(1));
    }
}
