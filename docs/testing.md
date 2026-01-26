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
- `sudo` privileges (required for `talosctl cluster create` to setup networking/CNI)
- `TALOS_DEV_TESTS=1` environment variable set

**Workflow:**

```bash
export TALOS_DEV_TESTS=1
# Usually requires sudo for the docker provisioner to manage networks
sudo -E cargo test -- --ignored
```

(Note: Integration tests should be marked `#[ignore]` by default so `cargo test` runs fast, or filtered by env var logic).

## Writing Tests

- Use `testkit` helpers to provision a cluster.
- Ensure tests tear down the cluster in a `Drop` guard or via `finally` logic.
