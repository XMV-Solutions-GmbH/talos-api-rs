# TODO

> **Updated**: 2025-01-26 - ApplyConfiguration implemented, ED25519 fixed.

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

## Phase 2: Alpha Release (Cluster Lifecycle Core) 🔄 IN PROGRESS

### ✅ Critical Blockers (RESOLVED)

- [x] **ED25519 Certificate Support**
  - ✅ Custom rustls connector with ring crypto provider
  - ✅ Custom PEM parser for "ED25519 PRIVATE KEY" label
  - ✅ Full mTLS working with Talos clusters
  - Merged: PR #7

### Priority 1: Alpha-Blocking Features

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

- [ ] **Reset** (graceful)
  - Graceful node shutdown/reset
  - Options: graceful, reboot, system_disk_wiping
  - Used for destroy and scale-down

### Priority 2: Beta Features

- [ ] **Health Check API**
  - Pre-flight checks before operations
  - Node health monitoring
  - Cluster-wide health status

- [ ] **EtcdRemoveMemberByID**
  - Remove control-plane node from etcd
  - Required for CP scale-down
  - Must remove before node reset

- [ ] **Dmesg** (server-streaming)
  - Kernel message buffer
  - Diagnostics and troubleshooting
  - Streaming API

### Priority 3: Production Features

- [ ] **Upgrade**
  - Trigger Talos version upgrade
  - Rolling upgrade support
  - Version verification

### Out of Scope (CLI-only, not gRPC)

These are local CLI operations, NOT gRPC APIs:

- `talosctl gen config` - Generates YAML locally
- `talosctl config endpoint/node` - Local talosconfig manipulation  
- `talosctl cluster create/destroy` - Docker provider (testkit only)

---

## Phase 3: Extended APIs

### etcd Operations

- [ ] EtcdMemberList - List etcd members
- [ ] EtcdLeaveCluster - Gracefully leave cluster
- [ ] EtcdForfeitLeadership - Transfer leadership
- [ ] EtcdStatus - etcd health and stats
- [ ] EtcdAlarmList - List active alarms
- [ ] EtcdAlarmDisarm - Clear alarms
- [ ] EtcdDefragment - Defragment etcd storage
- [ ] EtcdSnapshot - Backup etcd (server-streaming)
- [ ] EtcdRecover - Restore etcd (client-streaming)

### Service & Logs

- [ ] ServiceStart - Start a service
- [ ] ServiceStop - Stop a service
- [ ] ServiceRestart - Restart a service
- [ ] Logs - Service log streaming (server-streaming)
- [ ] LogsContainers - Container log streaming
- [ ] Events - Event stream (server-streaming)

### System Information

- [ ] Memory - Detailed memory statistics
- [ ] CPUInfo - CPU details
- [ ] DiskStats - Disk I/O statistics
- [ ] Mounts - Mounted filesystems
- [ ] NetworkDeviceStats - Network interface stats
- [ ] Processes - Running process list
- [ ] LoadAvg - System load averages

### File Operations (streaming)

- [ ] List - Directory listing (server-streaming)
- [ ] Read - File content (server-streaming)
- [ ] Copy - Copy files (server-streaming)
- [ ] DiskUsage - Disk usage info (server-streaming)

### Advanced

- [ ] Rollback - Revert to previous config
- [ ] GenerateClientConfiguration - Generate talosconfig
- [ ] PacketCapture - Network capture (server-streaming)
- [ ] Netstat - Network connections

---

## Phase 4: Production Readiness & crates.io

### Connection Management

- [ ] Multi-endpoint support with health-based routing
- [ ] Connection pooling
- [ ] Automatic reconnection
- [ ] Load balancing across nodes

### Resilience

- [ ] Configurable retry policies
- [ ] Exponential backoff
- [ ] Circuit breaker pattern
- [ ] Per-request timeouts
- [ ] Global timeouts

### Observability

- [ ] Logging interceptor
- [ ] Metrics (Prometheus-compatible)
- [ ] Distributed tracing (OpenTelemetry)

### Developer Experience

- [ ] High-level resource wrappers
- [ ] Builder patterns for complex requests
- [ ] Comprehensive documentation
- [ ] More examples

### Release Preparation

- [ ] crates.io publication
- [ ] Semantic versioning
- [ ] Changelog automation
- [ ] API stability guarantees
- [ ] MSRV (Minimum Supported Rust Version) policy
- [ ] docs.rs documentation

---

## OSS Compliance

- [x] README with disclaimer
- [x] LICENSE-MIT
- [x] LICENSE-APACHE
- [x] CONTRIBUTING.md
- [x] CODE_OF_CONDUCT.md
- [x] SECURITY.md
- [x] CI pipelines (Tests/Lint)
- [ ] Release pipelines (Build/Publish)
- [ ] Automated dependency updates (Dependabot)
- [ ] CHANGELOG.md
- [ ] Release checklist documentation

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
