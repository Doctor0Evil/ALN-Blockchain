#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"

cargo update
cargo build --workspace
cargo test --workspace

if command -v kani >/dev/null 2>&1; then
  cargo kani -p bioscale-upgrade-store || true
fi

cargo run -p bioscale-evolution-cli -- --emit "target/manifests"
