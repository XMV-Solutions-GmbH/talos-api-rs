<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# TODO

> **Updated**: 2026-01-27 - v0.2.0 Released! 🎉 Next: v0.3.0 Roadmap

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

### Observability ✅ COMPLETE

- [x] Logging interceptor (`LoggingInterceptor`, `RequestLogger`)
- [x] Log level configuration (Trace, Debug, Info, Warn, Error, Off)
- [x] Sensitive header redaction
- [x] Request metrics (`InterceptorMetrics`)
- [x] Metrics (Prometheus-compatible) - `MetricsCollector`, `MetricsConfig`
- [x] Distributed tracing (OpenTelemetry) - `TalosSpan`, `SpanFactory`, `TracingConfig`

### Developer Experience ✅ COMPLETE

- [x] High-level resource wrappers
- [x] Builder patterns for complex requests
- [x] `TalosClientConfig::builder()` fluent API
- [x] Comprehensive documentation
- [x] Example: resilient_client.rs - demonstrates all production features
- [x] Example: monitoring_metrics.rs - Prometheus metrics demo

### Release Preparation ✅ COMPLETE

- [ ] crates.io publication (requires CRATES_IO_TOKEN secret)
- [x] Semantic versioning
- [x] CHANGELOG.md
- [x] Release automation (GitHub Actions)
- [x] API stability guarantees - `docs/api-stability.md`
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

*No critical issues*

### 🟡 Medium

| Issue | Description | Status |
| ----- | ----------- | ------ |
| Server-streaming APIs | gRPC streaming not implemented | Open |
| Client-streaming | EtcdRecover needs client streaming | Open |
| Multi-node targeting | gRPC metadata for node selection | Open |

### 🟢 Low

| Issue | Description | Status |
| ----- | ----------- | ------ |
| Error granularity | Parse google.rpc.Status details | Open |
| Generated code size | machine.rs is ~6000 lines | Acceptable |

---

## Testing Checklist

### Unit Tests (Phase 1) ✅

| Area | Tests | Status |
| ----- | ------- | -------- |
| TalosClientConfig | default, validation | ✅ |
| TalosClient::new | invalid cert, insecure | ✅ |
| Version API | mock server call | ✅ |
| Machine client type | compile-time check | ✅ |
| Machine types | request/response construction | ✅ |

### Integration Tests (Phase 1) ✅

| Area | Tests | Status |
| ----- | ------- | -------- |
| Cluster lifecycle | create, connect, destroy | ✅ |
| Version API | real cluster call | ✅ |
| Hostname API | real cluster call | ✅ |
| ServiceList API | real cluster call | ✅ |
| SystemStat API | real cluster call | ✅ |

### Phase 2 Test Requirements ✅

- [x] ApplyConfiguration (insecure mode)
- [x] Bootstrap (ED25519 fixed)
- [x] Kubeconfig streaming
- [x] Reset graceful
- [x] Health check API (Version)
- [x] EtcdMemberList, EtcdStatus, EtcdAlarmList
- [x] Dmesg streaming
- [x] System APIs (Memory, CPU, LoadAvg, Disks, Mounts, Network, Processes)

---

## Release Milestones

### v0.1.0 (Current) - Experimental ✅

- ✅ Basic connectivity
- ✅ Version API
- ✅ Basic Machine API
- ✅ mTLS with ED25519 (ring crypto provider)

### v0.2.0 - Alpha ✅

- [x] ED25519 mTLS resolved (ring crypto provider)
- [x] ApplyConfiguration (with dry-run)
- [x] Bootstrap API
- [x] Kubeconfig API (server-streaming)
- [x] Reset API (graceful/force/halt)
- [x] Health API (Version)
- [x] Basic documentation

### v0.3.0 - Beta ✅

- [x] etcd operations (MemberList, Status, AlarmList, Defragment, etc.)
- [x] Streaming APIs (Kubeconfig, Dmesg, Logs, Files)
- [x] Service control (Start, Stop, Restart)
- [x] Full documentation

### v1.0.0 - Stable ✅

- [x] Production-grade error handling (`TalosError`)
- [x] Connection pooling (`ConnectionPool`, load balancing)
- [x] Retry policies (exponential, linear, fixed backoff)
- [x] Circuit breaker pattern
- [x] Prometheus metrics
- [x] OpenTelemetry tracing
- [x] API stability commitment (`docs/api-stability.md`)

---

## v0.2.0 Roadmap - Next Release

### Dependency Updates 🔧 ✅ COMPLETE (v0.2.0)

| PR | Description | Status | Notes |
| -- | ----------- | ------ | ----- |
| #19 | webpki-roots 0.26.11 → 1.0.5 | ✅ Merged | Security update: removed untrusted CAs |
| - | tonic 0.12 → 0.14 | ✅ Complete | MSRV 1.82, new tls features |
| - | prost 0.13 → 0.14 | ✅ Complete | tonic-prost extracted |
| - | tonic-build → tonic-prost-build | ✅ Complete | Renamed in tonic 0.14 |

**Note:** rand 0.8 → 0.9 still blocked by tower ecosystem.

### Streaming API Improvements 🚀

- [ ] True async streaming for `Kubeconfig`, `Dmesg`, `Logs`
- [ ] Backpressure handling for large streams
- [ ] Streaming progress callbacks

### Multi-Node Operations 🎯 ✅ COMPLETE (v0.1.2)

- [x] gRPC metadata for node targeting (`x-talos-node`)
- [x] Cluster-wide operations (apply to all nodes via `with_nodes()`)
- [ ] Parallel execution with result aggregation

### Missing APIs 📡

- [ ] EtcdRecover (client-streaming)
- [ ] EtcdSnapshot (server-streaming)
- [x] ImageList, ImagePull ✅ v0.2.0
- [ ] Events API

### Quality of Life 🛠️ ✅ COMPLETE (v0.1.2 / v0.2.0)

- [x] `talosctl` config file parsing (`~/.talos/config`) - `TalosConfig::load_default()`
- [x] Environment-based configuration (`TALOS_ENDPOINTS`, `TALOS_CONTEXT`) - `TalosConfig::load_with_env()`
- [x] `TalosClient::from_talosconfig()` for easy client creation
- [x] Cluster discovery helpers ✅ v0.2.0 - `ClusterDiscovery`, `ClusterHealth`

### Documentation 📚

- [ ] More examples (cluster upgrade workflow)
- [ ] Tutorial: Building a Talos operator
- [ ] API coverage matrix vs talosctl

---

## Future Considerations (v0.3.0+)

- [ ] Talos 1.10 API additions (when released)
- [ ] SideroLink integration
- [ ] Machine config validation (schema-based)
- [ ] Async trait stabilization (when Rust stabilizes)
