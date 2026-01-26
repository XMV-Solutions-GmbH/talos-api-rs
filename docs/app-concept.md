# Application Concept

A typed, async, idiomatic Rust client for the Talos Linux gRPC API.

## Core Design

- **One Client**: A central `TalosClient` manages connection pooling, authentication, and configuration.
- **Typed APIs**: All gRPC methods are exposed via strongly-typed Rust methods, hiding raw Protobuf details where possible.
- **Async-First**: Built on `tokio` and `tonic`.
- **No CLI Wrapper**: Does not call `talosctl` internally; uses pure gRPC.

## Modules

- `client`: Connection and auth logic.
- `api`: Generated gRPC code (via `tonic` + `prost`).
- `resources`: High-level wrappers for Talos resources.
- `testkit`: Integration testing harness using local Talos clusters.
