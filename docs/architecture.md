# Architecture

## Directory Structure

```text
crate
├── client/          # TalosClient, connection, auth, TLS
├── api/             # Generated gRPC APIs (prost/tonic)
├── resources/       # Strongly typed domain wrappers
├── runtime/         # Retry, backoff, timeouts, interceptors
├── error/           # Structured error types
├── testkit/         # Dev/test helpers (local Talos cluster)
├── examples/        # Minimal real-world usage examples
└── docs/
```

## Layers

1. **Transport Layer (`client`)**: Handles TLS connecting, mTLS certificates, and channel management.
2. **API Layer (`api`)**: Pure generated code. DO NOT EDIT manually.
3. **Resource Layer (`resources`)**: Provides a more "Rust-native" feel over the raw Protobuf structs.
4. **Runtime Layer (`runtime`)**: Cross-cutting concerns like retries.

## Testing Strategy

- **Unit Tests**: Mock logic in `client` and `resources`.
- **Integration Tests**: Use `testkit` to spin up ephemeral Docker-based Talos clusters (via `talosctl`) and run real commands against them.
