<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# API Stability

This document describes the API stability guarantees for `talos-api-rs`.

## Versioning Policy

We follow [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** version (X.0.0): Incompatible API changes
- **MINOR** version (0.X.0): Backwards-compatible functionality additions
- **PATCH** version (0.0.X): Backwards-compatible bug fixes

## Pre-1.0 Policy

During the 0.x development phase:

- **MINOR** version bumps may include breaking changes
- Breaking changes will be documented in the CHANGELOG
- We aim to minimize disruption but cannot guarantee full backwards compatibility
- Users should pin to exact versions or minor versions (`talos-api-rs = "0.1"`)

## Stability Tiers

### Stable API (Tier 1)

These APIs are considered stable and will not change without a MAJOR version bump after 1.0:

| Module | Items |
| ------ | ----- |
| `client` | `TalosClient`, `TalosClientConfig`, `TalosClientConfigBuilder` |
| `error` | `TalosError` |
| `runtime` | `RetryConfig`, `CircuitBreakerConfig`, `LoggingConfig` |

### Semi-Stable API (Tier 2)

These APIs are relatively stable but may see non-breaking changes:

| Module | Items |
| ------ | ----- |
| `resources::*` | All typed resource wrappers |
| `runtime` | `MetricsCollector`, `TracingConfig`, `SpanFactory` |
| `client::pool` | `ConnectionPool`, `ConnectionPoolConfig` |

### Experimental API (Tier 3)

These APIs may change significantly:

| Module | Items |
| ------ | ----- |
| `api::generated` | All generated protobuf types |
| Internal modules | Any module not re-exported from `lib.rs` |

## What Constitutes a Breaking Change

The following are considered breaking changes:

1. **Removing public items** (functions, types, traits, modules)
2. **Changing function signatures** (parameters, return types)
3. **Changing struct fields** (for non-`#[non_exhaustive]` structs)
4. **Changing enum variants** (for non-`#[non_exhaustive]` enums)
5. **Adding required trait methods** (without default implementations)
6. **Changing error types** returned by functions
7. **Changing MSRV** (Minimum Supported Rust Version)

## What Is NOT a Breaking Change

The following are not considered breaking changes:

1. Adding new public items
2. Adding new optional parameters (via builders)
3. Adding new enum variants to `#[non_exhaustive]` enums
4. Adding new struct fields to `#[non_exhaustive]` structs
5. Adding new trait methods with default implementations
6. Bug fixes that change incorrect behavior
7. Performance improvements
8. Documentation changes
9. Internal refactoring that doesn't affect public API
10. Dependency updates (unless they change public API)

## Generated Code Stability

The `api::generated` module contains code generated from Talos protobuf definitions:

- This module follows Talos upstream versioning
- Changes to Talos protobufs may result in breaking changes
- We track Talos versions in release notes
- Users needing stability should use the typed wrappers in `resources::*`

## MSRV Policy

- Current MSRV: **Rust 1.82**
- MSRV changes are documented in CHANGELOG
- We test against the current MSRV in CI
- MSRV will only be increased in MINOR versions (0.x.0) or MAJOR versions

## Deprecation Policy

1. Deprecated items will be marked with `#[deprecated]`
2. Deprecation warnings will include migration instructions
3. Deprecated items will be removed in the next MAJOR version
4. We aim to provide at least 6 months warning before removal

## Feature Flags

Currently, all features are enabled by default. Future versions may introduce:

- `tracing` - OpenTelemetry tracing support
- `metrics` - Prometheus metrics support

Feature flag changes will be documented and are not considered breaking changes.

## Commitment

After 1.0 release, we commit to:

1. Following semantic versioning strictly
2. Documenting all breaking changes in CHANGELOG
3. Providing migration guides for major version upgrades
4. Maintaining backwards compatibility within major versions
5. Responding to stability-related issues within 7 days

## Questions

If you have questions about API stability, please:

1. Open an issue on GitHub
2. Tag it with `api-stability`
3. Describe your use case
