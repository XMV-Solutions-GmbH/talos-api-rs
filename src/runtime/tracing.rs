// SPDX-License-Identifier: MIT OR Apache-2.0

//! OpenTelemetry tracing integration for the Talos API client.
//!
//! This module provides OpenTelemetry-compatible distributed tracing for
//! monitoring gRPC requests to Talos nodes.
//!
//! # Overview
//!
//! The tracing integration supports:
//! - Span creation for each gRPC request
//! - W3C Trace Context propagation
//! - Request/response attributes
//! - Error tracking
//!
//! # Usage with `tracing` Crate
//!
//! This module integrates with the standard Rust `tracing` ecosystem.
//! To use OpenTelemetry, configure an OpenTelemetry subscriber:
//!
//! ```rust,ignore
//! use tracing_subscriber::prelude::*;
//! use opentelemetry::sdk::trace::TracerProvider;
//! use opentelemetry_otlp::WithExportConfig;
//!
//! // Configure OTLP exporter
//! let exporter = opentelemetry_otlp::new_exporter()
//!     .tonic()
//!     .with_endpoint("http://localhost:4317");
//!
//! let provider = TracerProvider::builder()
//!     .with_batch_exporter(exporter.build_span_exporter()?)
//!     .build();
//!
//! opentelemetry::global::set_tracer_provider(provider);
//!
//! // Configure tracing subscriber with OpenTelemetry layer
//! let otel_layer = tracing_opentelemetry::layer()
//!     .with_tracer(opentelemetry::global::tracer("talos-client"));
//!
//! tracing_subscriber::registry()
//!     .with(otel_layer)
//!     .with(tracing_subscriber::fmt::layer())
//!     .init();
//! ```
//!
//! # Example: Manual Span Creation
//!
//! ```rust
//! use talos_api_rs::runtime::tracing::TalosSpan;
//! use std::time::Duration;
//!
//! // Create a span for a Talos API call
//! let span = TalosSpan::new("Version", "10.0.0.1:50000");
//!
//! // Record success
//! span.record_success(Duration::from_millis(42));
//!
//! // Or record error
//! // span.record_error("Connection refused");
//! ```
//!
//! # Semantic Conventions
//!
//! Spans follow OpenTelemetry semantic conventions:
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `rpc.system` | Always "grpc" |
//! | `rpc.service` | Talos service name (e.g., "machine.MachineService") |
//! | `rpc.method` | Method name (e.g., "Version") |
//! | `server.address` | Target endpoint |
//! | `rpc.grpc.status_code` | gRPC status code |
//!
//! # Note on Dependencies
//!
//! This module provides helpers for creating properly-attributed spans.
//! To actually export traces to an OpenTelemetry backend, you need to:
//!
//! 1. Add `opentelemetry` and `tracing-opentelemetry` to your dependencies
//! 2. Configure an OTLP exporter (e.g., `opentelemetry-otlp`)
//! 3. Set up a tracing subscriber with the OpenTelemetry layer
//!
//! The library itself only depends on `tracing`, keeping the dependency
//! footprint minimal for users who don't need distributed tracing.

use std::time::{Duration, Instant};
use tracing::{field, info_span, Span};

/// Configuration for OpenTelemetry tracing.
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name for traces
    pub service_name: String,
    /// Whether to record request payloads (may contain sensitive data)
    pub record_payloads: bool,
    /// Whether to record response payloads
    pub record_responses: bool,
    /// Maximum payload size to record (in bytes)
    pub max_payload_size: usize,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "talos-client".to_string(),
            record_payloads: false,
            record_responses: false,
            max_payload_size: 4096,
        }
    }
}

impl TracingConfig {
    /// Create a new builder for `TracingConfig`.
    pub fn builder() -> TracingConfigBuilder {
        TracingConfigBuilder::default()
    }
}

/// Builder for `TracingConfig`.
#[derive(Debug, Default)]
pub struct TracingConfigBuilder {
    service_name: Option<String>,
    record_payloads: Option<bool>,
    record_responses: Option<bool>,
    max_payload_size: Option<usize>,
}

impl TracingConfigBuilder {
    /// Set the service name for traces.
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// Enable recording of request payloads.
    pub fn record_payloads(mut self, enabled: bool) -> Self {
        self.record_payloads = Some(enabled);
        self
    }

    /// Enable recording of response payloads.
    pub fn record_responses(mut self, enabled: bool) -> Self {
        self.record_responses = Some(enabled);
        self
    }

    /// Set maximum payload size to record.
    pub fn max_payload_size(mut self, size: usize) -> Self {
        self.max_payload_size = Some(size);
        self
    }

    /// Build the `TracingConfig`.
    pub fn build(self) -> TracingConfig {
        let default = TracingConfig::default();
        TracingConfig {
            service_name: self.service_name.unwrap_or(default.service_name),
            record_payloads: self.record_payloads.unwrap_or(default.record_payloads),
            record_responses: self.record_responses.unwrap_or(default.record_responses),
            max_payload_size: self.max_payload_size.unwrap_or(default.max_payload_size),
        }
    }
}

/// A span for a Talos API call with OpenTelemetry attributes.
#[derive(Debug)]
pub struct TalosSpan {
    span: Span,
    start: Instant,
    method: String,
    endpoint: String,
}

impl TalosSpan {
    /// Create a new span for a Talos API call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use talos_api_rs::runtime::tracing::TalosSpan;
    ///
    /// let span = TalosSpan::new("Version", "10.0.0.1:50000");
    /// ```
    pub fn new(method: &str, endpoint: &str) -> Self {
        let span = info_span!(
            "talos.grpc",
            rpc.system = "grpc",
            rpc.service = "talos.machine.MachineService",
            rpc.method = %method,
            server.address = %endpoint,
            rpc.grpc.status_code = field::Empty,
            otel.status_code = field::Empty,
            error.message = field::Empty,
            duration_ms = field::Empty,
        );

        Self {
            span,
            start: Instant::now(),
            method: method.to_string(),
            endpoint: endpoint.to_string(),
        }
    }

    /// Create a span with a specific service name.
    pub fn with_service(method: &str, service: &str, endpoint: &str) -> Self {
        let span = info_span!(
            "talos.grpc",
            rpc.system = "grpc",
            rpc.service = %service,
            rpc.method = %method,
            server.address = %endpoint,
            rpc.grpc.status_code = field::Empty,
            otel.status_code = field::Empty,
            error.message = field::Empty,
            duration_ms = field::Empty,
        );

        Self {
            span,
            start: Instant::now(),
            method: method.to_string(),
            endpoint: endpoint.to_string(),
        }
    }

    /// Get the underlying `tracing::Span`.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Get the method name.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get the endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get elapsed time since span creation.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Record a successful response.
    pub fn record_success(&self, duration: Duration) {
        self.span.record("rpc.grpc.status_code", 0i64); // OK
        self.span.record("otel.status_code", "OK");
        self.span.record("duration_ms", duration.as_millis() as i64);
    }

    /// Record an error response.
    pub fn record_error(&self, error: &str) {
        let duration = self.start.elapsed();
        self.span.record("rpc.grpc.status_code", 2i64); // UNKNOWN
        self.span.record("otel.status_code", "ERROR");
        self.span.record("error.message", error);
        self.span.record("duration_ms", duration.as_millis() as i64);
    }

    /// Record a gRPC status code.
    pub fn record_grpc_status(&self, code: i32) {
        self.span.record("rpc.grpc.status_code", code as i64);
        let status = if code == 0 { "OK" } else { "ERROR" };
        self.span.record("otel.status_code", status);
        self.span
            .record("duration_ms", self.start.elapsed().as_millis() as i64);
    }

    /// Enter the span context for async work.
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self.span.enter()
    }
}

/// Helper macro for creating instrumented async functions.
///
/// # Example
///
/// ```rust,ignore
/// use talos_api_rs::instrument_talos;
///
/// async fn call_version(client: &TalosClient, endpoint: &str) -> Result<Version, Error> {
///     instrument_talos!("Version", endpoint, {
///         client.version().version(()).await
///     })
/// }
/// ```
#[macro_export]
macro_rules! instrument_talos {
    ($method:expr, $endpoint:expr, $body:expr) => {{
        let span = $crate::runtime::tracing::TalosSpan::new($method, $endpoint);
        let _guard = span.enter();
        let start = std::time::Instant::now();
        let result = $body;
        let duration = start.elapsed();
        match &result {
            Ok(_) => span.record_success(duration),
            Err(e) => span.record_error(&format!("{}", e)),
        }
        result
    }};
}

/// Span factory for creating consistent spans across the client.
#[derive(Debug, Clone)]
pub struct SpanFactory {
    config: TracingConfig,
}

impl SpanFactory {
    /// Create a new span factory with the given configuration.
    pub fn new(config: TracingConfig) -> Self {
        Self { config }
    }

    /// Create a span for a Talos API call.
    pub fn create_span(&self, method: &str, endpoint: &str) -> TalosSpan {
        TalosSpan::with_service(method, "talos.machine.MachineService", endpoint)
    }

    /// Create a span for an etcd API call.
    pub fn create_etcd_span(&self, method: &str, endpoint: &str) -> TalosSpan {
        TalosSpan::with_service(method, "talos.machine.MachineService/Etcd", endpoint)
    }

    /// Get the configuration.
    pub fn config(&self) -> &TracingConfig {
        &self.config
    }
}

impl Default for SpanFactory {
    fn default() -> Self {
        Self::new(TracingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert_eq!(config.service_name, "talos-client");
        assert!(!config.record_payloads);
        assert!(!config.record_responses);
        assert_eq!(config.max_payload_size, 4096);
    }

    #[test]
    fn test_tracing_config_builder() {
        let config = TracingConfig::builder()
            .service_name("my-service")
            .record_payloads(true)
            .record_responses(true)
            .max_payload_size(8192)
            .build();

        assert_eq!(config.service_name, "my-service");
        assert!(config.record_payloads);
        assert!(config.record_responses);
        assert_eq!(config.max_payload_size, 8192);
    }

    #[test]
    fn test_talos_span_new() {
        let span = TalosSpan::new("Version", "10.0.0.1:50000");
        assert_eq!(span.method(), "Version");
        assert_eq!(span.endpoint(), "10.0.0.1:50000");
    }

    #[test]
    fn test_talos_span_with_service() {
        let span = TalosSpan::with_service(
            "EtcdMemberList",
            "talos.machine.MachineService/Etcd",
            "10.0.0.1:50000",
        );
        assert_eq!(span.method(), "EtcdMemberList");
    }

    #[test]
    fn test_talos_span_record_success() {
        let span = TalosSpan::new("Version", "10.0.0.1:50000");
        span.record_success(Duration::from_millis(42));
        // Span should not panic
    }

    #[test]
    fn test_talos_span_record_error() {
        let span = TalosSpan::new("Version", "10.0.0.1:50000");
        span.record_error("Connection refused");
        // Span should not panic
    }

    #[test]
    fn test_talos_span_record_grpc_status() {
        let span = TalosSpan::new("Version", "10.0.0.1:50000");
        span.record_grpc_status(0); // OK
        span.record_grpc_status(14); // UNAVAILABLE
                                     // Span should not panic
    }

    #[test]
    fn test_span_factory_new() {
        let config = TracingConfig::builder()
            .service_name("test-service")
            .build();
        let factory = SpanFactory::new(config);
        assert_eq!(factory.config().service_name, "test-service");
    }

    #[test]
    fn test_span_factory_create_span() {
        let factory = SpanFactory::default();
        let span = factory.create_span("Version", "10.0.0.1:50000");
        assert_eq!(span.method(), "Version");
    }

    #[test]
    fn test_span_factory_create_etcd_span() {
        let factory = SpanFactory::default();
        let span = factory.create_etcd_span("EtcdMemberList", "10.0.0.1:50000");
        assert_eq!(span.method(), "EtcdMemberList");
    }

    #[test]
    fn test_talos_span_elapsed() {
        let span = TalosSpan::new("Version", "10.0.0.1:50000");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = span.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }
}
