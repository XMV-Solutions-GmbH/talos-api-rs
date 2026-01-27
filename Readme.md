# talos-api-rs

![Crates.io](https://img.shields.io/crates/v/talos-api-rs)
![Downloads](https://img.shields.io/crates/d/talos-api-rs)
![docs.rs](https://img.shields.io/docsrs/talos-api-rs)
![License](https://img.shields.io/crates/l/talos-api-rs)
[![CI](https://github.com/XMV-Solutions-GmbH/talos-api-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/XMV-Solutions-GmbH/talos-api-rs/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.82-blue.svg)](https://www.rust-lang.org)

A typed, async, idiomatic Rust client for SideroLabs [Talos Linux](https://www.talos.dev/) gRPC API.

Built for **production use** with connection pooling, circuit breakers, retry policies,
Prometheus metrics, and OpenTelemetry tracing. Inspired by [`kube-rs`](https://github.com/kube-rs/kube).

---

## Features

- **40+ APIs** — Machine, etcd, system, files, diagnostics
- **Async-first** — Built on `tokio` and `tonic`
- **Strongly typed** — No stringly-typed API calls
- **Production-ready** — Retries, circuit breakers, connection pooling
- **Observable** — Prometheus metrics, OpenTelemetry tracing
- **mTLS support** — ED25519 certificates (Talos default)

## Non-Goals

- ❌ Not a replacement for `talosctl`
- ❌ No opinionated workflows (bootstrap orchestration, etc.)
- ❌ No CLI/UI layer

---

## API Coverage

| Category | APIs | Status |
| -------- | ---- | ------ |
| **Machine** | Version, Hostname, Reboot, Shutdown, Upgrade, Rollback | ✅ |
| **Configuration** | ApplyConfiguration, GenerateConfiguration | ✅ |
| **Cluster** | Bootstrap, Kubeconfig, Reset, ClusterDiscovery | ✅ |
| **Services** | ServiceList, ServiceStart, ServiceStop, ServiceRestart | ✅ |
| **etcd** | MemberList, Status, AlarmList, Defragment, ForfeitLeadership | ✅ |
| **System** | Memory, CPUInfo, LoadAvg, DiskStats, Mounts, NetworkDeviceStats, Processes | ✅ |
| **Images** | ImageList, ImagePull | ✅ |
| **Files** | List, Read, Copy, DiskUsage | ✅ |
| **Diagnostics** | Dmesg, Logs, Netstat, PacketCapture | ✅ |

See [docs/todo.md](docs/todo.md) for the full roadmap.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
talos-api-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use talos_api_rs::{TalosClient, TalosClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect with insecure mode (for testing/maintenance)
    let config = TalosClientConfig {
        endpoint: "https://192.168.1.100:50000".to_string(),
        insecure: true,
        ..Default::default()
    };

    let client = TalosClient::new(config).await?;
    
    // Check version
    let version = client.version().await?;
    println!("Talos version: {}", version.into_inner().version);
    
    Ok(())
}
```

### With mTLS (Production)

```rust
use talos_api_rs::{TalosClient, TalosClientConfig};

let config = TalosClientConfig {
    endpoint: "https://192.168.1.100:50000".to_string(),
    ca_cert: Some("/path/to/ca.crt".into()),
    client_cert: Some("/path/to/client.crt".into()),
    client_key: Some("/path/to/client.key".into()),
    insecure: false,
    ..Default::default()
};

let client = TalosClient::new(config).await?;
```

### Production Client with Resilience

```rust
use talos_api_rs::{TalosClientConfig, TalosClient, RetryConfig, CircuitBreakerConfig};
use std::time::Duration;

// Configure with timeouts, retries, and circuit breaker
let config = TalosClientConfig::builder("https://10.0.0.1:50000")
    .ca_cert("/path/to/ca.crt")
    .client_cert("/path/to/client.crt")
    .client_key("/path/to/client.key")
    .connect_timeout(Duration::from_secs(5))
    .request_timeout(Duration::from_secs(30))
    .build();

let client = TalosClient::new(config).await?;

// Use high-level APIs
let hostname = client.hostname().await?;
let services = client.service_list().await?;
let kubeconfig = client.kubeconfig().await?;
```

### Prometheus Metrics

```rust
use talos_api_rs::runtime::{MetricsCollector, MetricsConfig};

let metrics = MetricsCollector::new(MetricsConfig::builder()
    .namespace("talos")
    .build());

// Record requests
metrics.record_request("Version", "10.0.0.1:50000", true, Duration::from_millis(42));

// Export Prometheus format
println!("{}", metrics.to_prometheus_text());
```

---

## Documentation

- **[API Documentation](https://docs.rs/talos-api-rs)** — Full Rustdoc on docs.rs
- [API Concept](docs/app-concept.md) — Architecture and design decisions
- [Architecture](docs/architecture.md) — Technical architecture
- [API Stability](docs/api-stability.md) — Stability guarantees
- [Testing](docs/testing.md) — Test strategy and harness
- [Release Checklist](docs/release-checklist.md) — crates.io publication guide
- [Release Preparations](docs/release-preparations.md) — Token and docs.rs setup

---

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

```bash
# Clone and build
git clone https://github.com/XMV-Solutions-GmbH/talos-api-rs.git
cd talos-api-rs
cargo build

# Run tests
cargo test

# Run integration tests (requires Docker and talosctl)
TALOS_DEV_TESTS=1 cargo test --test integration_test
```

---

## Disclaimer

> **This project is NOT affiliated with Sidero Labs or Talos Linux.**
>
> Provided **AS-IS**, without warranty of any kind. Use at your own risk.
>
> Talos® is a registered trademark of Sidero Labs, Inc.

---

## License

Licensed under either of:

- **MIT License** — [LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/license/mit/>
- **Apache License, Version 2.0** — [LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
