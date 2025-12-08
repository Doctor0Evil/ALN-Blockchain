#!/usr/bin/env bash
set -eo pipefail

echo "Running Energy Market Smoke Test"
RUST_BACKTRACE=1
# Build integration tests + contracts
cargo build --workspace --locked

cd tests/integration
echo "Running integration tests to exercise bridge & UBS flows"
cargo test -- --nocapture
echo "Smoke test completed"
