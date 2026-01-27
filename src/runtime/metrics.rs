// SPDX-License-Identifier: MIT OR Apache-2.0

//! Prometheus-compatible metrics for the Talos API client.
//!
//! This module provides a metrics collection system that exposes Prometheus-compatible
//! metrics for monitoring gRPC requests to Talos nodes.
//!
//! # Features
//!
//! - Request counters (total, success, failure)
//! - Response time histograms
//! - Per-method and per-endpoint metrics
//! - Circuit breaker state metrics
//! - Connection pool metrics
//!
//! # Example
//!
//! ```rust
//! use talos_api_rs::runtime::metrics::{MetricsCollector, MetricsConfig};
//!
//! let config = MetricsConfig::builder()
//!     .namespace("talos")
//!     .endpoint_label(true)
//!     .method_label(true)
//!     .build();
//!
//! let metrics = MetricsCollector::new(config);
//!
//! // Record a request
//! metrics.record_request("Version", "10.0.0.1:50000", true, std::time::Duration::from_millis(42));
//!
//! // Get Prometheus text format
//! let output = metrics.to_prometheus_text();
//! println!("{}", output);
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Configuration for the metrics collector.
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Metric namespace prefix (e.g., "talos" -> "talos_requests_total")
    pub namespace: String,
    /// Include endpoint as a label
    pub endpoint_label: bool,
    /// Include method as a label
    pub method_label: bool,
    /// Histogram buckets for response time (in seconds)
    pub histogram_buckets: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            namespace: "talos_client".to_string(),
            endpoint_label: true,
            method_label: true,
            histogram_buckets: vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        }
    }
}

impl MetricsConfig {
    /// Create a new builder for `MetricsConfig`.
    pub fn builder() -> MetricsConfigBuilder {
        MetricsConfigBuilder::default()
    }
}

/// Builder for `MetricsConfig`.
#[derive(Debug, Default)]
pub struct MetricsConfigBuilder {
    namespace: Option<String>,
    endpoint_label: Option<bool>,
    method_label: Option<bool>,
    histogram_buckets: Option<Vec<f64>>,
}

impl MetricsConfigBuilder {
    /// Set the metric namespace prefix.
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Enable or disable endpoint labels.
    pub fn endpoint_label(mut self, enabled: bool) -> Self {
        self.endpoint_label = Some(enabled);
        self
    }

    /// Enable or disable method labels.
    pub fn method_label(mut self, enabled: bool) -> Self {
        self.method_label = Some(enabled);
        self
    }

    /// Set histogram buckets for response time (in seconds).
    pub fn histogram_buckets(mut self, buckets: Vec<f64>) -> Self {
        self.histogram_buckets = Some(buckets);
        self
    }

    /// Build the `MetricsConfig`.
    pub fn build(self) -> MetricsConfig {
        let default = MetricsConfig::default();
        MetricsConfig {
            namespace: self.namespace.unwrap_or(default.namespace),
            endpoint_label: self.endpoint_label.unwrap_or(default.endpoint_label),
            method_label: self.method_label.unwrap_or(default.method_label),
            histogram_buckets: self.histogram_buckets.unwrap_or(default.histogram_buckets),
        }
    }
}

/// Labels for a metric sample.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Labels {
    method: Option<String>,
    endpoint: Option<String>,
    status: String,
}

/// A single histogram with bucket counters.
#[derive(Debug)]
struct Histogram {
    buckets: Vec<f64>,
    counts: Vec<AtomicU64>,
    sum: AtomicU64, // Store as nanoseconds
    count: AtomicU64,
}

impl Histogram {
    fn new(buckets: Vec<f64>) -> Self {
        let counts = buckets.iter().map(|_| AtomicU64::new(0)).collect();
        Self {
            buckets,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    fn observe(&self, value_secs: f64) {
        // Update bucket counters (cumulative)
        for (i, bucket) in self.buckets.iter().enumerate() {
            if value_secs <= *bucket {
                for j in i..self.buckets.len() {
                    self.counts[j].fetch_add(1, Ordering::Relaxed);
                }
                break;
            }
        }

        // If value exceeds all buckets, only +Inf is incremented (handled separately)
        self.sum
            .fetch_add((value_secs * 1_000_000_000.0) as u64, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    fn sum_secs(&self) -> f64 {
        self.sum.load(Ordering::Relaxed) as f64 / 1_000_000_000.0
    }

    fn total_count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

/// Key for histogram lookup (method, endpoint).
type HistogramKey = (Option<String>, Option<String>);

/// Thread-safe metrics collector for the Talos client.
#[derive(Debug)]
pub struct MetricsCollector {
    config: MetricsConfig,
    /// Counter: requests_total{method, endpoint, status}
    requests_total: RwLock<HashMap<Labels, AtomicU64>>,
    /// Histogram: request_duration_seconds{method, endpoint}
    request_duration: RwLock<HashMap<HistogramKey, Histogram>>,
    /// Gauge: circuit_breaker_state (0=closed, 1=half-open, 2=open)
    circuit_breaker_state: AtomicU64,
    /// Counter: circuit_breaker_rejections_total
    circuit_breaker_rejections: AtomicU64,
    /// Gauge: connection_pool_healthy_endpoints
    pool_healthy_endpoints: AtomicU64,
    /// Gauge: connection_pool_total_endpoints
    pool_total_endpoints: AtomicU64,
    /// Counter: connection_pool_failovers_total
    pool_failovers: AtomicU64,
    /// Start time for uptime metric
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector with the given configuration.
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            requests_total: RwLock::new(HashMap::new()),
            request_duration: RwLock::new(HashMap::new()),
            circuit_breaker_state: AtomicU64::new(0),
            circuit_breaker_rejections: AtomicU64::new(0),
            pool_healthy_endpoints: AtomicU64::new(0),
            pool_total_endpoints: AtomicU64::new(0),
            pool_failovers: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Create a new metrics collector with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(MetricsConfig::default())
    }

    /// Record a completed request.
    pub fn record_request(&self, method: &str, endpoint: &str, success: bool, duration: Duration) {
        let labels = Labels {
            method: if self.config.method_label {
                Some(method.to_string())
            } else {
                None
            },
            endpoint: if self.config.endpoint_label {
                Some(endpoint.to_string())
            } else {
                None
            },
            status: if success { "success" } else { "error" }.to_string(),
        };

        // Update counter
        {
            let counters = self.requests_total.read().expect("lock poisoned");
            if let Some(counter) = counters.get(&labels) {
                counter.fetch_add(1, Ordering::Relaxed);
            } else {
                drop(counters);
                let mut counters = self.requests_total.write().expect("lock poisoned");
                counters
                    .entry(labels)
                    .or_insert_with(|| AtomicU64::new(0))
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        // Update histogram
        let hist_key = (
            if self.config.method_label {
                Some(method.to_string())
            } else {
                None
            },
            if self.config.endpoint_label {
                Some(endpoint.to_string())
            } else {
                None
            },
        );

        {
            let histograms = self.request_duration.read().expect("lock poisoned");
            if let Some(hist) = histograms.get(&hist_key) {
                hist.observe(duration.as_secs_f64());
            } else {
                drop(histograms);
                let mut histograms = self.request_duration.write().expect("lock poisoned");
                let hist = histograms
                    .entry(hist_key)
                    .or_insert_with(|| Histogram::new(self.config.histogram_buckets.clone()));
                hist.observe(duration.as_secs_f64());
            }
        }
    }

    /// Update circuit breaker state (0=closed, 1=half-open, 2=open).
    pub fn set_circuit_breaker_state(&self, state: u64) {
        self.circuit_breaker_state.store(state, Ordering::Relaxed);
    }

    /// Record a circuit breaker rejection.
    pub fn record_circuit_breaker_rejection(&self) {
        self.circuit_breaker_rejections
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Update connection pool metrics.
    pub fn set_pool_endpoints(&self, healthy: u64, total: u64) {
        self.pool_healthy_endpoints
            .store(healthy, Ordering::Relaxed);
        self.pool_total_endpoints.store(total, Ordering::Relaxed);
    }

    /// Record a connection pool failover.
    pub fn record_pool_failover(&self) {
        self.pool_failovers.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the total number of requests.
    pub fn total_requests(&self) -> u64 {
        let counters = self.requests_total.read().expect("lock poisoned");
        counters.values().map(|c| c.load(Ordering::Relaxed)).sum()
    }

    /// Get the number of successful requests.
    pub fn successful_requests(&self) -> u64 {
        let counters = self.requests_total.read().expect("lock poisoned");
        counters
            .iter()
            .filter(|(labels, _)| labels.status == "success")
            .map(|(_, c)| c.load(Ordering::Relaxed))
            .sum()
    }

    /// Get the number of failed requests.
    pub fn failed_requests(&self) -> u64 {
        let counters = self.requests_total.read().expect("lock poisoned");
        counters
            .iter()
            .filter(|(labels, _)| labels.status == "error")
            .map(|(_, c)| c.load(Ordering::Relaxed))
            .sum()
    }

    /// Get client uptime.
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Export metrics in Prometheus text format.
    pub fn to_prometheus_text(&self) -> String {
        let mut output = String::new();
        let ns = &self.config.namespace;

        // Request counter
        output.push_str(&format!(
            "# HELP {ns}_requests_total Total number of requests\n"
        ));
        output.push_str(&format!("# TYPE {ns}_requests_total counter\n"));
        {
            let counters = self.requests_total.read().expect("lock poisoned");
            for (labels, count) in counters.iter() {
                let mut label_parts = vec![format!("status=\"{}\"", labels.status)];
                if let Some(ref method) = labels.method {
                    label_parts.insert(0, format!("method=\"{method}\""));
                }
                if let Some(ref endpoint) = labels.endpoint {
                    label_parts.insert(1, format!("endpoint=\"{endpoint}\""));
                }
                let label_str = label_parts.join(",");
                output.push_str(&format!(
                    "{ns}_requests_total{{{label_str}}} {}\n",
                    count.load(Ordering::Relaxed)
                ));
            }
        }
        output.push('\n');

        // Request duration histogram
        output.push_str(&format!(
            "# HELP {ns}_request_duration_seconds Request duration in seconds\n"
        ));
        output.push_str(&format!("# TYPE {ns}_request_duration_seconds histogram\n"));
        {
            let histograms = self.request_duration.read().expect("lock poisoned");
            for ((method, endpoint), hist) in histograms.iter() {
                let base_labels = match (method, endpoint) {
                    (Some(m), Some(e)) => format!("method=\"{m}\",endpoint=\"{e}\""),
                    (Some(m), None) => format!("method=\"{m}\""),
                    (None, Some(e)) => format!("endpoint=\"{e}\""),
                    (None, None) => String::new(),
                };

                // Bucket values
                for (i, bucket) in hist.buckets.iter().enumerate() {
                    let count = hist.counts[i].load(Ordering::Relaxed);
                    let le = if *bucket == f64::INFINITY {
                        "+Inf".to_string()
                    } else {
                        format!("{bucket}")
                    };
                    if base_labels.is_empty() {
                        output.push_str(&format!(
                            "{ns}_request_duration_seconds_bucket{{le=\"{le}\"}} {count}\n"
                        ));
                    } else {
                        output.push_str(&format!(
                            "{ns}_request_duration_seconds_bucket{{{base_labels},le=\"{le}\"}} {count}\n"
                        ));
                    }
                }

                // +Inf bucket (total count)
                let inf_count = hist.total_count();
                if base_labels.is_empty() {
                    output.push_str(&format!(
                        "{ns}_request_duration_seconds_bucket{{le=\"+Inf\"}} {inf_count}\n"
                    ));
                } else {
                    output.push_str(&format!(
                        "{ns}_request_duration_seconds_bucket{{{base_labels},le=\"+Inf\"}} {inf_count}\n"
                    ));
                }

                // Sum and count
                if base_labels.is_empty() {
                    output.push_str(&format!(
                        "{ns}_request_duration_seconds_sum {}\n",
                        hist.sum_secs()
                    ));
                    output.push_str(&format!(
                        "{ns}_request_duration_seconds_count {inf_count}\n"
                    ));
                } else {
                    output.push_str(&format!(
                        "{ns}_request_duration_seconds_sum{{{base_labels}}} {}\n",
                        hist.sum_secs()
                    ));
                    output.push_str(&format!(
                        "{ns}_request_duration_seconds_count{{{base_labels}}} {inf_count}\n"
                    ));
                }
            }
        }
        output.push('\n');

        // Circuit breaker metrics
        output.push_str(&format!(
            "# HELP {ns}_circuit_breaker_state Circuit breaker state (0=closed, 1=half-open, 2=open)\n"
        ));
        output.push_str(&format!("# TYPE {ns}_circuit_breaker_state gauge\n"));
        output.push_str(&format!(
            "{ns}_circuit_breaker_state {}\n\n",
            self.circuit_breaker_state.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP {ns}_circuit_breaker_rejections_total Requests rejected by circuit breaker\n"
        ));
        output.push_str(&format!(
            "# TYPE {ns}_circuit_breaker_rejections_total counter\n"
        ));
        output.push_str(&format!(
            "{ns}_circuit_breaker_rejections_total {}\n\n",
            self.circuit_breaker_rejections.load(Ordering::Relaxed)
        ));

        // Connection pool metrics
        output.push_str(&format!(
            "# HELP {ns}_pool_healthy_endpoints Number of healthy endpoints in pool\n"
        ));
        output.push_str(&format!("# TYPE {ns}_pool_healthy_endpoints gauge\n"));
        output.push_str(&format!(
            "{ns}_pool_healthy_endpoints {}\n\n",
            self.pool_healthy_endpoints.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP {ns}_pool_total_endpoints Total endpoints in pool\n"
        ));
        output.push_str(&format!("# TYPE {ns}_pool_total_endpoints gauge\n"));
        output.push_str(&format!(
            "{ns}_pool_total_endpoints {}\n\n",
            self.pool_total_endpoints.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP {ns}_pool_failovers_total Connection pool failover events\n"
        ));
        output.push_str(&format!("# TYPE {ns}_pool_failovers_total counter\n"));
        output.push_str(&format!(
            "{ns}_pool_failovers_total {}\n\n",
            self.pool_failovers.load(Ordering::Relaxed)
        ));

        // Uptime
        output.push_str(&format!(
            "# HELP {ns}_uptime_seconds Client uptime in seconds\n"
        ));
        output.push_str(&format!("# TYPE {ns}_uptime_seconds gauge\n"));
        output.push_str(&format!(
            "{ns}_uptime_seconds {}\n",
            self.uptime().as_secs_f64()
        ));

        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Snapshot of current metrics for programmatic access.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Circuit breaker state (0=closed, 1=half-open, 2=open)
    pub circuit_breaker_state: u64,
    /// Circuit breaker rejection count
    pub circuit_breaker_rejections: u64,
    /// Healthy endpoints in pool
    pub pool_healthy_endpoints: u64,
    /// Total endpoints in pool
    pub pool_total_endpoints: u64,
    /// Pool failover count
    pub pool_failovers: u64,
    /// Client uptime
    pub uptime: Duration,
}

impl MetricsCollector {
    /// Get a snapshot of current metrics.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests(),
            successful_requests: self.successful_requests(),
            failed_requests: self.failed_requests(),
            circuit_breaker_state: self.circuit_breaker_state.load(Ordering::Relaxed),
            circuit_breaker_rejections: self.circuit_breaker_rejections.load(Ordering::Relaxed),
            pool_healthy_endpoints: self.pool_healthy_endpoints.load(Ordering::Relaxed),
            pool_total_endpoints: self.pool_total_endpoints.load(Ordering::Relaxed),
            pool_failovers: self.pool_failovers.load(Ordering::Relaxed),
            uptime: self.uptime(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert_eq!(config.namespace, "talos_client");
        assert!(config.endpoint_label);
        assert!(config.method_label);
        assert!(!config.histogram_buckets.is_empty());
    }

    #[test]
    fn test_metrics_config_builder() {
        let config = MetricsConfig::builder()
            .namespace("my_talos")
            .endpoint_label(false)
            .method_label(true)
            .histogram_buckets(vec![0.1, 0.5, 1.0])
            .build();

        assert_eq!(config.namespace, "my_talos");
        assert!(!config.endpoint_label);
        assert!(config.method_label);
        assert_eq!(config.histogram_buckets, vec![0.1, 0.5, 1.0]);
    }

    #[test]
    fn test_record_request_success() {
        let metrics = MetricsCollector::with_defaults();
        metrics.record_request("Version", "10.0.0.1:50000", true, Duration::from_millis(42));

        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.successful_requests(), 1);
        assert_eq!(metrics.failed_requests(), 0);
    }

    #[test]
    fn test_record_request_failure() {
        let metrics = MetricsCollector::with_defaults();
        metrics.record_request(
            "Version",
            "10.0.0.1:50000",
            false,
            Duration::from_millis(100),
        );

        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.successful_requests(), 0);
        assert_eq!(metrics.failed_requests(), 1);
    }

    #[test]
    fn test_multiple_requests() {
        let metrics = MetricsCollector::with_defaults();
        metrics.record_request("Version", "10.0.0.1:50000", true, Duration::from_millis(10));
        metrics.record_request(
            "Hostname",
            "10.0.0.1:50000",
            true,
            Duration::from_millis(20),
        );
        metrics.record_request(
            "Version",
            "10.0.0.2:50000",
            false,
            Duration::from_millis(30),
        );

        assert_eq!(metrics.total_requests(), 3);
        assert_eq!(metrics.successful_requests(), 2);
        assert_eq!(metrics.failed_requests(), 1);
    }

    #[test]
    fn test_circuit_breaker_metrics() {
        let metrics = MetricsCollector::with_defaults();

        metrics.set_circuit_breaker_state(0);
        assert_eq!(metrics.circuit_breaker_state.load(Ordering::Relaxed), 0);

        metrics.set_circuit_breaker_state(2);
        assert_eq!(metrics.circuit_breaker_state.load(Ordering::Relaxed), 2);

        metrics.record_circuit_breaker_rejection();
        metrics.record_circuit_breaker_rejection();
        assert_eq!(
            metrics.circuit_breaker_rejections.load(Ordering::Relaxed),
            2
        );
    }

    #[test]
    fn test_pool_metrics() {
        let metrics = MetricsCollector::with_defaults();

        metrics.set_pool_endpoints(3, 5);
        assert_eq!(metrics.pool_healthy_endpoints.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.pool_total_endpoints.load(Ordering::Relaxed), 5);

        metrics.record_pool_failover();
        assert_eq!(metrics.pool_failovers.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_snapshot() {
        let metrics = MetricsCollector::with_defaults();
        metrics.record_request("Version", "10.0.0.1:50000", true, Duration::from_millis(10));
        metrics.set_circuit_breaker_state(1);
        metrics.set_pool_endpoints(2, 3);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 1);
        assert_eq!(snapshot.successful_requests, 1);
        assert_eq!(snapshot.circuit_breaker_state, 1);
        assert_eq!(snapshot.pool_healthy_endpoints, 2);
        assert_eq!(snapshot.pool_total_endpoints, 3);
    }

    #[test]
    fn test_prometheus_text_format() {
        let metrics = MetricsCollector::new(MetricsConfig {
            namespace: "test".to_string(),
            endpoint_label: false,
            method_label: true,
            histogram_buckets: vec![0.1, 1.0],
        });

        metrics.record_request("Version", "10.0.0.1:50000", true, Duration::from_millis(50));

        let output = metrics.to_prometheus_text();

        assert!(output.contains("# HELP test_requests_total"));
        assert!(output.contains("# TYPE test_requests_total counter"));
        assert!(output.contains("test_requests_total{method=\"Version\",status=\"success\"}"));
        assert!(output.contains("# HELP test_request_duration_seconds"));
        assert!(output.contains("test_request_duration_seconds_bucket"));
        assert!(output.contains("test_circuit_breaker_state"));
        assert!(output.contains("test_pool_healthy_endpoints"));
        assert!(output.contains("test_uptime_seconds"));
    }

    #[test]
    fn test_histogram_buckets() {
        let hist = Histogram::new(vec![0.01, 0.1, 1.0]);

        // Value of 0.05 should increment 0.1 and 1.0 buckets
        hist.observe(0.05);

        assert_eq!(hist.counts[0].load(Ordering::Relaxed), 0); // 0.01 bucket
        assert_eq!(hist.counts[1].load(Ordering::Relaxed), 1); // 0.1 bucket
        assert_eq!(hist.counts[2].load(Ordering::Relaxed), 1); // 1.0 bucket
        assert_eq!(hist.total_count(), 1);
    }

    #[test]
    fn test_metrics_without_labels() {
        let config = MetricsConfig::builder()
            .endpoint_label(false)
            .method_label(false)
            .build();

        let metrics = MetricsCollector::new(config);
        metrics.record_request("Version", "10.0.0.1:50000", true, Duration::from_millis(10));

        let output = metrics.to_prometheus_text();
        assert!(output.contains("status=\"success\""));
        assert!(!output.contains("method=\"Version\""));
        assert!(!output.contains("endpoint="));
    }

    #[test]
    fn test_uptime_increases() {
        let metrics = MetricsCollector::with_defaults();
        let uptime1 = metrics.uptime();
        std::thread::sleep(Duration::from_millis(10));
        let uptime2 = metrics.uptime();
        assert!(uptime2 > uptime1);
    }
}
