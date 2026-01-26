# talos-api-rs

[![Crates.io](https://img.shields.io/crates/v/talos-api-rs.svg)](https://crates.io/crates/talos-api-rs)
[![Documentation](https://docs.rs/talos-api-rs/badge.svg)](https://docs.rs/talos-api-rs)
[![CI](https://github.com/XMV-Solutions-GmbH/talos-api-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/XMV-Solutions-GmbH/talos-api-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-experimental-orange.svg)](#status)

Idiomatic, async Rust client for the Talos Linux gRPC API.

This crate provides a **typed, production-grade API client** for interacting with
[Talos Linux](https://www.talos.dev/) nodes and clusters from Rust, inspired by the
design principles of [`kube-rs`](https://github.com/kube-rs/kube).

---

## Goals

- **Async-first** — Built on `tokio` and `tonic`
- **Strongly typed** — No stringly-typed API calls
- **Minimal abstraction** — Thin wrapper over gRPC, no magic
- **Observable** — Tracing and logging built-in
- **Production-ready** — Error handling, retries, timeouts (planned)

## Non-Goals

- ❌ Not a replacement for `talosctl`
- ❌ No opinionated workflows (bootstrap orchestration, etc.)
- ❌ No CLI/UI layer

---

## Status

> ⚠️ **Experimental** — API surface and internals may change without notice.

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ Complete | Core client, TLS, Version & basic Machine APIs |
| Phase 2 | 🔄 In Progress | Cluster lifecycle (Bootstrap, Reset, Kubeconfig) |
| Phase 3 | 📋 Planned | Extended APIs (etcd, logs, events, streaming) |
| Phase 4 | 📋 Planned | Production readiness, crates.io release |

### Current Capabilities

- ✅ TLS and insecure mode connections
- ✅ Version API (health checks)
- ✅ Machine API: Hostname, ServiceList, SystemStat, Reboot, Shutdown
- ⚠️ mTLS with ED25519 certificates (Talos default) — **in progress**

### Known Limitations

- ED25519 client certificates require additional rustls configuration
- Streaming APIs (Logs, Events, Kubeconfig) not yet implemented
- Multi-node targeting not yet supported

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

---

## Documentation

- [API Concept](docs/app-concept.md) — Architecture and design decisions
- [TODO & Roadmap](docs/todo.md) — Development progress and plans
- [Architecture](docs/architecture.md) — Technical architecture
- [Testing](docs/testing.md) — Test strategy and harness
- [Release Checklist](docs/release-checklist.md) — crates.io publication guide

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
