# Testing Guide

## Unit Tests

Run standard unit tests:

```bash
cargo test
```

## Integration Tests (Dev Harness)

We use `talosctl` to manage local Docker-based clusters for testing.

**Prerequisites:**
- Docker
- `talosctl` installed and in PATH
- `TALOS_DEV_TESTS=1` environment variable set

**Workflow:**

```bash
export TALOS_DEV_TESTS=1
cargo test -- --ignored
```

(Note: Integration tests should be marked `#[ignore]` by default so `cargo test` runs fast, or filtered by env var logic).

## Writing Tests

- Use `testkit` helpers to provision a cluster.
- Ensure tests tear down the cluster in a `Drop` guard or via `finally` logic.
