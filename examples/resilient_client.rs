// SPDX-License-Identifier: MIT OR Apache-2.0

//! Example: Resilient Client with Retry and Circuit Breaker
//!
//! This example demonstrates how to use the production resilience features:
//! - Connection pool with multiple endpoints
//! - Circuit breaker for failure protection
//! - Retry policies with exponential backoff
//! - Request logging with metrics

use std::time::Duration;
use talos_api_rs::{
    BackoffStrategy, CircuitBreaker, CircuitBreakerConfig, ConnectionPoolConfig,
    ExponentialBackoff, LoadBalancer, LoggingConfig, RequestLogger, RetryConfig, TalosClientConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (for demonstration)
    tracing_subscriber::fmt::init();

    // ==========================================================================
    // 1. Single Client with Timeouts
    // ==========================================================================
    println!("=== Single Client with Timeouts ===");

    let config = TalosClientConfig::builder("https://192.168.1.100:50000")
        .connect_timeout(Duration::from_secs(5))
        .request_timeout(Duration::from_secs(30))
        .keepalive(Duration::from_secs(30), Duration::from_secs(10))
        .build();

    println!("Config: {:?}", config);

    // ==========================================================================
    // 2. Connection Pool with Multiple Endpoints
    // ==========================================================================
    println!("\n=== Connection Pool Configuration ===");

    let pool_config = ConnectionPoolConfig::new(vec![
        "https://node1.talos.local:50000".to_string(),
        "https://node2.talos.local:50000".to_string(),
        "https://node3.talos.local:50000".to_string(),
    ])
    .with_load_balancer(LoadBalancer::RoundRobin)
    .with_failure_threshold(3)
    .with_recovery_threshold(2)
    .with_health_check_interval(Duration::from_secs(30))
    .with_base_config(
        TalosClientConfig::builder("ignored")
            .connect_timeout(Duration::from_secs(5))
            .request_timeout(Duration::from_secs(30))
            .build(),
    );

    println!("Pool endpoints: {:?}", pool_config.endpoints);
    println!("Load balancer: {:?}", pool_config.load_balancer);

    // Note: ConnectionPool::new() would attempt to connect, so we skip for demo
    // let pool = ConnectionPool::new(pool_config).await?;
    // let client = pool.get_client().await?;

    // ==========================================================================
    // 3. Circuit Breaker
    // ==========================================================================
    println!("\n=== Circuit Breaker ===");

    let cb_config = CircuitBreakerConfig::new()
        .with_failure_threshold(5)
        .with_success_threshold(2)
        .with_reset_timeout(Duration::from_secs(30))
        .with_half_open_max_requests(3);

    let circuit_breaker = CircuitBreaker::new(cb_config);

    println!("Initial state: {:?}", circuit_breaker.state().await);
    println!("Can execute: {}", circuit_breaker.can_execute().await);

    // Simulate some requests
    for i in 1..=3 {
        let result = circuit_breaker
            .call(|| async {
                // Simulated success
                println!("  Request {} executed", i);
                Ok::<_, talos_api_rs::TalosError>(format!("Response {}", i))
            })
            .await;
        println!("  Result: {:?}", result);
    }

    println!("After 3 successes:");
    println!("  Total calls: {}", circuit_breaker.total_calls());
    println!("  Failures: {}", circuit_breaker.total_failures());
    println!(
        "  Failure rate: {:.2}%",
        circuit_breaker.failure_rate() * 100.0
    );

    // ==========================================================================
    // 4. Retry Configuration
    // ==========================================================================
    println!("\n=== Retry Configuration ===");

    let retry_config = RetryConfig::builder()
        .max_retries(3)
        .backoff(
            ExponentialBackoff::new(Duration::from_millis(100))
                .with_max_delay(Duration::from_secs(5))
                .with_multiplier(2.0),
        )
        .build();

    println!("Max retries: {}", retry_config.max_retries);

    // Demonstrate backoff calculation
    let backoff = ExponentialBackoff::new(Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(5))
        .with_multiplier(2.0)
        .with_jitter(false);

    println!("\nBackoff delays:");
    for attempt in 0..5 {
        println!("  Attempt {}: {:?}", attempt, backoff.delay(attempt));
    }

    // ==========================================================================
    // 5. Request Logging
    // ==========================================================================
    println!("\n=== Request Logging ===");

    let logging_config = LoggingConfig::verbose();
    let logger = RequestLogger::with_config(logging_config);

    // Simulate logging some requests
    let span = logger.start("Version");
    std::thread::sleep(Duration::from_millis(10));
    logger.finish_success(span);

    let span = logger.start("ApplyConfiguration");
    std::thread::sleep(Duration::from_millis(5));
    logger.finish_error(span, "Permission denied");

    println!("Metrics:");
    println!("  Total requests: {}", logger.metrics().total_requests());
    println!("  Successful: {}", logger.metrics().successful_requests());
    println!("  Failed: {}", logger.metrics().failed_requests());
    println!(
        "  Success rate: {:.1}%",
        logger.metrics().success_rate() * 100.0
    );

    // ==========================================================================
    // 6. Putting It All Together
    // ==========================================================================
    println!("\n=== Production Pattern ===");
    println!(
        "
// In production, you would combine these features:

let pool_config = ConnectionPoolConfig::new(endpoints)
    .with_load_balancer(LoadBalancer::LeastFailures)
    .with_base_config(client_config);

let pool = ConnectionPool::new(pool_config).await?;
let circuit_breaker = CircuitBreaker::new(cb_config);
let retry = RetryConfig::builder().max_retries(3).build();
let logger = RequestLogger::new();

// Execute with all protections
let span = logger.start(\"ApplyConfiguration\");
let result = retry.execute(|| async {{
    circuit_breaker.call(|| async {{
        let client = pool.get_client().await?;
        client.apply_configuration(request).await
    }}).await
}}).await;

match result {{
    Ok(_) => logger.finish_success(span),
    Err(e) => logger.finish_error(span, &e.to_string()),
}}
"
    );

    println!("\n✅ Example complete!");
    Ok(())
}
