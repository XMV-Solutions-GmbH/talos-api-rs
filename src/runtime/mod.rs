// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runtime utilities for resilience and observability.
//!
//! This module provides production-ready features like retry policies,
//! circuit breakers, logging interceptors, metrics, tracing, and timeouts
//! for the Talos API client.

mod circuit_breaker;
mod logging;
mod metrics;
mod retry;
pub mod tracing;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use logging::{
    InterceptorMetrics, LogLevel, LoggingConfig, LoggingInterceptor, RequestLogger, RequestSpan,
};
pub use metrics::{MetricsCollector, MetricsConfig, MetricsConfigBuilder, MetricsSnapshot};
pub use retry::{
    BackoffStrategy, CustomRetryPolicy, DefaultRetryPolicy, ExponentialBackoff, FixedBackoff,
    LinearBackoff, NoBackoff, NoRetryPolicy, RetryConfig, RetryConfigBuilder, RetryPolicy,
};
pub use tracing::{SpanFactory, TalosSpan, TracingConfig, TracingConfigBuilder};
