# Testing Guide

## Unit Tests

Run standard unit tests:

```bash
cargo test
```

## Integration Tests (Dev Harness)

We use `talosctl` to manage local Docker-based clusters for testing.

**Prerequisites:**

- Docker running
- `talosctl` installed and in PATH
- `yq` installed (for config parsing)

**Run Integration Tests:**

```bash
# Full integration test with real Talos cluster
TALOS_DEV_TESTS=1 cargo test --test integration_test -- --nocapture
```

This will:

1. Create a Docker-based Talos cluster (`talos-dev-integration`)
2. Extract mTLS certificates from talosconfig
3. Test API calls (Hostname, ServiceList, SystemStat)
4. Show cluster status via `talosctl`
5. Destroy the cluster on completion

**Expected Output:**

```
========================================
  Talos Integration Test Suite
========================================
Cluster provisioned at https://127.0.0.1:XXXXX

Using mTLS with certs from:
  CA:  /tmp/.../ca.crt
  CRT: /tmp/.../client.crt
  KEY: /tmp/.../client.key

--- Machine API: Hostname ---
✓ Node: unknown -> hostname: talos-dev-integration-controlplane-1

--- Machine API: ServiceList ---
✓ Node: unknown
  Services:
    ✓ apid [Running]
    ✓ containerd [Running]
    ...

--- Machine API: SystemStat ---
✓ Node: unknown
  Boot time:         ...
  Processes running: ...

========================================
  Integration Tests Complete
========================================
```

**Test Duration:** ~3 minutes (includes cluster provisioning)

**Skip Integration Tests:**

Without `TALOS_DEV_TESTS=1`, integration tests are skipped:

```bash
cargo test  # Only runs unit tests
```

## Writing Tests

- Use `testkit` helpers to provision a cluster.
- Ensure tests tear down the cluster in a `Drop` guard or via `finally` logic.
- Integration tests check for `TALOS_DEV_TESTS` env var before running.
