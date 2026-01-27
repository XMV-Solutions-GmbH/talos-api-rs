# TODO

> **Updated**: 2025-01-27 - Phase 4 Production Readiness - Logging Interceptor added.

## Phase 1: Core Foundation ✅ COMPLETE

- [x] Project scaffolding
- [x] TalosClient core
- [x] Auth & TLS config (Detailed implementation)
- [x] Insecure mode (skip TLS verify)
- [x] Version API (Health check)
- [x] Machine API basics (ServiceList, Hostname, SystemStat, Reboot, Shutdown)
- [x] Unit tests (Core, Auth & Machine APIs)
- [x] Integration test harness (Docker-based Talos cluster)

---

## Phase 2: Alpha Release (Cluster Lifecycle Core) ✅ COMPLETE

### ✅ Critical Blockers (RESOLVED)

- [x] **ED25519 Certificate Support**
  - ✅ Custom rustls connector with ring crypto provider
  - ✅ Custom PEM parser for "ED25519 PRIVATE KEY" label
  - ✅ Full mTLS working with Talos clusters
  - Merged: PR #7

### ✅ Priority 1: Alpha-Blocking Features (ALL COMPLETE)

- [x] **ApplyConfiguration**
  - ✅ Typed wrappers: `ApplyMode`, `ApplyConfigurationRequest`, `ApplyConfigurationResponse`
  - ✅ Builder pattern for request construction
  - ✅ High-level `TalosClient::apply_configuration()` method
  - ✅ Convenience `apply_configuration_yaml()` method
  - ✅ Unit tests for all types
  - ✅ Integration test (dry-run mode)

- [x] **Bootstrap**
  - ✅ Typed wrappers: `BootstrapRequest`, `BootstrapResponse`, `BootstrapResult`
  - ✅ Builder pattern with recovery options
  - ✅ High-level `TalosClient::bootstrap()` method
  - ✅ Convenience `bootstrap_cluster()` method
  - ✅ Unit tests (6 tests)
  - ✅ Integration test (verifies rejection on already-bootstrapped cluster)

- [x] **Kubeconfig** (server-streaming)
  - ✅ Typed wrapper: `KubeconfigResponse`
  - ✅ Server-streaming RPC handling with chunk assembly
  - ✅ High-level `TalosClient::kubeconfig()` method
  - ✅ Helper methods: `as_str()`, `write_to_file()`, `len()`
  - ✅ Unit tests (5 tests)
  - ✅ Integration test (retrieves kubeconfig, validates structure)

- [x] **Reset** (destructive)
  - ✅ Typed wrappers: `WipeMode`, `ResetPartitionSpec`, `ResetRequest`, `ResetResponse`, `ResetResult`
  - ✅ Builder pattern with presets: `graceful()`, `force()`, `halt()`
  - ✅ High-level `TalosClient::reset()` method
  - ✅ Convenience `reset_graceful()` method
  - ✅ Unit tests (9 tests)
  - ✅ Integration test (API verification only - destructive operation skipped)

### ✅ Priority 2-3: Beta Features (ALL COMPLETE)

- [x] **etcd Operations** (8 APIs)
  - ✅ EtcdMemberList, EtcdRemoveMemberByID, EtcdLeaveCluster, EtcdForfeitLeadership
  - ✅ EtcdStatus, EtcdAlarmList, EtcdAlarmDisarm, EtcdDefragment
  - ✅ Full typed wrappers with EtcdMember, EtcdMemberStatus, EtcdAlarmType
  - ✅ Unit tests (7 tests)

- [x] **Dmesg** (server-streaming)
  - ✅ Kernel message buffer streaming
  - ✅ Follow/tail options
  - ✅ Builder pattern
  - ✅ Unit tests (6 tests)

- [x] **Upgrade**
  - ✅ UpgradeRebootMode (Default, PowerCycle)
  - ✅ Stage, preserve, force options
  - ✅ Builder pattern
  - ✅ Unit tests (4 tests)

- [x] **Service Management**
  - ✅ ServiceStart, ServiceStop, ServiceRestart
  - ✅ Full typed wrappers
  - ✅ Unit tests (3 tests)

- [x] **Logs** (server-streaming)
  - ✅ Container log streaming
  - ✅ ContainerDriver (Containerd, Cri)
  - ✅ Namespace, follow, tail options
  - ✅ Unit tests (5 tests)

---

## Phase 3: Extended APIs ✅ COMPLETE

### ✅ System Information (7 APIs)

- [x] Memory - Detailed memory statistics
- [x] CPUInfo - CPU details
- [x] DiskStats - Disk I/O statistics
- [x] Mounts - Mounted filesystems
- [x] NetworkDeviceStats - Network interface stats
- [x] Processes - Running process list
- [x] LoadAvg - System load averages
- ✅ Unit tests (7 tests)

### ✅ File Operations (4 APIs, all streaming)

- [x] List - Directory listing (server-streaming)
- [x] Read - File content (server-streaming)
- [x] Copy - Copy files as tar (server-streaming)
- [x] DiskUsage - Disk usage info (server-streaming)
- ✅ Builder patterns for List and DiskUsage
- ✅ Unit tests (9 tests)

### ✅ Advanced APIs (4 APIs)

- [x] Rollback - Revert to previous config
- [x] GenerateClientConfiguration - Generate talosconfig
- [x] PacketCapture - Network capture (server-streaming)
- [x] Netstat - Network connections with filtering
- ✅ L4ProtoFilter, NetstatFilter, ConnectionState
- ✅ Unit tests (9 tests)

---

## Phase 4: Production Readiness & crates.io

### Connection Management ✅ PARTIAL

- [x] Multi-endpoint support with health-based routing (`ConnectionPool`)
- [x] Connection pooling with endpoint health tracking
- [x] Automatic reconnection on failure
- [x] Load balancing strategies (RoundRobin, Random, LeastFailures, Failover)

### Resilience ✅ COMPLETE

- [x] Configurable retry policies
- [x] Exponential backoff (with jitter)
- [x] Linear and fixed backoff strategies
- [x] Circuit breaker pattern (`CircuitBreaker` with Closed/Open/HalfOpen states)
- [x] Per-request timeouts (`request_timeout`)
- [x] Connection timeouts (`connect_timeout`)
- [x] HTTP/2 keepalive configuration

### Observability ✅ PARTIAL

- [x] Logging interceptor (`LoggingInterceptor`, `RequestLogger`)
- [x] Log level configuration (Trace, Debug, Info, Warn, Error, Off)
- [x] Sensitive header redaction
- [x] Request metrics (`InterceptorMetrics`)
- [ ] Metrics (Prometheus-compatible)
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Distributed tracing (OpenTelemetry)

### Developer Experience ✅ PARTIAL

- [x] High-level resource wrappers
- [x] Builder patterns for complex requests
- [x] `TalosClientConfig::builder()` fluent API
- [x] Comprehensive documentation
- [x] Example: resilient_client.rs - demonstrates all production features
- [ ] More examples (cluster lifecycle, monitoring)

### Release Preparation ✅ PARTIAL

- [ ] crates.io publication
- [x] Semantic versioning
- [x] CHANGELOG.md
- [ ] Changelog automation
- [ ] API stability guarantees
- [x] MSRV (Minimum Supported Rust Version) policy - Rust 1.75+
- [x] docs.rs documentation metadata

---

## OSS Compliance

- [x] README with disclaimer
- [x] LICENSE-MIT
- [x] LICENSE-APACHE
- [x] CONTRIBUTING.md
- [x] CODE_OF_CONDUCT.md
- [x] SECURITY.md
- [x] CI pipelines (Tests/Lint)
- [x] Release pipelines (Build/Publish) - `.github/workflows/release.yml`
- [x] Automated dependency updates (Dependabot) - `.github/dependabot.yml`
- [x] CHANGELOG.md
- [x] Release checklist documentation - `docs/release-checklist.md`

---

## Known Issues Tracking

### 🔴 Critical

| Issue | Description | Status | Blocks |
|-------|-------------|--------|--------|
| ED25519 mTLS | Talos ED25519 certs not working with rustls | Open | Phase 2 Alpha |

### 🟡 Medium

| Issue | Description | Status |
|-------|-------------|--------|
| Server-streaming APIs | gRPC streaming not implemented | Open |
| Client-streaming | EtcdRecover needs client streaming | Open |
| Multi-node targeting | gRPC metadata for node selection | Open |

### 🟢 Low

| Issue | Description | Status |
|-------|-------------|--------|
| Error granularity | Parse google.rpc.Status details | Open |
| Generated code size | machine.rs is ~6000 lines | Acceptable |

---

## Testing Checklist

### Unit Tests (Phase 1) ✅

| Area | Tests | Status |
|------|-------|--------|
| TalosClientConfig | default, validation | ✅ |
| TalosClient::new | invalid cert, insecure | ✅ |
| Version API | mock server call | ✅ |
| Machine client type | compile-time check | ✅ |
| Machine types | request/response construction | ✅ |

### Integration Tests (Phase 1) ✅

| Area | Tests | Status |
|------|-------|--------|
| Cluster lifecycle | create, connect, destroy | ✅ |
| Version API | real cluster call | ✅ |
| Hostname API | real cluster call | ⚠️ (mTLS blocked) |
| ServiceList API | real cluster call | ⚠️ (mTLS blocked) |
| SystemStat API | real cluster call | ⚠️ (mTLS blocked) |

### Phase 2 Test Requirements

- [ ] ApplyConfiguration (insecure mode)
- [ ] Bootstrap (after ED25519 fix)
- [ ] Kubeconfig streaming
- [ ] Reset graceful
- [ ] Health check API
- [ ] EtcdRemoveMember
- [ ] Dmesg streaming
- [ ] Upgrade API

---

## Release Milestones

### v0.1.0 (Current) - Experimental

- ✅ Basic connectivity
- ✅ Version API
- ✅ Basic Machine API
- ⚠️ mTLS partially working

### v0.2.0 (Target) - Alpha

- [ ] ED25519 mTLS resolved
- [ ] ApplyConfiguration (insecure)
- [ ] Bootstrap API
- [ ] Kubeconfig API
- [ ] Reset API
- [ ] Health API
- [ ] Basic documentation

### v0.3.0 - Beta

- [ ] etcd operations
- [ ] Streaming APIs
- [ ] Service control
- [ ] Full documentation

### v1.0.0 - Stable

- [ ] Production-grade error handling
- [ ] Connection pooling
- [ ] Retry policies
- [ ] API stability commitment
