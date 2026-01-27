<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Release Preparations

This guide covers the one-time setup required before publishing `talos-api-rs` to crates.io.

---

## 1. crates.io Account & Token

### Create a crates.io Account

1. Go to [crates.io](https://crates.io)
2. Click "Log in with GitHub"
3. Authorize crates.io to access your GitHub account

### Generate an API Token

1. Log in to [crates.io](https://crates.io)
2. Go to **Account Settings** → [API Tokens](https://crates.io/settings/tokens)
3. Click **"New Token"**
4. Configure the token:
   - **Name**: `talos-api-rs-publish` (or any descriptive name)
   - **Scopes**: Select `publish-new` and `publish-update`
   - **Crates**: Optionally restrict to `talos-api-rs`
5. Click **"Generate Token"**
6. **Copy the token immediately** - it won't be shown again!

### Add Token as GitHub Secret

Using the GitHub CLI (`gh`):

```bash
# Authenticate if not already
gh auth login

# Add the secret to your repository
gh secret set CRATES_IO_TOKEN --repo XMV-Solutions-GmbH/talos-api-rs

# You'll be prompted to enter the token value
# Paste your crates.io token and press Enter
```

Alternatively, via GitHub Web UI:

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **"New repository secret"**
4. Name: `CRATES_IO_TOKEN`
5. Value: Paste your crates.io API token
6. Click **"Add secret"**

### Verify Token Works

Test the token locally before relying on CI:

```bash
# Dry-run publish (won't actually publish)
cargo publish --dry-run --token <your-token>
```

---

## 2. docs.rs Documentation

[docs.rs](https://docs.rs) automatically builds and hosts documentation for all crates published to crates.io.

### How It Works

1. You publish your crate to crates.io
2. docs.rs detects the new version
3. Documentation is built automatically using `cargo doc`
4. Docs are available at `https://docs.rs/talos-api-rs/<version>`

### Configuration

Documentation settings are configured in `Cargo.toml`:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

This tells docs.rs to:

- Build with all features enabled
- Set the `docsrs` cfg flag (for conditional documentation)

### Conditional Documentation

Use the `docsrs` cfg to add docs.rs-specific attributes:

```rust
#![cfg_attr(docsrs, feature(doc_cfg))]

/// This function requires the `tracing` feature.
#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub fn traced_function() { }
```

### Documentation Best Practices

1. **All public items must be documented**:

   ```rust
   /// Short description.
   ///
   /// Longer explanation with details.
   ///
   /// # Examples
   ///
   /// ```rust
   /// let client = TalosClient::new(config).await?;
   /// ```
   pub fn my_function() { }
   ```

2. **Module-level documentation**:

   ```rust
   //! This module provides X functionality.
   //!
   //! # Overview
   //!
   //! ...
   ```

3. **Test your examples** - doc examples are compiled and tested:

   ```bash
   cargo test --doc
   ```

### Preview Documentation Locally

Before publishing, preview your documentation:

```bash
# Build documentation
cargo doc --no-deps --open

# Build with all features (like docs.rs)
cargo doc --no-deps --all-features --open
```

### After Publishing

Once published, documentation will be available at:

- **Latest**: <https://docs.rs/talos-api-rs>
- **Specific version**: <https://docs.rs/talos-api-rs/0.1.0>

Documentation typically appears within 5-15 minutes after publishing.

### Rebuild Documentation

If docs.rs fails to build (rare), you can request a rebuild:

1. Go to <https://docs.rs/crate/talos-api-rs>
2. Click the gear icon (⚙️)
3. Select "Rebuild documentation"

---

## 3. First-Time Publish Checklist

Before your first publish to crates.io:

- [x] crates.io account created
- [x] API token generated with publish scope
- [x] `CRATES_IO_TOKEN` secret added to GitHub
- [x] `cargo publish --dry-run` succeeds locally
- [x] All documentation compiles: `cargo doc --no-deps`
- [x] Doc tests pass: `cargo test --doc`
- [x] Version in `Cargo.toml` is correct
- [x] CHANGELOG.md is up to date
- [x] All required `Cargo.toml` metadata is present:
  - `name`, `version`, `edition`
  - `description`, `license`
  - `repository`, `documentation`
  - `keywords`, `categories`

---

## 4. Publishing Workflow

### Automated (Recommended)

The release workflow triggers on version tags:

```bash
# Ensure main is up to date
git checkout main
git pull

# Update version in Cargo.toml
# Update CHANGELOG.md

# Commit and tag
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v0.1.0"
git tag v0.1.0
git push origin main --tags
```

The GitHub Action will:

1. Validate the release
2. Run tests
3. Publish to crates.io
4. Create a GitHub Release

### Manual (Fallback)

If automated publishing fails:

```bash
# Ensure you're on a clean main branch
git checkout main
git status  # Should be clean

# Publish
cargo publish
```

---

## 5. Troubleshooting

### "crate already exists"

You cannot overwrite an existing version. Bump the version and try again.

### "not logged in" / "unauthorized"

- Verify token has correct scopes
- Regenerate token if expired
- Check `CRATES_IO_TOKEN` secret is set correctly

### docs.rs build failed

1. Check the [docs.rs build queue](https://docs.rs/releases/queue)
2. View build logs on the crate page
3. Common issues:
   - Missing system dependencies
   - Feature flag issues
   - Compilation errors

### "package is too big"

Check your `exclude` patterns in `Cargo.toml`:

```toml
exclude = [
    ".github/",
    "tests/",
    "docs/",
    "*.md",
    "!Readme.md",
]
```

Run `cargo package --list` to see what's included.
