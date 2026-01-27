<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Architecture

> **Updated**: 2026-01-27 (v0.2.0)

## Directory Structure

```text
talos-api-rs/
├── src/
│   ├── lib.rs              # Crate root, public exports
│   ├── client/
│   │   ├── mod.rs          # TalosClient, TalosClientConfig
│   │   ├── pool.rs         # ConnectionPool, load balancing
│   │   ├── node_target.rs  # NodeTarget for multi-node operations
│   │   ├── discovery.rs    # ClusterDiscovery, ClusterHealth
│   │   └── tests.rs        # Unit tests
│   ├── config/
│   │   ├── mod.rs          # TalosClientConfig, TalosClientConfigBuilder
│   │   └── talosconfig.rs  # TalosConfig (~/.talos/config parser)
│   ├── api/
│   │   ├── mod.rs          # API module exports
│   │   └── generated/      # Protobuf-generated code (DO NOT EDIT)
│   │       ├── mod.rs
│   │       ├── version.rs
│   │       ├── machine.rs
│   │       ├── common.rs
│   │       └── google.rpc.rs
│   ├── resources/          # Typed wrappers for Talos APIs
│   │   ├── mod.rs
│   │   ├── bootstrap.rs    # BootstrapRequest, BootstrapResponse
│   │   ├── configuration.rs # ApplyConfiguration types
│   │   ├── kubeconfig.rs   # KubeconfigResponse
│   │   ├── reset.rs        # ResetRequest, WipeMode
│   │   ├── etcd.rs         # EtcdMemberList, EtcdStatus, etc.
│   │   ├── dmesg.rs        # DmesgRequest, DmesgResponse
│   │   ├── upgrade.rs      # UpgradeRequest, UpgradeRebootMode
│   │   ├── services.rs     # ServiceStart/Stop/Restart
│   │   ├── logs.rs         # LogsRequest, ContainerDriver
│   │   ├── system.rs       # Memory, CPU, LoadAvg, etc.
│   │   ├── files.rs        # List, Read, Copy, DiskUsage
│   │   ├── advanced.rs     # Netstat, PacketCapture, Rollback
│   │   └── images.rs       # ImageList, ImagePull
│   ├── runtime/            # Production-grade utilities
│   │   ├── mod.rs
│   │   ├── retry.rs        # RetryConfig, backoff strategies
│   │   ├── circuit_breaker.rs # CircuitBreaker pattern
│   │   ├── logging.rs      # LoggingInterceptor, RequestLogger
│   │   ├── metrics.rs      # MetricsCollector, Prometheus format
│   │   └── tracing.rs      # TalosSpan, SpanFactory, OpenTelemetry
│   ├── error/
│   │   └── mod.rs          # TalosError enum
│   └── testkit/
│       └── mod.rs          # TalosCluster for integration tests
├── proto/
│   ├── common/
│   │   ├── version.proto
│   │   └── common.proto
│   ├── machine/
│   │   └── machine.proto
│   └── google/rpc/
│       └── status.proto
├── examples/
│   ├── version.rs          # Basic version check
│   ├── machine.rs          # Machine operations
│   ├── mtls_test.rs        # mTLS connection test
│   ├── resilient_client.rs # Connection pool, retries, circuit breaker
│   └── monitoring_metrics.rs # Prometheus metrics demo
├── tests/
│   └── integration_test.rs # 20 integration tests
├── docs/
│   ├── app-concept.md      # Concept and roadmap
│   ├── architecture.md     # This file
│   ├── api-stability.md    # API stability guarantees
│   ├── testing.md          # Testing guide
│   ├── release-checklist.md
│   ├── release-preparations.md
│   └── todo.md             # Development tracking
└── build.rs                # Protobuf compilation
```

## Layers

### 1. Transport Layer (client/)

Handles TLS/mTLS, connection management, and channel creation.

```rust
TalosClientConfig {
    endpoint: String,       // e.g., "https://127.0.0.1:50000"
    ca_cert: Option<PathBuf>,
    client_cert: Option<PathBuf>,
    client_key: Option<PathBuf>,
    insecure: bool,         // Skip TLS verification
    connect_timeout: Duration,
    request_timeout: Duration,
}
```

**Key Components**:

- `TalosClient::new()` - Async constructor with TLS setup
- `TalosClient::from_talosconfig()` - Create from ~/.talos/config
- `TalosClientConfig::builder()` - Fluent configuration API
- `NodeTarget` - Multi-node targeting via gRPC metadata
- `ConnectionPool` - Multi-endpoint with health-based routing
- Custom `ring` crypto provider for ED25519 mTLS support

### 2. API Layer (api/)

Pure generated code from Protobuf. **NEVER EDIT MANUALLY**.

**Generated Clients**:

- `VersionServiceClient` - Health/version checks
- `MachineServiceClient` - Machine operations (60+ methods)

**Access Pattern**:

```rust
let client = TalosClient::new(config).await?;

// Direct API calls
let version = client.version().await?;
let hostname = client.hostname().await?;

// With node targeting
let client = client.with_node("10.0.0.5");
let services = client.service_list().await?;
```

### 3. Resources Layer (resources/)

Typed wrappers providing ergonomic APIs over raw Protobuf types.

```rust
// Instead of raw protobuf
let request = ApplyConfigurationRequest::new(yaml_config)
    .with_mode(ApplyMode::Reboot)
    .dry_run();

let response = client.apply_configuration(request).await?;
```

### 4. Runtime Layer (runtime/)

Production-grade utilities for resilient operations.

| Component | Purpose |
| --------- | ------- |
| `RetryConfig` | Configurable retry with exponential/linear/fixed backoff |
| `CircuitBreaker` | Prevent cascade failures (Closed/Open/HalfOpen states) |
| `LoggingInterceptor` | Request/response logging with sensitive data redaction |
| `MetricsCollector` | Prometheus-compatible metrics (counters, histograms) |
| `TalosSpan` / `SpanFactory` | OpenTelemetry distributed tracing |

### 5. Error Layer (error/)

Structured error handling.

```rust
pub enum TalosError {
    Config(String),
    Transport(tonic::transport::Error),
    Grpc(tonic::Status),
    Io(std::io::Error),
    Certificate(String),
}
```

### 6. Test Infrastructure (testkit/)

Integration test harness using Docker-based Talos clusters.

```rust
pub struct TalosCluster {
    pub name: String,
    pub endpoint: String,
    pub talosconfig_path: PathBuf,
    pub ca_path: PathBuf,
    pub crt_path: PathBuf,
    pub key_path: PathBuf,
}
```

### 4. Test Infrastructure (testkit/)

Integration test harness using Docker-based Talos clusters.

```rust
pub struct TalosCluster {
    pub name: String,
    pub endpoint: String,
    pub talosconfig_path: PathBuf,
    pub ca_path: PathBuf,
    pub crt_path: PathBuf,
    pub key_path: PathBuf,
}
```

**Workflow**:

1. `TalosCluster::create()` - Provisions cluster via `talosctl cluster create docker`
2. Extracts certificates from generated talosconfig
3. Provides endpoint for client testing
4. `Drop` implementation destroys cluster automatically

## Data Flow

```text
User Code
    │
    ▼
TalosClient (holds tonic::Channel)
    │
    ├─► NodeTarget (optional, sets x-talos-node header)
    │
    ▼
MachineServiceClient<Channel> (generated by tonic)
    │
    ▼
tonic::Channel (HTTP/2 + TLS via rustls with ring)
    │
    ▼
Talos API (gRPC on port 50000, requires mTLS)
```

## TLS Configuration

### Standard mTLS Mode (ED25519 Supported)

```rust
// Using ring crypto provider for ED25519 support
let config = TalosClientConfig::builder("https://192.168.1.100:50000")
    .ca_cert("/path/to/ca.crt")
    .client_cert("/path/to/client.crt")
    .client_key("/path/to/client.key")
    .build();
```

### From talosconfig

```rust
// Automatically loads certs from ~/.talos/config
let client = TalosClient::from_talosconfig(None, None).await?;
```

### Insecure Mode

For maintenance mode (nodes without config):

```rust
let config = TalosClientConfig {
    endpoint: "https://192.168.1.100:50000".to_string(),
    insecure: true,
    ..Default::default()
};
```

## Testing Strategy

### Unit Tests (200 tests)

- Test all typed wrappers and builders
- No external dependencies
- Fast execution (~0.1s)

### Integration Tests (20 tests)

- Require `TALOS_DEV_TESTS=1`
- Provision real Docker-based Talos cluster
- Test all API categories
- Automatic cleanup via Drop
