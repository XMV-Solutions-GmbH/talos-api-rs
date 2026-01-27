# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
