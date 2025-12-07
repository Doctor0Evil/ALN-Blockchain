#!/usr/bin/env bash
set -euo pipefail

echo "Running Node tests and linters..."
npm ci
npm run test --workspace=aln/tests
npm run test --workspace=aln/core

echo "Running all Rust tests (workspace)..."
cargo test --all --workspace --verbose

echo "Running cw-multi-test integration tests..."
pushd tests/integration >/dev/null
cargo test --manifest-path Cargo.toml --verbose
popd >/dev/null

echo "All tests finished."
