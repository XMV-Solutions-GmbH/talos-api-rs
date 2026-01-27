<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Application Concept

A typed, async, idiomatic Rust client for the Talos Linux gRPC API.

## Overview

`talos-api-rs` provides a production-grade Rust client for interacting with [Talos Linux](https://www.talos.dev/) clusters via gRPC. It's designed for building automation tools, operators, and desktop applications that manage Talos infrastructure.

### What This Library Does

- **Pure gRPC Client** — Direct communication with Talos API (port 50000)
- **Typed APIs** — Strongly-typed Rust wrappers over raw Protobuf
- **Production-Ready** — Connection pooling, retries, circuit breakers, metrics, tracing
- **mTLS Support** — Full ED25519 certificate support (Talos default)

### What This Library Does NOT Do

- ❌ Not a replacement for `talosctl`
- ❌ No config generation (`talosctl gen config` is local CLI)
- ❌ No opinionated workflows or orchestration logic
- ❌ No YAML/UI abstractions

---

## Core Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     Your Application                        │
├─────────────────────────────────────────────────────────────┤
│                      TalosClient                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ NodeTarget  │  │ Config      │  │ ConnectionPool      │ │
│  │ (multi-node)│  │ (talosconfig)│ │ (health routing)    │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Resource Wrappers                        │
│  Bootstrap │ Config │ Etcd │ Services │ Files │ System    │
├─────────────────────────────────────────────────────────────┤
│                    Runtime Layer                            │
│  RetryConfig │ CircuitBreaker │ Metrics │ Tracing          │
├─────────────────────────────────────────────────────────────┤
│                    gRPC (tonic + prost)                     │
│                    TLS (rustls + ring)                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
                    Talos API (port 50000)
```

---

## Authentication

### mTLS (Production)

Talos requires mutual TLS for all post-bootstrap API calls. This library fully supports Talos ED25519 certificates via the `ring` crypto provider.

```rust
let client = TalosClient::from_talosconfig(None, None).await?;
// or
let config = TalosClientConfig::builder("https://10.0.0.1:50000")
    .ca_cert("/path/to/ca.crt")
    .client_cert("/path/to/client.crt")
    .client_key("/path/to/client.key")
    .build();
```

### Insecure Mode (Maintenance)

For nodes in maintenance mode (no config yet), TLS verification can be skipped:

```rust
let config = TalosClientConfig {
    endpoint: "https://192.168.1.100:50000".to_string(),
    insecure: true,
    ..Default::default()
};
```

---

## API Coverage

**43 of 52 Machine Service methods implemented (83%)**

### Cluster Lifecycle

| API | Description |
| --- | ----------- |
| `ApplyConfiguration` | Apply machine config (dry-run, reboot modes) |
| `Bootstrap` | Initialize etcd cluster |
| `Kubeconfig` | Retrieve kubeconfig (streaming) |
| `Reset` | Reset node (graceful, force, halt) |
| `Upgrade` | Upgrade Talos version |
| `Rollback` | Rollback to previous config |

### etcd Operations

| API | Description |
| --- | ----------- |
| `EtcdMemberList` | List cluster members |
| `EtcdRemoveMemberByID` | Remove member |
| `EtcdLeaveCluster` | Node leaves cluster |
| `EtcdForfeitLeadership` | Force leader election |
| `EtcdStatus` | Cluster health |
| `EtcdAlarmList/Disarm` | Alarm management |
| `EtcdDefragment` | Defragment database |

### System Information

| API | Description |
| --- | ----------- |
| `Version` | Talos version |
| `Hostname` | Node hostname |
| `Memory`, `CPUInfo`, `LoadAvg` | Resource stats |
| `DiskStats`, `Mounts` | Storage info |
| `NetworkDeviceStats` | Network interfaces |
| `Processes` | Running processes |
| `SystemStat` | System statistics |

### Services & Logs

| API | Description |
| --- | ----------- |
| `ServiceList` | List services |
| `ServiceStart/Stop/Restart` | Control services |
| `Logs` | Service logs (streaming) |
| `Dmesg` | Kernel logs (streaming) |

### File Operations

| API | Description |
| --- | ----------- |
| `List` | Directory listing |
| `Read` | File content |
| `Copy` | Copy files (tar) |
| `DiskUsage` | Disk usage |

### Container Images

| API | Description |
| --- | ----------- |
| `ImageList` | List images |
| `ImagePull` | Pull images |

### Diagnostics

| API | Description |
| --- | ----------- |
| `Netstat` | Network connections |
| `PacketCapture` | Network capture |
| `GenerateClientConfiguration` | Generate talosconfig |

### Not Implemented

| API | Reason |
| --- | ------ |
| `Events` | Server-streaming complexity |
| `EtcdSnapshot` | Server-streaming complexity |
| `EtcdRecover` | Client-streaming required |
| `Containers`, `LogsContainers` | Low demand |
| `CPUFreqStats`, `Stats`, `Restart` | Low demand |
| `MetaWrite`, `MetaDelete` | Advanced use case |

---

## Multi-Node Operations

Target specific nodes in a cluster:

```rust
// Single node
let client = client.with_node("10.0.0.5");

// Multiple nodes
let client = client.with_nodes(vec!["10.0.0.5", "10.0.0.6"]);
```

### Cluster Discovery

Discover cluster members and check health:

```rust
let discovery = ClusterDiscovery::from_endpoint("https://10.0.0.1:50000")
    .ca_cert("/path/to/ca.crt")
    .client_cert("/path/to/client.crt")
    .client_key("/path/to/client.key")
    .build()
    .await?;

let members = discovery.discover_members().await?;
let health = discovery.check_cluster_health().await?;
```

---

## Production Features

### Connection Pool

```rust
let pool_config = ConnectionPoolConfig::builder()
    .max_endpoints(5)
    .health_check_interval(Duration::from_secs(30))
    .load_balancer(LoadBalancer::LeastFailures)
    .build();
```

### Retry & Circuit Breaker

```rust
let retry = RetryConfig::builder()
    .max_retries(3)
    .backoff(BackoffStrategy::Exponential { base: Duration::from_millis(100) })
    .build();

let circuit_breaker = CircuitBreakerConfig::builder()
    .failure_threshold(5)
    .recovery_timeout(Duration::from_secs(30))
    .build();
```

### Observability

```rust
// Prometheus metrics
let metrics = MetricsCollector::new(MetricsConfig::default());
metrics.record_request("Version", "10.0.0.1:50000", true, Duration::from_millis(42));
println!("{}", metrics.to_prometheus_text());

// OpenTelemetry tracing
let span = TalosSpan::new("machine.Version", "10.0.0.1:50000");
```

---

## Dependencies

| Dependency | Version | Purpose |
| ---------- | ------- | ------- |
| tonic | 0.14 | gRPC framework |
| tonic-prost | 0.14 | Prost integration |
| prost | 0.14 | Protobuf codegen |
| tokio | 1.x | Async runtime |
| rustls | 0.23 | TLS (ring provider for ED25519) |

### Talos Compatibility

- **Minimum**: Talos 1.9.x
- **Protobuf Source**: github.com/siderolabs/talos

---

## Non-Goals & Out of Scope

These operations are **local CLI functions**, not gRPC APIs:

| Operation | Alternative |
| --------- | ----------- |
| `talosctl gen config` | Generate YAML locally |
| `talosctl config endpoint` | Manipulate local talosconfig |
| `talosctl cluster create` | Docker provider (test only) |
| Config validation | Consider separate schema library |

---

## Further Reading

- [Architecture](architecture.md) — Technical implementation details
- [API Stability](api-stability.md) — Versioning and stability guarantees
- [Testing Guide](testing.md) — How to run tests
- [Talos Documentation](https://www.talos.dev/docs/) — Official Talos docs
