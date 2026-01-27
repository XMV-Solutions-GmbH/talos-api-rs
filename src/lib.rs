// SPDX-License-Identifier: MIT OR Apache-2.0

//! # talos-api-rs
//!
//! A typed, async, idiomatic Rust client for the [Talos Linux](https://www.talos.dev/) gRPC API.
//!
//! ## Features
//!
//! - **40+ APIs** — Machine, etcd, system, files, diagnostics
//! - **Async-first** — Built on `tokio` and `tonic`
//! - **Strongly typed** — No stringly-typed API calls
//! - **Production-ready** — Retries, circuit breakers, connection pooling
//! - **Observable** — Prometheus metrics, OpenTelemetry tracing
//! - **mTLS support** — ED25519 certificates (Talos default)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use talos_api_rs::{TalosClient, TalosClientConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect with mTLS
//! let config = TalosClientConfig::builder("https://10.0.0.1:50000")
//!     .ca_cert("/path/to/ca.crt")
//!     .client_cert("/path/to/client.crt")
//!     .client_key("/path/to/client.key")
//!     .build();
//!
//! let client = TalosClient::new(config).await?;
//!
//! // Get kubeconfig from cluster
//! let kubeconfig = client.kubeconfig().await?;
//! println!("Got kubeconfig for cluster");
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Overview
//!
//! - [`client`] — Core client and connection management
//! - [`config`] — Configuration file parsing (talosctl config)
//! - [`resources`] — Typed API request/response wrappers
//! - [`runtime`] — Resilience (retry, circuit breaker) and observability
//! - [`error`] — Error types
//! - [`api`] — Generated protobuf types (low-level)
//! - [`testkit`] — Testing utilities
//!
//! ## Production Features
//!
//! ### Retry Policies
//!
//! ```rust
//! use talos_api_rs::runtime::{RetryConfig, ExponentialBackoff};
//! use std::time::Duration;
//!
//! let retry = RetryConfig::builder()
//!     .max_retries(3)
//!     .backoff(ExponentialBackoff::new(Duration::from_millis(100)))
//!     .build();
//! ```
//!
//! ### Circuit Breaker
//!
//! ```rust
//! use talos_api_rs::runtime::{CircuitBreaker, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! let cb = CircuitBreaker::new(
//!     CircuitBreakerConfig::new()
//!         .with_failure_threshold(5)
//!         .with_reset_timeout(Duration::from_secs(30))
//! );
//! ```
//!
//! ### Prometheus Metrics
//!
//! ```rust
//! use talos_api_rs::runtime::{MetricsCollector, MetricsConfig};
//!
//! let metrics = MetricsCollector::new(
//!     MetricsConfig::builder()
//!         .namespace("talos")
//!         .build()
//! );
//!
//! // Export Prometheus format
//! let output = metrics.to_prometheus_text();
//! ```
//!
//! ## Disclaimer
//!
//! This project is **NOT** affiliated with Sidero Labs or Talos Linux.
//! Provided AS-IS, without warranty of any kind.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod resources;
pub mod runtime;
pub mod testkit;

pub use client::{
    ConnectionPool, ConnectionPoolConfig, EndpointHealth, HealthStatus, LoadBalancer, NodeTarget,
    TalosClient, TalosClientConfig, TalosClientConfigBuilder, NODE_METADATA_KEY,
};
pub use config::{
    TalosConfig, TalosContext, ENV_TALOSCONFIG, ENV_TALOS_CONTEXT, ENV_TALOS_ENDPOINTS,
    ENV_TALOS_NODES,
};
pub use error::TalosError;
pub use resources::{
    ApplyConfigurationRequest, ApplyConfigurationResponse, ApplyConfigurationResult, ApplyMode,
    BootstrapRequest, BootstrapResponse, BootstrapResult, ConnectionRecord, ConnectionState,
    ContainerDriver, CopyRequest, CopyResponse, CpuInfo, CpuInfoResponse, CpuInfoResult, DiskStat,
    DiskStatsResponse, DiskStatsResult, DiskUsageInfo, DiskUsageRequest, DiskUsageResponse,
    DmesgRequest, DmesgResponse, EtcdAlarmDisarmResponse, EtcdAlarmListResponse, EtcdAlarmType,
    EtcdDefragmentResponse, EtcdForfeitLeadershipRequest, EtcdForfeitLeadershipResponse,
    EtcdLeaveClusterRequest, EtcdLeaveClusterResponse, EtcdMember, EtcdMemberAlarm,
    EtcdMemberListRequest, EtcdMemberListResponse, EtcdMemberStatus, EtcdRemoveMemberByIdRequest,
    EtcdRemoveMemberByIdResponse, EtcdStatusResponse, FileInfo, FileType,
    GenerateClientConfigurationRequest, GenerateClientConfigurationResponse,
    GenerateClientConfigurationResult, KubeconfigResponse, L4ProtoFilter, ListRequest,
    ListResponse, LoadAvgResponse, LoadAvgResult, LogsRequest, LogsResponse, MemoryResponse,
    MemoryResult, MountStat, MountsResponse, MountsResult, NetDevStat, NetstatFilter,
    NetstatRequest, NetstatResponse, NetstatResult, NetworkDeviceStatsResponse,
    NetworkDeviceStatsResult, PacketCaptureRequest, PacketCaptureResponse, ProcessInfo,
    ProcessesResponse, ProcessesResult, ReadRequest, ReadResponse, ResetPartitionSpec,
    ResetRequest, ResetResponse, ResetResult, RollbackResponse, RollbackResult,
    ServiceRestartRequest, ServiceRestartResponse, ServiceStartRequest, ServiceStartResponse,
    ServiceStopRequest, ServiceStopResponse, UpgradeRebootMode, UpgradeRequest, UpgradeResponse,
    UpgradeResult, WipeMode,
};
pub use runtime::{
    BackoffStrategy, CircuitBreaker, CircuitBreakerConfig, CircuitState, CustomRetryPolicy,
    DefaultRetryPolicy, ExponentialBackoff, FixedBackoff, InterceptorMetrics, LinearBackoff,
    LogLevel, LoggingConfig, LoggingInterceptor, NoBackoff, NoRetryPolicy, RequestLogger,
    RequestSpan, RetryConfig, RetryConfigBuilder, RetryPolicy,
};
