<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Testing Guide

> **Updated**: 2026-01-27 (v0.2.0)
>
> **MANDATORY**: Integration tests MUST be run before every PR that adds or modifies API functionality.

## Test Coverage Summary

| Category | Unit Tests | Integration Tests |
| -------- | ---------- | ----------------- |
| Client core | 14 tests | Connection, mTLS |
| Node targeting | 10 tests | Multi-node operations |
| Connection pool | 12 tests | Health-based routing |
| Cluster discovery | 11 tests | Member discovery, health |
| Configuration | 15 tests | TalosConfig parsing |
| Resources (all) | 100+ tests | API wrappers |
| Runtime | 50+ tests | Retry, circuit breaker, metrics, tracing |
| **Total** | **200 tests** | **20 test cases** |

## Unit Tests

Run standard unit tests:

```bash
cargo test --lib
```

Expected output:
```
test result: ok. 200 passed; 0 failed; 0 ignored
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

```text
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

| # | API | Status | Notes |
| - | --- | ------ | ----- |
| 1 | Version API | ✅ | Returns version info |
| 2 | Hostname | ✅ | Returns node hostname |
| 3 | ServiceList | ✅ | Lists all running services |
| 4 | SystemStat | ✅ | CPU, memory, boot time |
| 5 | ApplyConfiguration | ✅ | Dry-run validation test |
| 6 | Bootstrap | ✅ | Rejection on already-bootstrapped |
| 7 | Kubeconfig | ✅ | Server-streaming, validates structure |
| 8 | Reset | ✅ | API verification (destructive skipped) |
| 9 | EtcdMemberList | ✅ | Lists etcd members |
| 10 | EtcdStatus | ✅ | Etcd cluster status |
| 11 | Dmesg | ✅ | Kernel message streaming |
| 12 | System APIs | ✅ | Memory, CPU, LoadAvg, Disks, Mounts, Network, Processes |
| 13 | File List | ✅ | Directory listing streaming |
| 14 | Logs | ✅ | Service log streaming |
| 15 | Upgrade | ✅ | Dry-run mode |
| 16 | Advanced APIs | ✅ | Rollback, GenerateClientConfig, Netstat, PacketCapture |
| 17 | Node Targeting | ✅ | `with_node()` / `with_nodes()` |
| 18 | TalosConfig | ✅ | Config file parsing, env vars |
| 19 | ImageList | ✅ | Container image listing (v0.2.0) |
| 20 | ClusterDiscovery | ✅ | Member discovery, health check (v0.2.0) |
