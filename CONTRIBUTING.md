# Contributing to talos-api-rs

Thank you for your interest in contributing! This document provides guidelines
and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

- Rust 1.75+ (see `rust-version` in Cargo.toml)
- Docker (for integration tests)
- `talosctl` CLI (for integration tests)

### Setup

```bash
# Clone the repository
git clone https://github.com/XMV-Solutions-GmbH/talos-api-rs.git
cd talos-api-rs

# Build
cargo build

# Run tests
cargo test

# Run integration tests (requires Docker)
TALOS_DEV_TESTS=1 cargo test --test integration_test -- --nocapture
```

## Development Guidelines

### Code Style

- **Rust 2021 edition**
- Run `cargo fmt` before committing
- Run `cargo clippy --all-targets --all-features -- -D warnings`
- No `unwrap()` in library code (use `expect()` with context or `?`)
- No `todo!()` or `unimplemented!()` in library code

### SPDX Headers

Every source file must start with:

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
```

### Documentation

- All public items must have doc comments (`///`)
- Include examples in doc comments where helpful
- Examples must compile (`cargo test --doc`)

### Testing

- **Unit tests** are mandatory for all new functionality
- **Integration tests** are required for new API methods
- Run the full test suite before submitting PRs

```bash
# Unit tests
cargo test --lib

# Integration tests
TALOS_DEV_TESTS=1 cargo test --test integration_test -- --nocapture

# Doc tests
cargo test --doc
```

## Pull Request Process

1. **Fork** the repository
2. **Create a feature branch** from `main`
   ```bash
   git checkout -b feature/my-feature
   ```
3. **Make your changes** with clear, atomic commits
4. **Run all checks**
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```
5. **Open a Pull Request** with a clear description
6. **Wait for CI** to pass
7. **Address review feedback** if any

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add EtcdSnapshot API
fix: handle empty response in Kubeconfig
docs: update README with new examples
test: add integration test for Reset API
refactor: simplify error handling in client
ci: add release workflow
```

### Breaking Changes

- Breaking changes require discussion in an issue first
- Update CHANGELOG.md with migration notes
- Bump version appropriately

## Architecture

See [docs/architecture.md](docs/architecture.md) for the codebase structure.

Key directories:
- `src/client/` — Core client and connection management
- `src/resources/` — Typed API wrappers
- `src/runtime/` — Resilience (retry, circuit breaker) and observability
- `src/api/` — Generated protobuf code
- `tests/` — Integration tests

## Need Help?

- Open an [issue](https://github.com/XMV-Solutions-GmbH/talos-api-rs/issues) for bugs or feature requests
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the
MIT OR Apache-2.0 dual license.
