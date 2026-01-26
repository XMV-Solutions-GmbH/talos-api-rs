# talos-api-rs

Idiomatic, async Rust client for the Talos Linux gRPC API.

This crate provides a **typed, production-grade API client** for interacting with
[Talos Linux](https://www.talos.dev/) nodes and clusters from Rust, inspired by the
design principles of `kube-rs`.

## Goals

- Async-first Rust API (`tokio`)
- Strongly typed gRPC client
- Minimal abstraction, no magic
- Suitable for automation, operators and tooling

## Non-Goals

- ❌ Not a replacement for `talosctl`
- ❌ No opinionated workflows
- ❌ No UI / CLI layer

## Status

> ⚠️ **Experimental** — API surface and internals may change.

## Disclaimer

This project is **not affiliated with Sidero Labs or Talos Linux**.  
Provided **AS-IS**, without warranty of any kind.

## License

Licensed under either of

- MIT License  
  <https://opensource.org/license/mit/>

- Apache License, Version 2.0  
  <https://www.apache.org/licenses/LICENSE-2.0.txt>

at your option.
