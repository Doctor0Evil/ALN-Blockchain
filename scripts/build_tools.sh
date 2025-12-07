#!/usr/bin/env bash
set -euo pipefail

# Build CLI tools
pushd tools/kujira_orphan_scanner
cargo build --release
cp target/release/kujira_orphan_scanner ../../artifacts/kujira_orphan_scanner || true
popd

pushd tools/aln_tools
cargo build --release
cp target/release/aln_tools ../../artifacts/aln_tools || true
popd

pushd tools/ubs_analyzer
cargo build --release
cp target/release/ubs_analyzer ../../artifacts/ubs_analyzer || true
popd

echo "Built tools and copied to artifacts/"