<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# GitHub Copilot Instructions – Talos Rust Client

You are acting as a senior Rust engineer and open-source maintainer. This repository hosts a Talos Linux API client written in Rust. The goal is to build a professional, production-grade client library, following the same architectural and quality principles as kube-rs, but NOT replacing talosctl.

This document is binding for all Copilot contributions.

## 1. Project Goal & Scope

Goal

Provide a typed, async, idiomatic Rust client for the Talos gRPC API, suitable for:
-	cluster lifecycle tooling
-	operators / controllers
-	automation & integration
-	desktop tools (via Rust backend)

Explicit Non-Goals
-	❌ No CLI replacement for talosctl
-	❌ No opinionated workflows (bootstrap, upgrade orchestration)
-	❌ No YAML/UX abstractions

The library is a thin, correct, observable API client.

## 2. High-Level Architecture (kube-rs inspired)

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
-	Async-first (tokio)
-	Typed APIs, no stringly-typed calls
-	Explicit error handling
-	Minimal magic
-	Stable public API

## 3. Programming Approach

gRPC & Protobuf
-	Use tonic + prost
-	Protobufs are sourced from official Talos repositories only
-	Generated code lives in api::generated
-	NEVER manually edit generated files

Client Design

let client = TalosClient::new(config).await?;

let machines = client.machines().list().await?;

Rules:
-	One shared TalosClient
-	API groups exposed as sub-clients
-	Explicit auth & endpoint config

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

## 4. Supported API Surface (Phased)

Phase 1 – Critical / Core
-	Connection & authentication
-	Version / health APIs
-	Machine list / status
-	Node reboot / shutdown

Phase 2 – Common Operations
-	Machine config get/set
-	Upgrade status
-	Logs & diagnostics
-	Cluster info

Phase 3 – Full Coverage
-	Bootstrap APIs
-	Certificate APIs
-	Maintenance mode
-	Advanced diagnostics

All phases must be tracked in /docs/todo.md.

## 5. Testing Strategy (MANDATORY)

Unit Tests
-	Every public function
-	Mock gRPC services where possible
-	Deterministic, fast

Dev / Integration Tests (MANDATORY PER FEATURE)

Copilot MUST implement a local Talos test harness:

Dev Test Workflow
	1.	Create local Talos cluster

talosctl cluster create --name talos-dev


	2.	Run client operations against it
	3.	Validate results
	4.	Destroy cluster

talosctl cluster destroy --name talos-dev

Rules:
-	Integration tests live under tests/integration_test.rs
-	Tests must auto-skip if talosctl is missing
-	Use env-guard (TALOS_DEV_TESTS=1)
-	**Every new API feature MUST have an integration test**
-	**Integration tests MUST be run before every PR**

Run Integration Tests:
```bash
TALOS_DEV_TESTS=1 cargo test --test integration_test -- --nocapture
```

For Unsupported Features
-	Use talosctl via subprocess ONLY in tests
-	Never in library runtime code

## 6. Code Hygiene Rules
-	Rust 2021 edition
-	#![deny(warnings)]
-	clippy::pedantic enabled
-	No unwrap() in library code
-	No todo!() or unimplemented!()
-	Public API changes require changelog entry
  • Every source file MUST start with an SPDX header:
    `// SPDX-License-Identifier: MIT OR Apache-2.0`
  • This rule applies to ALL `.rs` files, including generated code
  • Generated code MUST additionally include a `DO NOT EDIT` notice

## 7. Documentation Requirements

Mandatory Docs
-	README.md
-	docs/app-concept.md
-	docs/todo.md
-	docs/architecture.md
-	docs/testing.md

Rust Docs
-	All public structs & functions documented
-	Examples compile
-	cargo doc --no-deps must succeed
  • SPDX headers MUST also be present in non-Rust files where applicable (Markdown, YAML, TOML)

Markdown Formatting Rules (MANDATORY)

All Markdown files MUST follow these formatting rules:

1. **Blank line after headings** — Always add one blank line after `#`, `##`, `###`, etc.
2. **Table formatting** — Use spaces around pipe characters: `| text |` not `|text|`
3. **List indentation** — Use consistent indentation (2 or 4 spaces)
4. **Trailing newline** — Every file ends with exactly one newline
5. **No trailing whitespace** — Remove trailing spaces from lines
6. **Blank lines around code blocks** — Add blank line before and after fenced code blocks

Example table formatting:
```markdown
| Column 1 | Column 2 |
| -------- | -------- |
| value    | value    |
```

NOT:
```markdown
|Column 1|Column 2|
|--------|--------|
|value|value|
```

## 8. Open Source Governance

License
-	Dual license: MIT OR Apache-2.0
-	Include LICENSE-MIT and LICENSE-APACHE

Required Files
-	CODE_OF_CONDUCT.md
-	CONTRIBUTING.md
-	SECURITY.md

Risk Mitigation Texts

README must include:

This project is NOT affiliated with Sidero Labs or Talos Linux.
Provided AS-IS without warranty.


## 9. Release & Maintenance Process

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

## 10. Contribution Workflow

Copilot must maintain:
-	Conventional Commits
-	Semantic Versioning
-	CI with:
-	cargo fmt
-	cargo clippy
-	cargo test

**Before creating a PR (`/create-pr`) - MANDATORY:**
1. Run `cargo fmt` to format all code
2. Run `cargo clippy --all-targets --all-features -- -D warnings`
3. Run `cargo test` (unit tests)
4. **Run `TALOS_DEV_TESTS=1 cargo test --test integration_test -- --nocapture` (integration tests)**
5. Commit any formatting changes
6. Then create the PR

**Integration Tests are MANDATORY for every feature that adds or modifies API functionality.**
Skipping integration tests is NOT allowed unless explicitly approved by the user.

## 11. Changelog Management

Copilot MUST maintain a `CHANGELOG.md` file with every release.
Rules:
-	Follow Keep a Changelog format
-	Use Semantic Versioning
-	Must include date of each release
-	Must categorize changes under Added, Changed, Fixed, Removed, Deprecated, Security
-	Must note breaking changes explicitly
-	Must include migration guides for breaking changes
- Link to live documantation <https://docs.rs/talos-api-rs/X.X.X/> (e.g. version 0.1.0: <https://docs.rs/talos-api-rs/0.1.0/>)

- mention oldest version of talos gRPC API supported in each release (update Readme.md as well)
- update todo and open points in documentation (/docs/...) before each release


## 12. Final Rule

If something is fundamentally unclear, you are allowed to stop and ask. But try to be self-sufficient first.

Never guess Talos semantics.

This repository must look, feel, and behave like a serious open-source project from day one.
SPDX compliance is mandatory and non-negotiable.