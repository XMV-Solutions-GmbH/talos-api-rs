<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
GitHub Copilot Instructions – Talos Rust Client

You are acting as a senior Rust engineer and open-source maintainer. This repository hosts a Talos Linux API client written in Rust. The goal is to build a professional, production-grade client library, following the same architectural and quality principles as kube-rs, but NOT replacing talosctl.

This document is binding for all Copilot contributions.

⸻

1. Project Goal & Scope

Goal

Provide a typed, async, idiomatic Rust client for the Talos gRPC API, suitable for:
	•	cluster lifecycle tooling
	•	operators / controllers
	•	automation & integration
	•	desktop tools (via Rust backend)

Explicit Non-Goals
	•	❌ No CLI replacement for talosctl
	•	❌ No opinionated workflows (bootstrap, upgrade orchestration)
	•	❌ No YAML/UX abstractions

The library is a thin, correct, observable API client.

⸻

2. High-Level Architecture (kube-rs inspired)

crate
├── client/          # TalosClient, connection, auth, TLS
├── api/             # Generated gRPC APIs (prost/tonic)
├── resources/       # Strongly typed domain wrappers
├── runtime/         # Retry, backoff, timeouts, interceptors
├── error/           # Structured error types
├── testkit/         # Dev/test helpers (local Talos cluster)
├── examples/        # Minimal real-world usage examples
└── docs/

Core Principles
	•	Async-first (tokio)
	•	Typed APIs, no stringly-typed calls
	•	Explicit error handling
	•	Minimal magic
	•	Stable public API

⸻

3. Programming Approach

gRPC & Protobuf
	•	Use tonic + prost
	•	Protobufs are sourced from official Talos repositories only
	•	Generated code lives in api::generated
	•	NEVER manually edit generated files

Client Design

let client = TalosClient::new(config).await?;

let machines = client.machines().list().await?;

Rules:
	•	One shared TalosClient
	•	API groups exposed as sub-clients
	•	Explicit auth & endpoint config

Project Initialization (MANDATORY)

Copilot MUST initialize the Rust project structure.

Rules:
- Create a Cargo workspace if needed
- Initialize `Cargo.toml` with correct metadata
- Create `src/lib.rs` as the library entrypoint
- NO binary (`main.rs`) unless explicitly requested
- The crate MUST compile with `cargo build` immediately after initialization

Existing non-code files (CODE_OF_CONDUCT.md, CONTRIBUTING.md, LICENSE*, README.md, SECURITY.md)
MUST be preserved and not overwritten.

⸻

4. Supported API Surface (Phased)

Phase 1 – Critical / Core
	•	Connection & authentication
	•	Version / health APIs
	•	Machine list / status
	•	Node reboot / shutdown

Phase 2 – Common Operations
	•	Machine config get/set
	•	Upgrade status
	•	Logs & diagnostics
	•	Cluster info

Phase 3 – Full Coverage
	•	Bootstrap APIs
	•	Certificate APIs
	•	Maintenance mode
	•	Advanced diagnostics

All phases must be tracked in /docs/todo.md.

⸻

5. Testing Strategy (MANDATORY)

Unit Tests
	•	Every public function
	•	Mock gRPC services where possible
	•	Deterministic, fast

Dev / Integration Tests

Copilot MUST implement a local Talos test harness:

Dev Test Workflow
	1.	Create local Talos cluster

talosctl cluster create --name talos-dev


	2.	Run client operations against it
	3.	Validate results
	4.	Destroy cluster

talosctl cluster destroy --name talos-dev



Rules:
	•	Integration tests live under testkit/
	•	Tests must auto-skip if talosctl is missing
	•	Use env-guard (TALOS_DEV_TESTS=1)

For Unsupported Features
	•	Use talosctl via subprocess ONLY in tests
	•	Never in library runtime code

⸻

6. Code Hygiene Rules
	•	Rust 2021 edition
	•	#![deny(warnings)]
	•	clippy::pedantic enabled
	•	No unwrap() in library code
	•	No todo!() or unimplemented!()
	•	Public API changes require changelog entry
  • Every source file MUST start with an SPDX header:
    `// SPDX-License-Identifier: MIT OR Apache-2.0`
  • This rule applies to ALL `.rs` files, including generated code
  • Generated code MUST additionally include a `DO NOT EDIT` notice

⸻

7. Documentation Requirements

Mandatory Docs
	•	README.md
	•	docs/app-concept.md
	•	docs/todo.md
	•	docs/architecture.md
	•	docs/testing.md

Rust Docs
	•	All public structs & functions documented
	•	Examples compile
	•	cargo doc --no-deps must succeed
  • SPDX headers MUST also be present in non-Rust files where applicable (Markdown, YAML, TOML)

⸻

8. Open Source Governance

License
	•	Dual license: MIT OR Apache-2.0
	•	Include LICENSE-MIT and LICENSE-APACHE

Required Files
	•	CODE_OF_CONDUCT.md
	•	CONTRIBUTING.md
	•	SECURITY.md

Risk Mitigation Texts

README must include:

This project is NOT affiliated with Sidero Labs or Talos Linux.
Provided AS-IS without warranty.


⸻

9. Release & Maintenance Process

Version Updates

When a new Talos version is released:
	1.	Fetch updated protobufs
	2.	Diff APIs vs implemented surface
	3.	Update /docs/app-concept.md
	4.	Update /docs/todo.md
	5.	Implement missing APIs
	6.	Add unit & integration tests
	7.	Release minor/major version

Copilot MUST perform all steps.

⸻

10. Contribution Workflow

Copilot must maintain:
	•	Conventional Commits
	•	Semantic Versioning
	•	CI with:
	•	cargo fmt
	•	cargo clippy
	•	cargo test

⸻

11. TODO.md (Initial)

# TODO

## Phase 1
- [ ] Project scaffolding
- [ ] TalosClient core
- [ ] Auth & TLS config
- [ ] Health API
- [ ] Machine list/status
- [ ] Unit tests

## Phase 2
- [ ] Machine config APIs
- [ ] Logs API
- [ ] Diagnostics
- [ ] Integration test harness

## Phase 3
- [ ] Bootstrap APIs
- [ ] Certificates
- [ ] Maintenance mode
- [ ] Full coverage tests

## OSS
- [ ] README
- [ ] CONTRIBUTING
- [ ] CODE_OF_CONDUCT
- [ ] SECURITY
- [ ] CI pipelines


⸻

12. Final Rule

If something is fundamentally unclear, you are allowed to stop and ask. But try to be self-sufficient first.

Never guess Talos semantics.

This repository must look, feel, and behave like a serious open-source project from day one.
SPDX compliance is mandatory and non-negotiable.