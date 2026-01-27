// SPDX-License-Identifier: MIT OR Apache-2.0

//! Example: Prometheus-compatible Metrics for Talos Client
//!
//! This example demonstrates how to collect and expose Prometheus-compatible
//! metrics from the Talos API client.
//!
//! # Running
//!
//! ```bash
//! cargo run --example monitoring_metrics
//! ```

use std::sync::Arc;
use std::time::Duration;
use talos_api_rs::runtime::{MetricsCollector, MetricsConfig};

fn main() {
    // Configure metrics with custom settings
    let metrics_config = MetricsConfig::builder()
        .namespace("talos")
        .endpoint_label(true)
        .method_label(true)
        .histogram_buckets(vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ])
        .build();

    let metrics = Arc::new(MetricsCollector::new(metrics_config));

    println!("📊 Talos Client Metrics Demo\n");
    println!("Simulating API calls to multiple endpoints...\n");

    // Simulate API calls to different endpoints
    let endpoints = vec!["10.0.0.1:50000", "10.0.0.2:50000", "10.0.0.3:50000"];

    // Simulate Version API calls
    for endpoint in &endpoints {
        metrics.record_request("Version", endpoint, true, Duration::from_millis(12));
        metrics.record_request("Version", endpoint, true, Duration::from_millis(15));
    }

    // Simulate Hostname API calls (one failure)
    metrics.record_request("Hostname", endpoints[0], true, Duration::from_millis(8));
    metrics.record_request("Hostname", endpoints[1], false, Duration::from_millis(5000));
    metrics.record_request("Hostname", endpoints[2], true, Duration::from_millis(10));

    // Simulate ServiceList API calls
    for endpoint in &endpoints {
        metrics.record_request("ServiceList", endpoint, true, Duration::from_millis(45));
    }

    // Simulate SystemStat API calls with varying latencies
    metrics.record_request("SystemStat", endpoints[0], true, Duration::from_millis(25));
    metrics.record_request("SystemStat", endpoints[1], true, Duration::from_millis(150));
    metrics.record_request("SystemStat", endpoints[2], true, Duration::from_millis(75));

    // Simulate some slower operations
    metrics.record_request(
        "ApplyConfiguration",
        endpoints[0],
        true,
        Duration::from_millis(2500),
    );
    metrics.record_request("Kubeconfig", endpoints[0], true, Duration::from_millis(500));

    // Update circuit breaker state (simulating normal operation)
    metrics.set_circuit_breaker_state(0); // Closed

    // Update pool metrics
    metrics.set_pool_endpoints(3, 3);

    // Simulate one failover
    metrics.record_pool_failover();

    // Print summary
    print_metrics(&metrics);

    // Print Prometheus text format
    println!("\n{}", "=".repeat(70));
    println!("Prometheus Text Format (for scraping by Prometheus server):");
    println!("{}", "=".repeat(70));
    println!("{}", metrics.to_prometheus_text());

    // Show how to get metrics programmatically
    println!("{}", "=".repeat(70));
    println!("Programmatic Access (MetricsSnapshot):");
    println!("{}", "=".repeat(70));
    let snapshot = metrics.snapshot();
    println!("{:#?}", snapshot);
}

fn print_metrics(metrics: &MetricsCollector) {
    let snapshot = metrics.snapshot();

    println!("📊 Metrics Summary");
    println!("{}", "-".repeat(50));
    println!("Total Requests:           {}", snapshot.total_requests);
    println!("Successful Requests:      {}", snapshot.successful_requests);
    println!("Failed Requests:          {}", snapshot.failed_requests);
    if snapshot.total_requests > 0 {
        println!(
            "Success Rate:             {:.1}%",
            (snapshot.successful_requests as f64 / snapshot.total_requests as f64) * 100.0
        );
    }
    println!();
    println!(
        "Circuit Breaker State:    {} (0=closed, 1=half-open, 2=open)",
        snapshot.circuit_breaker_state
    );
    println!(
        "CB Rejections:            {}",
        snapshot.circuit_breaker_rejections
    );
    println!();
    println!(
        "Pool Endpoints:           {}/{} healthy",
        snapshot.pool_healthy_endpoints, snapshot.pool_total_endpoints
    );
    println!("Pool Failovers:           {}", snapshot.pool_failovers);
    println!();
    println!(
        "Uptime:                   {:.3}s",
        snapshot.uptime.as_secs_f64()
    );
}
