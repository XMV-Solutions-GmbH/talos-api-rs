# Application Concept

A typed, async, idiomatic Rust client for the Talos Linux gRPC API.

## Core Design

- **One Client**: A central `TalosClient` manages connection pooling, authentication, and configuration.
- **Typed APIs**: All gRPC methods are exposed via strongly-typed Rust methods, hiding raw Protobuf details where possible.
- **Async-First**: Built on `tokio` and `tonic`.
- **No CLI Wrapper**: Does not call `talosctl` internally; uses pure gRPC.

## Authentication & Security

The client supports:

- **mTLS**: Mutual TLS using Client Certificate, Client Key, and CA Certificate.
- **Insecure Mode**: A specific flag (`insecure: true`) to bypass TLS verification (useful for bootstrap or maintenance).
- **Endpoint validation**: Hostname verification (default on, disabled in insecure mode).

## Modules

- `client`: Connection and auth logic.
- `api`: Generated gRPC code (via `tonic` + `prost`).
- `resources`: High-level wrappers for Talos resources.
- `testkit`: Integration testing harness using local Talos clusters.

---

## Development Phases

> **Note**: Phases reprioritized based on cluster-lifecycle-manager requirements (2026-01-26).

### Phase 1: Core Foundation ✅ COMPLETE

**Goal**: Establish a working client with basic connectivity and essential APIs.

| Feature | Status | Notes |
| ------- | ------ | ----- |
| Project scaffolding | ✅ | Cargo workspace, CI/CD |
| TalosClient core | ✅ | Connection management |
| TLS/mTLS config | ✅ | Certificate loading |
| Insecure mode | ✅ | Custom TLS verifier |
| Version API | ✅ | Health check capability |
| Machine API (basic) | ✅ | ServiceList, Hostname, SystemStat, Reboot, Shutdown |
| Unit tests | ✅ | 6 tests covering core functionality |
| Integration harness | ✅ | Docker-based Talos cluster |

---

### Phase 2: Alpha Release (Cluster Lifecycle Core) 🔄 IN PROGRESS

**Goal**: Enable core cluster lifecycle operations for the Tauri app.

> Based on `talosctl` commands analysis for cluster-lifecycle-manager.

#### Priority 1: Absolute Core (Alpha-Blocking)

| # | Feature | mTLS | Status | Description |
| - | ------- | ---- | ------ | ----------- |
| 1 | `gen config` | ❌ | ❌ | Machine config generation (NOT gRPC - CLI only) |
| 2 | `ApplyConfiguration --insecure` | ❌ | ❌ | Initial config in maintenance mode |
| 3 | `Bootstrap` | ✅ | ❌ | Initialize etcd on first control-plane |
| 4 | `Kubeconfig` (streaming) | ✅ | ❌ | Retrieve kubeconfig |
| 5 | `Reset --graceful` | ✅ | ❌ | Graceful node teardown |

**Critical Blocker**: ED25519 mTLS must work for Bootstrap, Kubeconfig, Reset.

#### Priority 2: Beta Operations

| # | Feature | mTLS | Status | Description |
| - | ------- | ---- | ------ | ----------- |
| 6 | Health check API | ✅ | ❌ | Pre-flight checks, monitoring |
| 7 | `EtcdRemoveMember` | ✅ | ❌ | Control-plane scale-down |
| 8 | `Dmesg` (streaming) | ✅ | ❌ | Kernel logs for diagnostics |

#### Priority 3: Production Day-2

| # | Feature | mTLS | Status | Description |
| - | ------- | ---- | ------ | ----------- |
| 9 | `Upgrade` | ✅ | ❌ | Talos version upgrades |
| 10 | `Version` (remote) | ✅ | ✅ | Remote version check |

#### Non-gRPC Operations (Out of Scope for Library)

These are **local CLI operations**, not gRPC APIs:

| Operation | Notes |
| --------- | ----- |
| `gen config` | Generates YAML files locally (consider separate helper) |
| `config endpoint` | Manipulates local talosconfig |
| `config node` | Manipulates local talosconfig |
| `cluster create/destroy` | Docker provider (test harness only) |

---

### Phase 3: Extended APIs

**Goal**: Complete API coverage for advanced operations.

| Feature | Priority | Description |
| ------- | -------- | ----------- |
| Service Control | 🟡 High | Start, Stop, Restart services |
| Logs API | 🟡 High | Service log streaming |
| Events API | 🟡 High | Cluster event stream |
| etcd Snapshot | 🟡 High | Backup etcd data |
| etcd Recover | 🟡 High | Restore from snapshot |
| File Operations | 🟢 Medium | Read, List, Copy, DiskUsage |
| System Info | 🟢 Medium | Memory, CPU, Disk, Network stats |
| Process List | 🟢 Medium | Running processes |
| Packet Capture | 🟢 Low | Network debugging |
| Netstat | 🟢 Low | Network connections |

---

### Phase 4: Production Readiness & crates.io

**Goal**: Production-grade library with public release.

| Feature | Description |
| ------- | ----------- |
| Connection pooling | Multiple endpoint support with failover |
| Retry policies | Configurable retry with exponential backoff |
| Timeouts | Per-request and global timeouts |
| Interceptors | Logging, metrics, tracing hooks |
| Resource wrappers | High-level Rust types over Protobuf |
| Builder patterns | Fluent API for complex requests |
| Full documentation | docs.rs ready, examples |
| crates.io release | Public package publication |
| MSRV policy | Minimum Supported Rust Version |

---

## mTLS Requirement Summary

```text
┌─────────────────────────────────────────────────────────────┐
│  mTLS NOT required (--insecure or local ops):               │
│  • ApplyConfiguration (maintenance mode with --insecure)    │
│  • gen config (local CLI, not gRPC)                         │
│  • config endpoint/node (local talosconfig manipulation)    │
│  • cluster create/destroy (Docker provider)                 │
│  • version --client (local CLI)                             │
├─────────────────────────────────────────────────────────────┤
│  mTLS REQUIRED (post-bootstrap operations):                 │
│  • Bootstrap, Kubeconfig, Reset, Health                     │
│  • EtcdRemoveMember, Upgrade, Dmesg, Logs                   │
│  • All remote API calls after bootstrap                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Known Issues & Technical Debt

### 🔴 Critical: ED25519 Certificate Support

**Problem**: Talos generates ED25519 client certificates by default. The current rustls configuration does not properly handle ED25519 for client authentication.

**Symptoms**:

```text
mTLS connection failed: Transport error: transport error
received fatal alert: CertificateRequired
```

**Root Cause**:

- Talos uses `ED25519` for all PKI (Signature Algorithm: ED25519)
- `tonic`'s default TLS config expects RSA/ECDSA certificates
- PEM parsing works, but the TLS handshake fails during client cert presentation

**Potential Solutions**:

1. **Configure rustls with ED25519 support** - Requires proper `CryptoProvider` setup
2. **Use `ring` crypto provider** - May have better ED25519 support
3. **Alternative: `native-tls`** - Switch from rustls to OpenSSL-based TLS
4. **Workaround: RSA certs** - Generate Talos cluster with RSA (non-standard)

**Impact**: Full mTLS authentication is blocked until resolved.

---

### 🟡 Medium: Streaming gRPC Methods

**Problem**: Several Talos APIs use server-streaming gRPC (Logs, Events, Kubeconfig, etcd Snapshot, etc.). These require different handling than unary calls.

**Current State**: Not implemented.

**Required Changes**:

- Return `tonic::Streaming<T>` instead of `Response<T>`
- Handle stream lifecycle (cancellation, errors, completion)
- Provide async iterator/stream wrapper for ergonomic usage

**Affected APIs**:

- `Logs`, `LogsContainers`
- `Events`
- `Dmesg`
- `Kubeconfig`
- `EtcdSnapshot`
- `Copy`, `Read`
- `List` (file listing)
- `DiskUsage`
- `PacketCapture`

---

### 🟡 Medium: Client-Streaming gRPC Methods

**Problem**: Some APIs require client-to-server streaming (e.g., `EtcdRecover`).

**Current State**: Not implemented.

**Required Changes**:

- Accept `impl Stream<Item = T>` as input
- Handle backpressure and flow control

**Affected APIs**:

- `EtcdRecover` (upload snapshot)

---

### 🟢 Low: Multi-Node Targeting

**Problem**: Talos API supports targeting multiple nodes in a single request via metadata. Current implementation targets single endpoints.

**Current State**: Single-endpoint only.

**Required Changes**:

- Implement gRPC metadata for node targeting
- Handle multi-node responses (responses contain per-node results)
- Consider connection multiplexing

---

### 🟢 Low: Error Handling Granularity

**Problem**: Current error types are basic. Talos returns rich error information that should be preserved.

**Current State**: Generic `TalosError` enum.

**Required Changes**:

- Parse `google.rpc.Status` details
- Expose node-specific errors from multi-node responses
- Categorize errors (retriable vs permanent)

---

### 🟢 Low: Generated Code Organization

**Problem**: Generated Protobuf code is large (~6000 lines for machine.rs alone). IDE performance may suffer.

**Observation**: The `machine.proto` generates extensive code because it includes 60+ RPC methods.

**Potential Improvements**:

- Split into feature-gated modules
- Lazy loading of sub-clients
- Consider code generation optimizations

---

## API Surface Tracking

### Machine Service (machine.proto)

| Method | Phase | Implemented | Tested |
| ------ | ----- | ----------- | ------ |
| ApplyConfiguration | 2 | ❌ | ❌ |
| Bootstrap | 3 | ❌ | ❌ |
| Containers | 2 | ❌ | ❌ |
| Copy | 2 | ❌ | ❌ |
| CPUFreqStats | 2 | ❌ | ❌ |
| CPUInfo | 2 | ❌ | ❌ |
| DiskStats | 2 | ❌ | ❌ |
| Dmesg | 2 | ❌ | ❌ |
| Events | 2 | ❌ | ❌ |
| EtcdMemberList | 3 | ❌ | ❌ |
| EtcdRemoveMemberByID | 3 | ❌ | ❌ |
| EtcdLeaveCluster | 3 | ❌ | ❌ |
| EtcdForfeitLeadership | 3 | ❌ | ❌ |
| EtcdRecover | 3 | ❌ | ❌ |
| EtcdSnapshot | 3 | ❌ | ❌ |
| EtcdAlarmList | 3 | ❌ | ❌ |
| EtcdAlarmDisarm | 3 | ❌ | ❌ |
| EtcdDefragment | 3 | ❌ | ❌ |
| EtcdStatus | 3 | ❌ | ❌ |
| Hostname | 1 | ✅ | ✅ |
| Kubeconfig | 3 | ❌ | ❌ |
| List | 2 | ❌ | ❌ |
| DiskUsage | 2 | ❌ | ❌ |
| LoadAvg | 2 | ❌ | ❌ |
| Logs | 2 | ❌ | ❌ |
| LogsContainers | 2 | ❌ | ❌ |
| Memory | 2 | ❌ | ❌ |
| Mounts | 2 | ❌ | ❌ |
| NetworkDeviceStats | 2 | ❌ | ❌ |
| Processes | 2 | ❌ | ❌ |
| Read | 2 | ❌ | ❌ |
| Reboot | 1 | ✅ | ✅ |
| Restart | 2 | ❌ | ❌ |
| Rollback | 2 | ❌ | ❌ |
| Reset | 2 | ❌ | ❌ |
| ServiceList | 1 | ✅ | ✅ |
| ServiceRestart | 2 | ❌ | ❌ |
| ServiceStart | 2 | ❌ | ❌ |
| ServiceStop | 2 | ❌ | ❌ |
| Shutdown | 1 | ✅ | ✅ |
| Stats | 2 | ❌ | ❌ |
| SystemStat | 1 | ✅ | ✅ |
| Upgrade | 2 | ❌ | ❌ |
| Version | 1 | ✅ | ✅ |
| GenerateClientConfiguration | 3 | ❌ | ❌ |
| PacketCapture | 3 | ❌ | ❌ |
| Netstat | 3 | ❌ | ❌ |
| MetaWrite | 3 | ❌ | ❌ |
| MetaDelete | 3 | ❌ | ❌ |
| ImageList | 2 | ❌ | ❌ |
| ImagePull | 2 | ❌ | ❌ |

### Version Service (version.proto)

| Method | Phase | Implemented | Tested |
| ------ | ----- | ----------- | ------ |
| Version | 1 | ✅ | ✅ |

---

## Dependencies & Version Tracking

| Dependency | Current | Purpose | Notes |
| ---------- | ------- | ------- | ----- |
| tonic | 0.12 | gRPC framework | |
| prost | 0.13 | Protobuf codegen | |
| tokio | 1.x | Async runtime | |
| rustls | 0.23 | TLS implementation | ED25519 issue |
| tokio-rustls | 0.26 | Async TLS | |
| hyper-util | 0.1 | HTTP utilities | Custom connector |

### Talos Protobuf Sources

| Proto | Source | Version |
| ----- | ------ | ------- |
| machine.proto | github.com/siderolabs/talos | main |
| common.proto | github.com/siderolabs/talos | main |
| version.proto | github.com/siderolabs/talos | main |
| google/rpc/status.proto | googleapis | - |

**Update Process**:

1. Check Talos releases for API changes
2. Download updated protos
3. Regenerate Rust code (`cargo build`)
4. Update API surface tracking table
5. Implement new methods
6. Update version in docs
