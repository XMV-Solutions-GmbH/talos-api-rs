#!/bin/bash
# SPDX-License-Identifier: MIT OR Apache-2.0

# Copilot Harness Script
# Usage: ./copilot-harness.sh

set -euo pipefail

echo "=== [Copilot Harness] Starting Cleanup ==="

# --- Cleanup Logic (Aggressive) ---
TALOS_DIR="${HOME}/.talos"
CLUSTERS_DIR="${TALOS_DIR}/clusters"

ok()   { printf '[OK] %s\n' "$*"; }
warn() { printf '[WARN] %s\n' "$*" >&2; }
err()  { printf '[ERR] %s\n' "$*" >&2; }

# Fix permissions first if needed
if [[ -d "$CLUSTERS_DIR" && ! -w "$CLUSTERS_DIR" ]]; then
    warn "Permissions issue detected on $CLUSTERS_DIR. Attempting fix..."
    sudo chown -R "$USER" "$CLUSTERS_DIR" || warn "Failed to fix permissions"
fi

# 1) Destroy Talos clusters via talosctl
if [[ -d "$CLUSTERS_DIR" ]]; then
  for path in "$CLUSTERS_DIR"/*; do
    [[ -d "$path" ]] || continue
    name="$(basename "$path")"
    
    # Try to destroy known names or just all
    if talosctl cluster destroy --name "$name" --force >/dev/null 2>&1; then
      ok "Destroyed Talos cluster: $name"
    else
      # If talosctl fails, we rely on manual cleanup
      warn "talosctl failed to destroy '$name' (will attempt manual cleanup)"
    fi
  done
else
  ok "No clusters directory found at $CLUSTERS_DIR"
fi

# 2) Docker cleanup (Aggressive)
# Remove any container or network with "talos" in the name
echo ">>> Cleaning Docker resources..."
containers="$(docker ps -aq --filter "name=talos" 2>/dev/null || true)"
if [[ -n "${containers//[[:space:]]/}" ]]; then
  docker rm -f $containers >/dev/null 2>&1 && ok "Removed Docker containers (name=talos)" || warn "Failed to remove some Docker containers"
fi

networks="$(docker network ls -q --filter "name=talos" 2>/dev/null || true)"
if [[ -n "${networks//[[:space:]]/}" ]]; then
  docker network rm $networks >/dev/null 2>&1 && ok "Removed Docker networks (name=talos)" || warn "Failed to remove some Docker networks"
fi

# 3) Hard state cleanup
if [[ -d "$CLUSTERS_DIR" ]]; then
    TEST_CLUSTER="talos-dev-integration" 
    if [[ -d "$CLUSTERS_DIR/$TEST_CLUSTER" ]]; then
        rm -rf "$CLUSTERS_DIR/$TEST_CLUSTER" || sudo rm -rf "$CLUSTERS_DIR/$TEST_CLUSTER"
        ok "Removed cluster state dir for $TEST_CLUSTER"
    fi
fi

echo "=== [Copilot Harness] Cleanup Finished. Starting Tests... ==="

# --- Test Execution ---

# 2. Environment Setup
export TALOS_DEV_TESTS=1
export RUST_BACKTRACE=1

# 3. Unit Tests
echo ">>> Running Unit Tests..."
cargo test --lib --quiet

# 4. Integration Tests
echo ">>> Running Integration Tests..."

if [ "$EUID" -ne 0 ]; then
    echo ">>> Running as non-root user (preferred for Docker provider)..."
    cargo test --test integration_test -- --nocapture
else
    # We are root
    echo ">>> Running as root..."
    cargo test --test integration_test -- --nocapture
fi

echo "=== [Copilot Harness] Success ==="
