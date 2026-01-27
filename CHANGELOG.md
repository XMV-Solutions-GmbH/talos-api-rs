# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-27

**Talos gRPC API**: v1.9.x (compatible with Talos Linux 1.9+)
**Documentation**: <https://docs.rs/talos-api-rs/0.2.0/>

### Added

- **Container Image APIs**
  - `ImageListRequest` - List container images with namespace filtering
  - `ImageInfo` - Detailed image information (name, digest, size, created_at)
  - Helper methods: `size_human()`, `repository()`, `tag()`, `is_digest_reference()`
  - `ImagePullRequest` - Pull container images with builder pattern
  - `ContainerdNamespace` enum (Unknown, System, Cri)
  - `image_list()` and `image_pull()` methods on `TalosClient`

- **Cluster Discovery Helpers**
  - `ClusterDiscovery` - Discover and health-check cluster members
  - `ClusterDiscoveryBuilder` - Fluent configuration for discovery
  - `ClusterMember` - Information about discovered nodes
  - `ClusterHealth` / `NodeHealth` - Health status tracking
  - `NodeRole` enum (ControlPlane, Worker, Unknown)
  - Methods: `discover_members()`, `check_cluster_health()`, `get_cluster_versions()`

### Changed

- **BREAKING: MSRV increased** - Minimum Supported Rust Version is now 1.82 (was 1.75)
- **BREAKING: Dependencies upgraded**
  - `tonic` 0.12 → 0.14.2
  - `prost` 0.13 → 0.14.3
  - `prost-types` 0.13 → 0.14.3
  - `tonic-build` 0.12 → `tonic-prost-build` 0.14.2
- **New dependency**: `tonic-prost` 0.14.2 (prost extracted from tonic)
- **Feature rename**: `tls-roots` → `tls-webpki-roots` (tonic 0.14 API change)
- **New feature**: `tls-ring` for TLS with ring crypto provider

### Migration Guide

If upgrading from 0.1.x:

1. Ensure Rust 1.82+ is installed
2. The TLS features changed:
   - Old: `tonic = { features = ["tls", "tls-roots"] }`
   - New: `tonic = { features = ["transport", "tls-ring", "tls-webpki-roots"] }`
3. `tonic-prost` is now a separate runtime dependency

## [0.1.2] - 2026-01-27

### Added

- **Node Targeting** - Target specific nodes in multi-node clusters
  - `NodeTarget` enum for specifying target nodes (single, multiple, or default)
  - `with_node()` / `with_nodes()` methods on `TalosClient`
  - gRPC metadata `x-talos-node` header support
  - Cluster-wide operations support
- **Environment Configuration** - Environment variable support for configuration
  - `TALOSCONFIG` - Override config file path
  - `TALOS_CONTEXT` - Override active context
  - `TALOS_ENDPOINTS` - Override endpoints (comma-separated)
  - `TALOS_NODES` - Target specific nodes (comma-separated)
  - `TalosConfig::load_with_env()` method
- **TalosConfig Integration** - Create client directly from talosconfig
  - `TalosClient::from_talosconfig()` method
  - Automatic cert extraction and mTLS setup
  - Context-based endpoint selection

### Fixed

- **Release naming** - Fixed double "v" in GitHub release titles (was "vv0.1.1", now "v0.1.2")

### Changed

- Improved documentation for config module with environment variable details

## [0.1.1] - 2025-01-27

### Fixed

- **docs.rs build failure** - Skip protobuf code generation when `DOCS_RS=1` environment variable is set.
  docs.rs has a read-only filesystem, so we use pre-generated code instead.

## [Unreleased]

### Added
- **OpenTelemetry Tracing** - Distributed tracing support
  - `TalosSpan` for creating spans with OpenTelemetry semantic conventions
  - `SpanFactory` for consistent span creation across the client
  - `TracingConfig` for configuring tracing behavior
  - W3C Trace Context compatible attributes
  - `instrument_talos!` macro for easy instrumentation
- **Prometheus Metrics** - Production-grade observability
  - `MetricsCollector` for collecting request metrics
  - `MetricsConfig` with configurable namespace, labels, and histogram buckets
  - Request counters with method/endpoint/status labels
  - Response time histograms
  - Circuit breaker state and rejection metrics
  - Connection pool health metrics
  - Prometheus text format export (`to_prometheus_text()`)
  - `MetricsSnapshot` for programmatic access
- **New Example** - `monitoring_metrics.rs` demonstrating Prometheus metrics
- **Logging Interceptor** - Structured logging for gRPC requests
  - `LoggingInterceptor` for tonic interceptor integration
  - `RequestLogger` for manual request/response logging with timing
  - `LogLevel` configuration (Trace, Debug, Info, Warn, Error, Off)
  - `LoggingConfig` with sensitive header redaction
  - `InterceptorMetrics` for request counting and success rates
  - Verbose and quiet configuration presets
- **Circuit Breaker** - Resilience pattern for protecting against cascading failures
  - `CircuitBreaker` with configurable thresholds and timeouts
  - States: `Closed`, `Open`, `HalfOpen`
  - Automatic recovery with configurable success threshold
  - Metrics: failure rate, total calls, rejections
- **Connection Pool** - Multi-endpoint support with health-based routing
  - `ConnectionPool` for managing connections to multiple Talos nodes
  - `EndpointHealth` tracking with failure/success metrics
  - Load balancing strategies: `RoundRobin`, `Random`, `LeastFailures`, `Failover`
  - Automatic health checks and endpoint recovery
- **New Error Type** - `TalosError::Connection` for connection-related errors
- **New Error Type** - `TalosError::CircuitOpen` for circuit breaker rejections
- **Retry Policies** - Configurable retry logic with backoff strategies
  - `RetryConfig` with `ExponentialBackoff`, `LinearBackoff`, `FixedBackoff`, `NoBackoff`
  - `DefaultRetryPolicy` - retry on transient gRPC errors (Unavailable, Unknown, etc.)
  - `CustomRetryPolicy` - retry on specific error codes
  - Jitter support to prevent thundering herd
- **Timeout Configuration** - Enhanced `TalosClientConfig` with:
  - `connect_timeout` - connection establishment timeout
  - `request_timeout` - per-request timeout
  - `keepalive_interval` / `keepalive_timeout` - HTTP/2 keepalive settings
- **Builder Pattern** - `TalosClientConfig::builder()` for fluent configuration

## [0.1.0] - 2025-01-26

### Added
- **Core Client** (`TalosClient`)
  - gRPC connectivity with `tonic`
  - mTLS support with ED25519 certificates (using `ring` crypto provider)
  - Insecure mode for development
  - HTTP (non-TLS) support

- **Authentication & TLS**
  - Full mTLS with client certificates
  - Custom PEM parser for ED25519 private keys
  - CA certificate validation
  - SNI support

- **Machine APIs** (37+ methods)
  - Version, Hostname, SystemStat
  - Reboot, Shutdown
  - ServiceList, ServiceStart, ServiceStop, ServiceRestart
  - ApplyConfiguration (with dry-run support)
  - Bootstrap (cluster initialization)
  - Kubeconfig (server-streaming)
  - Reset (graceful/force/halt modes)
  - Upgrade (staged upgrades, reboot modes)
  - Dmesg (kernel messages, server-streaming)
  - Logs (container logs, server-streaming)
  - Rollback

- **etcd Operations** (8 APIs)
  - EtcdMemberList - list cluster members
  - EtcdRemoveMemberByID - remove member
  - EtcdLeaveCluster - graceful leave
  - EtcdForfeitLeadership - step down as leader
  - EtcdStatus - cluster health
  - EtcdAlarmList - list alarms
  - EtcdAlarmDisarm - clear alarms
  - EtcdDefragment - compact database

- **System Information** (7 APIs)
  - Memory - RAM usage
  - CPUInfo - processor details
  - DiskStats - I/O statistics
  - Mounts - mounted filesystems
  - NetworkDeviceStats - NIC statistics
  - Processes - running processes
  - LoadAvg - system load averages

- **File Operations** (4 APIs, all streaming)
  - List - directory listing
  - Read - file content
  - Copy - copy as tar archive
  - DiskUsage - directory size info

- **Advanced APIs** (4 APIs)
  - GenerateClientConfiguration - create talosconfig
  - PacketCapture - network capture (streaming)
  - Netstat - network connections with filtering
  - Rollback - revert to previous config

- **Test Infrastructure**
  - `testkit` module with `TalosCluster` helper
  - Automatic cluster provisioning via `talosctl`
  - Environment guard (`TALOS_DEV_TESTS=1`)
  - 94 unit tests
  - Comprehensive integration tests

- **Documentation**
  - Full API documentation with examples
  - Architecture documentation
  - Contributing guidelines
  - Security policy
  - MIT/Apache-2.0 dual license

### Fixed
- ED25519 certificate support with custom `ring` crypto provider
- Server-streaming RPC handling for Kubeconfig, Dmesg, Logs, and file operations

### Security
- SPDX license headers on all source files
- No `unwrap()` in library code
- Proper error handling throughout

[Unreleased]: https://github.com/XMV-Solutions-GmbH/talos-api-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/XMV-Solutions-GmbH/talos-api-rs/releases/tag/v0.1.0
