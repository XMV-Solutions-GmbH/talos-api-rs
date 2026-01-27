<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# TODO

> **Current**: v0.2.0 | **Target**: v1.0.0 Stable

## Roadmap to v1.0.0

### Missing APIs

| API | Type | Blocker | Notes |
| --- | ---- | ------- | ----- |
| EtcdRecover | Client-streaming | v1.0 | Requires client-streaming support |
| EtcdSnapshot | Server-streaming | v1.0 | Backup/restore workflows |
| Events | Server-streaming | v1.0 | Cluster event monitoring |

### Streaming Improvements

- [ ] True async iterators for streaming APIs (Kubeconfig, Dmesg, Logs, Files)
- [ ] Backpressure handling for large streams
- [ ] Streaming progress callbacks

### Multi-Node Operations

- [x] gRPC metadata for node targeting (`x-talos-node`)
- [x] Cluster-wide operations (`with_nodes()`)
- [ ] Parallel execution with result aggregation

### Quality of Life

- [ ] More examples (cluster upgrade workflow)
- [ ] Tutorial: Building a Talos operator
- [ ] API coverage matrix vs talosctl

### Release Blockers

- [ ] crates.io publication (requires `CRATES_IO_TOKEN` secret)

---

## Known Issues

### Medium Priority

| Issue | Description |
| ----- | ----------- |
| Streaming collection | APIs collect full stream into memory, no true async iteration |
| Client-streaming | EtcdRecover requires unimplemented client-streaming |

### Low Priority

| Issue | Description |
| ----- | ----------- |
| Error granularity | Could parse `google.rpc.Status` details for richer errors |
| Generated code size | `machine.rs` is ~6000 lines (acceptable, just large) |

---

## Future (v1.1.0+)

- [ ] Talos 1.10 API additions (when released)
- [ ] SideroLink integration
- [ ] Machine config validation (schema-based)
- [ ] Async trait stabilization (when Rust stabilizes)

---

## Completed (v0.2.0)

Summary of what's done - see [CHANGELOG.md](../CHANGELOG.md) for details.

- [x] **43/52 Machine Service APIs** (83% coverage)
- [x] **ED25519 mTLS** with ring crypto provider
- [x] **Connection pooling** with load balancing strategies
- [x] **Retry policies** (exponential, linear, fixed backoff)
- [x] **Circuit breaker** pattern
- [x] **Prometheus metrics** export
- [x] **OpenTelemetry tracing** integration
- [x] **talosconfig parsing** (`~/.talos/config`)
- [x] **Cluster discovery** helpers
- [x] **Full documentation** on docs.rs
