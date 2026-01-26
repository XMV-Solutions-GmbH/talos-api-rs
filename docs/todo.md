# TODO

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

## Phase 2: Extended Machine Operations 🔄 IN PROGRESS

### Critical Blockers

- [ ] **ED25519 Certificate Support** 🔴
  - Talos uses ED25519 for mTLS certificates
  - Current rustls config doesn't properly handle ED25519 client certs
  - Options: Configure CryptoProvider, switch to native-tls, or use ring
  - Blocks: Full mTLS authentication

### Machine Config

- [ ] ApplyConfiguration - Apply machine config changes
- [ ] Reset - Factory reset with options
- [ ] Rollback - Revert to previous config

### System Information

- [ ] Memory - Detailed memory statistics
- [ ] CPUInfo - CPU details
- [ ] CPUFreqStats - CPU frequency info
- [ ] LoadAvg - System load averages
- [ ] DiskStats - Disk I/O statistics
- [ ] Mounts - Mounted filesystems
- [ ] NetworkDeviceStats - Network interface stats
- [ ] Processes - Running process list
- [ ] Containers - Container information
- [ ] Stats - Container resource stats

### Streaming APIs (requires special handling)

- [ ] Logs - Service log streaming (server-streaming)
- [ ] LogsContainers - Container log streaming
- [ ] Dmesg - Kernel messages (server-streaming)
- [ ] Events - Event stream (server-streaming)

### File Operations (streaming)

- [ ] List - Directory listing (server-streaming)
- [ ] Read - File content (server-streaming)
- [ ] Copy - Copy files (server-streaming)
- [ ] DiskUsage - Disk usage info (server-streaming)

### Service Control

- [ ] ServiceStart - Start a service
- [ ] ServiceStop - Stop a service
- [ ] ServiceRestart - Restart a service
- [ ] Restart - Restart machined

### Upgrade & Maintenance

- [ ] Upgrade - Trigger Talos upgrade
- [ ] ImageList - List container images
- [ ] ImagePull - Pull container images

---

## Phase 3: Cluster & etcd Operations

### Bootstrap

- [ ] Bootstrap - Initialize etcd cluster (critical for new clusters)

### etcd Operations

- [ ] EtcdMemberList - List etcd members
- [ ] EtcdRemoveMemberByID - Remove member by ID
- [ ] EtcdLeaveCluster - Gracefully leave cluster
- [ ] EtcdForfeitLeadership - Transfer leadership
- [ ] EtcdStatus - etcd health and stats
- [ ] EtcdAlarmList - List active alarms
- [ ] EtcdAlarmDisarm - Clear alarms
- [ ] EtcdDefragment - Defragment etcd storage
- [ ] EtcdSnapshot - Backup etcd (server-streaming)
- [ ] EtcdRecover - Restore etcd (client-streaming)

### Cluster Configuration

- [ ] Kubeconfig - Generate kubeconfig (server-streaming)
- [ ] GenerateClientConfiguration - Generate talosconfig

### Advanced Diagnostics

- [ ] PacketCapture - Network capture (server-streaming)
- [ ] Netstat - Network connections
- [ ] MetaWrite - Write metadata
- [ ] MetaDelete - Delete metadata

---

## Phase 4: Production Readiness (Future)

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

### Release

- [ ] crates.io publication
- [ ] Semantic versioning
- [ ] Changelog automation
- [ ] API stability guarantees

---

## OSS Compliance

- [x] README with disclaimer
- [x] LICENSE-MIT
- [x] LICENSE-APACHE
- [ ] CONTRIBUTING (needs expansion)
- [ ] CODE_OF_CONDUCT (needs expansion)
- [ ] SECURITY (needs expansion)
- [x] CI pipelines (Tests/Lint)
- [ ] Release pipelines (Build/Publish)
- [ ] Automated dependency updates (Dependabot)

---

## Known Issues Tracking

### 🔴 Critical

| Issue | Description | Status |
|-------|-------------|--------|
| ED25519 mTLS | Talos ED25519 certs not working with rustls | Open |

### 🟡 Medium

| Issue | Description | Status |
|-------|-------------|--------|
| Streaming APIs | Server-streaming gRPC not implemented | Open |
| Client-streaming | EtcdRecover needs client streaming | Open |
| Multi-node targeting | gRPC metadata for node selection | Open |

### 🟢 Low

| Issue | Description | Status |
|-------|-------------|--------|
| Error granularity | Parse google.rpc.Status details | Open |
| Generated code size | machine.rs is ~6000 lines | Acceptable |

---

## Testing Checklist

### Unit Tests

| Area | Tests | Status |
|------|-------|--------|
| TalosClientConfig | default, validation | ✅ |
| TalosClient::new | invalid cert, insecure | ✅ |
| Version API | mock server call | ✅ |
| Machine client type | compile-time check | ✅ |
| Machine types | request/response construction | ✅ |

### Integration Tests (Harness)

| Area | Tests | Status |
|------|-------|--------|
| Cluster lifecycle | create, connect, destroy | ✅ |
| Version API | real cluster call | ✅ |
| Hostname API | real cluster call | ✅ (blocked by mTLS) |
| ServiceList API | real cluster call | ✅ (blocked by mTLS) |
| SystemStat API | real cluster call | ✅ (blocked by mTLS) |
| talosctl commands | visual verification | ✅ |

### Future Test Requirements

- [ ] Each Phase 2 API method needs unit test
- [ ] Each Phase 2 API method needs integration test
- [ ] Streaming API tests (mock + real)
- [ ] Error handling tests
- [ ] Timeout/retry tests
- [ ] Multi-node targeting tests
