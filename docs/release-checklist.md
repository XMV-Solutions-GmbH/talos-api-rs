<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Release Checklist

Comprehensive checklist for publishing `talos-api-rs` to crates.io.

---

## Pre-Release Requirements

### Cargo.toml Metadata (Required by crates.io)

- [x] `name` - Package name
- [x] `version` - Semantic version
- [x] `edition` - Rust edition (2021)
- [x] `description` - Short description
- [x] `license` - SPDX expression (`MIT OR Apache-2.0`)
- [x] `repository` - GitHub URL
- [x] `authors` - Contributor list
- [x] `keywords` - Max 5 keywords for search
- [x] `categories` - crates.io categories
- [ ] `readme` - Path to README.md
- [ ] `documentation` - Link to docs.rs
- [ ] `homepage` - Project homepage (optional)
- [ ] `rust-version` - MSRV (Minimum Supported Rust Version)
- [ ] `exclude` - Files to exclude from package

### Documentation (docs.rs)

- [ ] All public items have `///` doc comments
- [ ] Module-level docs with `//!`
- [ ] Examples in doc comments (tested via `cargo test --doc`)
- [ ] `#![deny(missing_docs)]` enabled in lib.rs
- [ ] Feature flags documented
- [ ] Error types documented with recovery hints
- [ ] README.md included and renders correctly

### Code Quality

- [ ] `cargo fmt` - No formatting issues
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - No warnings
- [ ] `cargo test` - All tests pass
- [ ] `cargo doc --no-deps` - Documentation builds
- [ ] `cargo publish --dry-run` - Package validates
- [ ] No `todo!()` or `unimplemented!()` in library code
- [ ] No `unwrap()` in library code (use `expect()` with context or `?`)
- [ ] SPDX headers in all source files

### API Stability

- [ ] Public API reviewed for stability
- [ ] Breaking changes documented in CHANGELOG
- [ ] Deprecated items marked with `#[deprecated]`
- [ ] Version follows SemVer:
  - `0.x.y` - Initial development, API may change
  - `1.0.0` - Stable API commitment
  - Major bump for breaking changes
  - Minor bump for new features
  - Patch bump for bug fixes

---

## Release Process

### 1. Version Bump

```bash
# Update version in Cargo.toml
# Update CHANGELOG.md with release notes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v0.2.0"
```

### 2. Final Checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo doc --no-deps
cargo publish --dry-run
```

### 3. Tag Release

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

### 4. Publish to crates.io

```bash
# First time: cargo login <token>
cargo publish
```

### 5. GitHub Release

```bash
gh release create v0.2.0 --title "v0.2.0" --notes-file CHANGELOG.md
```

---

## crates.io Account Setup

### Prerequisites

1. Create account at <https://crates.io> (GitHub OAuth)
2. Generate API token: <https://crates.io/settings/tokens>
3. Login: `cargo login <token>`

### First Publish

- Package name must be unique
- Cannot be changed after first publish
- Yanking removes from search but doesn't delete

---

## Version Strategy

### Current: `0.1.0` (Experimental)

- API is unstable
- Breaking changes expected
- Use for early adopters only

### Target: `0.2.0` (Alpha)

- Core cluster lifecycle APIs working
- mTLS resolved
- Basic documentation complete

### Future: `1.0.0` (Stable)

- API stability commitment
- Full test coverage
- Production-ready error handling
- Comprehensive documentation

---

## docs.rs Configuration

Add to `Cargo.toml`:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

In `lib.rs`:

```rust
#![cfg_attr(docsrs, feature(doc_cfg))]
```

This enables feature-gated documentation on docs.rs.

---

## Badge Status Levels

| Badge | Meaning | Requirements |
|-------|---------|--------------|
| 🔴 Experimental | API unstable, use at own risk | Basic functionality |
| 🟡 Alpha | Core features work, API may change | Tests, basic docs |
| 🟢 Beta | Feature complete, API stabilizing | Full docs, examples |
| 🔵 Stable | Production ready | 1.0.0 release |

Current status: **🔴 Experimental**

---

## Post-Release Tasks

- [ ] Announce on relevant channels
- [ ] Monitor crates.io download stats
- [ ] Watch for issue reports
- [ ] Update dependent projects
- [ ] Plan next release

---

## Rollback Procedure

If a broken release is published:

```bash
# Yank the broken version (prevents new downloads)
cargo yank --version 0.2.0

# Fix the issue and publish patch
cargo publish  # 0.2.1
```

Note: Yanking does NOT delete - existing users can still use it.

---

## CI/CD Release Pipeline (Future)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

Add `CARGO_REGISTRY_TOKEN` to GitHub Secrets.
