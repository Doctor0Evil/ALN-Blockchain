#!/usr/bin/env bash
set -euo pipefail

# Build all CosmWasm contracts to artifacts/ using cargo wasm
# Requires: rust toolchain, wasm32-unknown-unknown target, cosmwasm optimize toolchain optional

mkdir -p artifacts

# Example for all contracts; update package names as needed
RUSTFLAGS="-C link-arg=-s"

pushd contracts/auet
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/aln_auet.wasm ../../artifacts/aln_auet.wasm || true
wasm-opt -Oz -o ../../artifacts/aln_auet.optimized.wasm ../../artifacts/aln_auet.wasm || true
popd

pushd contracts/aln20_csp
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/aln_csp.wasm ../../artifacts/aln_csp.wasm || true
wasm-opt -Oz -o ../../artifacts/aln_csp.optimized.wasm ../../artifacts/aln_csp.wasm || true
popd

pushd contracts/bridge
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/aln_bridge.wasm ../../artifacts/aln_bridge.wasm || true
wasm-opt -Oz -o ../../artifacts/aln_bridge.optimized.wasm ../../artifacts/aln_bridge.wasm || true
popd
 
pushd contracts/aln_registry
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/aln_registry.wasm ../../artifacts/aln_registry.wasm || true
wasm-opt -Oz -o ../../artifacts/aln_registry.optimized.wasm ../../artifacts/aln_registry.wasm || true
popd

pushd contracts/energy_router
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/energy_router.wasm ../../artifacts/energy_router.wasm || true
wasm-opt -Oz -o ../../artifacts/energy_router.optimized.wasm ../../artifacts/energy_router.wasm || true
popd

# Add more contracts here

echo "Built and optimized sample contracts into artifacts/"

# Optionally compute provenance
if command -v ./target/release/did_provenance >/dev/null 2>&1 || command -v did_provenance >/dev/null 2>&1; then
	echo "Generating provenance using did_provenance"
	if [ -f ./target/release/did_provenance ]; then
		./target/release/did_provenance prove-wasm artifacts/aln_auet.wasm aln_auet || true
		./target/release/did_provenance prove-wasm artifacts/aln_csp.wasm aln_csp || true
		./target/release/did_provenance prove-wasm artifacts/aln_registry.wasm aln_registry || true
		./target/release/did_provenance prove-wasm artifacts/aln_bridge.wasm aln_bridge || true
		./target/release/did_provenance prove-wasm artifacts/energy_router.wasm energy_router || true
	else
		did_provenance prove-wasm artifacts/aln_auet.wasm aln_auet || true
		did_provenance prove-wasm artifacts/aln_csp.wasm aln_csp || true
		did_provenance prove-wasm artifacts/aln_registry.wasm aln_registry || true
		did_provenance prove-wasm artifacts/aln_bridge.wasm aln_bridge || true
		did_provenance prove-wasm artifacts/energy_router.wasm energy_router || true
	fi
fi
