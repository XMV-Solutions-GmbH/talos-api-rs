# Testing Guide

> **MANDATORY**: Integration tests MUST be run before every PR that adds or modifies API functionality.

## Unit Tests

Run standard unit tests:

```bash
cargo test
```

## Integration Tests (Dev Harness) - MANDATORY

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

## PR Checklist (MANDATORY)

Before creating a PR, run:

```bash
# 1. Format code
cargo fmt

# 2. Lint check
cargo clippy --all-targets --all-features -- -D warnings

# 3. Unit tests
cargo test

# 4. Integration tests (MANDATORY for API changes)
TALOS_DEV_TESTS=1 cargo test --test integration_test -- --nocapture
```

**Integration tests are REQUIRED for every feature that adds or modifies API functionality.**

## Current Integration Test Coverage

The integration test (`tests/integration_test.rs`) covers:

| API | Status | Notes |
|-----|--------|-------|
| Version API | ✓ | Returns "Unimplemented" (expected) |
| Hostname | ✓ | Returns node hostname |
| ServiceList | ✓ | Lists all running services |
| SystemStat | ✓ | CPU, memory, boot time |
| ApplyConfiguration | ✓ | Dry-run validation test |
