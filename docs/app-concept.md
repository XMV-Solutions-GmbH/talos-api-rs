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

> **Updated**: 2026-01-27 (v0.2.0 released)

### Phase 1: Core Foundation ✅ COMPLETE

**Goal**: Establish a working client with basic connectivity and essential APIs.

| Feature | Status | Notes |
| ------- | ------ | ----- |
| Project scaffolding | ✅ | Cargo workspace, CI/CD |
| TalosClient core | ✅ | Connection management |
| TLS/mTLS config | ✅ | Certificate loading, ED25519 support |
| Insecure mode | ✅ | Custom TLS verifier |
| Version API | ✅ | Health check capability |
| Machine API (basic) | ✅ | ServiceList, Hostname, SystemStat, Reboot, Shutdown |
| Unit tests | ✅ | 200 tests covering all modules |
| Integration harness | ✅ | Docker-based Talos cluster |

---

### Phase 2: Alpha Release (Cluster Lifecycle Core) ✅ COMPLETE

**Goal**: Enable core cluster lifecycle operations.

#### Priority 1: Absolute Core ✅ ALL COMPLETE

| # | Feature | mTLS | Status | Description |
| - | ------- | ---- | ------ | ----------- |
| 1 | `gen config` | ❌ | N/A | Machine config generation (NOT gRPC - CLI only) |
| 2 | `ApplyConfiguration` | ❌/✅ | ✅ | Initial config (insecure) + updates (mTLS) |
| 3 | `Bootstrap` | ✅ | ✅ | Initialize etcd on first control-plane |
| 4 | `Kubeconfig` (streaming) | ✅ | ✅ | Retrieve kubeconfig |
| 5 | `Reset --graceful` | ✅ | ✅ | Graceful node teardown |

**ED25519 mTLS**: ✅ RESOLVED with ring crypto provider (PR #7)

#### Priority 2: Beta Operations ✅ ALL COMPLETE

| # | Feature | mTLS | Status | Description |
| - | ------- | ---- | ------ | ----------- |
| 6 | Health check API | ✅ | ✅ | Version API for health checks |
| 7 | `EtcdRemoveMember` | ✅ | ✅ | Control-plane scale-down |
| 8 | `Dmesg` (streaming) | ✅ | ✅ | Kernel logs for diagnostics |

#### Priority 3: Production Day-2 ✅ ALL COMPLETE

| # | Feature | mTLS | Status | Description |
| - | ------- | ---- | ------ | ----------- |
| 9 | `Upgrade` | ✅ | ✅ | Talos version upgrades |
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

### Phase 3: Extended APIs ✅ COMPLETE

**Goal**: Complete API coverage for advanced operations.

| Feature | Priority | Status | Description |
| ------- | -------- | ------ | ----------- |
| Service Control | 🟡 High | ✅ | Start, Stop, Restart services |
| Logs API | 🟡 High | ✅ | Service log streaming |
| Events API | 🟡 High | ❌ | Cluster event stream (not implemented) |
| etcd Snapshot | 🟡 High | ❌ | Backup etcd data (not implemented) |
| etcd Recover | 🟡 High | ❌ | Restore from snapshot (client-streaming) |
| File Operations | 🟢 Medium | ✅ | Read, List, Copy, DiskUsage |
| System Info | 🟢 Medium | ✅ | Memory, CPU, Disk, Network stats |
| Process List | 🟢 Medium | ✅ | Running processes |
| Packet Capture | 🟢 Low | ✅ | Network debugging |
| Netstat | 🟢 Low | ✅ | Network connections |
| **ImageList/ImagePull** | 🟢 Medium | ✅ | Container image management (v0.2.0) |

---

### Phase 4: Production Readiness & crates.io ✅ COMPLETE

**Goal**: Production-grade library with public release.

| Feature | Status | Description |
| ------- | ------ | ----------- |
| Connection pooling | ✅ | Multiple endpoint support with failover |
| Retry policies | ✅ | Configurable retry with exponential backoff |
| Timeouts | ✅ | Per-request and global timeouts |
| Interceptors | ✅ | Logging, metrics, tracing hooks |
| Resource wrappers | ✅ | High-level Rust types over Protobuf |
| Builder patterns | ✅ | Fluent API for complex requests |
| Full documentation | ✅ | docs.rs ready, examples |
| crates.io release | ✅ | Published: v0.1.0, v0.1.1, v0.1.2, v0.2.0 |
| MSRV policy | ✅ | Rust 1.82+ (v0.2.0) |

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
│                                                             │
│  ED25519 Support: ✅ WORKING (ring crypto provider)         │
└─────────────────────────────────────────────────────────────┘
```

---

## Known Issues & Technical Debt

### ✅ RESOLVED: ED25519 Certificate Support

**Status**: Fixed in PR #7 with ring crypto provider.

Talos ED25519 client certificates are now fully supported via custom rustls configuration with the `ring` crypto provider.

---

### ✅ RESOLVED: Streaming gRPC Methods

**Status**: Server-streaming implemented for most APIs.

Implemented streaming APIs with chunk assembly:

- ✅ `Kubeconfig` - Assembles kubeconfig chunks
- ✅ `Dmesg` - Kernel message streaming
- ✅ `Logs` - Service log streaming
- ✅ `Copy`, `Read` - File content streaming
- ✅ `List`, `DiskUsage` - File listing streaming
- ✅ `PacketCapture` - Network capture streaming
- ✅ `ImageList` - Container image listing

**Remaining**:

- ❌ `Events` - Not implemented
- ❌ `EtcdSnapshot` - Not implemented

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

### ✅ RESOLVED: Multi-Node Targeting

**Status**: Fixed in v0.1.2.

Multi-node targeting implemented via gRPC metadata:

- `NodeTarget` enum for specifying target nodes
- `with_node()` / `with_nodes()` methods on `TalosClient`
- gRPC metadata `x-talos-node` header support
- Cluster-wide operations support

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

> **Updated**: 2026-01-27 (v0.2.0)

### Machine Service (machine.proto)

| Method | Phase | Implemented | Tested | Notes |
| ------ | ----- | ----------- | ------ | ----- |
| ApplyConfiguration | 2 | ✅ | ✅ | Dry-run, mode selection |
| Bootstrap | 2 | ✅ | ✅ | Recovery options |
| Containers | 3 | ❌ | ❌ | |
| Copy | 3 | ✅ | ✅ | Server-streaming |
| CPUFreqStats | 3 | ❌ | ❌ | |
| CPUInfo | 3 | ✅ | ✅ | |
| DiskStats | 3 | ✅ | ✅ | |
| DiskUsage | 3 | ✅ | ✅ | Server-streaming |
| Dmesg | 2 | ✅ | ✅ | Server-streaming |
| Events | 3 | ❌ | ❌ | Server-streaming |
| EtcdAlarmDisarm | 3 | ✅ | ✅ | |
| EtcdAlarmList | 3 | ✅ | ✅ | |
| EtcdDefragment | 3 | ✅ | ✅ | |
| EtcdForfeitLeadership | 3 | ✅ | ✅ | |
| EtcdLeaveCluster | 3 | ✅ | ✅ | |
| EtcdMemberList | 3 | ✅ | ✅ | |
| EtcdRecover | 3 | ❌ | ❌ | Client-streaming |
| EtcdRemoveMemberByID | 3 | ✅ | ✅ | |
| EtcdSnapshot | 3 | ❌ | ❌ | Server-streaming |
| EtcdStatus | 3 | ✅ | ✅ | |
| GenerateClientConfiguration | 3 | ✅ | ✅ | |
| Hostname | 1 | ✅ | ✅ | |
| ImageList | 2 | ✅ | ✅ | v0.2.0 |
| ImagePull | 2 | ✅ | ✅ | v0.2.0 |
| Kubeconfig | 2 | ✅ | ✅ | Server-streaming |
| List | 3 | ✅ | ✅ | Server-streaming |
| LoadAvg | 3 | ✅ | ✅ | |
| Logs | 2 | ✅ | ✅ | Server-streaming |
| LogsContainers | 3 | ❌ | ❌ | |
| Memory | 3 | ✅ | ✅ | |
| MetaDelete | 4 | ❌ | ❌ | |
| MetaWrite | 4 | ❌ | ❌ | |
| Mounts | 3 | ✅ | ✅ | |
| Netstat | 3 | ✅ | ✅ | Filtering support |
| NetworkDeviceStats | 3 | ✅ | ✅ | |
| PacketCapture | 3 | ✅ | ✅ | Server-streaming |
| Processes | 3 | ✅ | ✅ | |
| Read | 3 | ✅ | ✅ | Server-streaming |
| Reboot | 1 | ✅ | ✅ | |
| Reset | 2 | ✅ | ✅ | Graceful/Force/Halt |
| Restart | 3 | ❌ | ❌ | Container restart |
| Rollback | 3 | ✅ | ✅ | |
| ServiceList | 1 | ✅ | ✅ | |
| ServiceRestart | 3 | ✅ | ✅ | |
| ServiceStart | 3 | ✅ | ✅ | |
| ServiceStop | 3 | ✅ | ✅ | |
| Shutdown | 1 | ✅ | ✅ | |
| Stats | 3 | ❌ | ❌ | |
| SystemStat | 1 | ✅ | ✅ | |
| Upgrade | 2 | ✅ | ✅ | Reboot mode options |
| Version | 1 | ✅ | ✅ | |

**Coverage**: 43/52 methods implemented (83%)

### Version Service (version.proto)

| Method | Phase | Implemented | Tested |
| ------ | ----- | ----------- | ------ |
| Version | 1 | ✅ | ✅ |

### Not Implemented (Planned)

| Method | Reason | Priority |
| ------ | ------ | -------- |
| Containers | Low demand | Low |
| CPUFreqStats | Low demand | Low |
| Events | Streaming complexity | Medium |
| EtcdRecover | Client-streaming | Medium |
| EtcdSnapshot | Streaming complexity | Medium |
| LogsContainers | Low demand | Low |
| MetaDelete | Advanced use case | Low |
| MetaWrite | Advanced use case | Low |
| Restart | Container-specific | Low |
| Stats | Overlaps with other APIs | Low |

---

## Dependencies & Version Tracking

| Dependency | Current | Purpose | Notes |
| ---------- | ------- | ------- | ----- |
| tonic | 0.14 | gRPC framework | Updated v0.2.0 |
| tonic-prost | 0.14 | Prost integration | New in v0.2.0 |
| prost | 0.14 | Protobuf codegen | Updated v0.2.0 |
| tokio | 1.x | Async runtime | |
| rustls | 0.23 | TLS implementation | ED25519 ✅ fixed |
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
