# CosmWasm Local Dev Playbook (Sanitized)

This playbook documents repo-specific commands to build, test, and optimize CosmWasm artifacts and mirrors the CI behaviors. It maps generic instructions to the repository's scripts and tasks.

---

## 1. Build & Optimize WASM (Repo scripts)
Use the provided script to build and optimize all CosmWasm contracts. The script performs:
- cargo build for each contract crate that targets wasm32
- wasm-opt (binaryen) optimization
- optional DID provenance generation if `did_provenance` binary is installed

```
# Build and optimize (POSIX)
./scripts/build_wasm.sh

# PowerShell (Windows)
.
./scripts/build_wasm.ps1  # if present, otherwise run the Linux script in WSL/Cygwin
```

Artifacts are stored in the `artifacts/` folder for traceability and CI uploads.

---

## 2. Run cw-multi-test Integration Tests
Use cargo test and the repo's integration test harness.

```
# Run all Rust tests in the workspace (unit + integration)
cargo test --workspace --locked

# Run only a specific contract tests (e.g., bridge)
cargo test -p aln_bridge --locked

# Run multi-test integration tests
RUST_LOG=debug cargo test -p aln_bridge -- --nocapture
```

The repository also exposes a `scripts/run_all_tests.sh` to run the full dev test suite.

---

## 3. wasm_artifacts Job (Mirrors CI)
CI runs a `wasm-artifacts` job to build + optimize wasm then compute provenance and size analysis. Locally, this is covered by `scripts/build_wasm.sh`. Reproduce the CI job using a single known command:

```
# Build + optimize + generate provenance (requires did_provenance tool)
./scripts/build_wasm.sh
```

If you want the exact actions mirrored (Rust toolchain setup and wasm32 target only):

```
# Set up wasm target
rustup target add wasm32-unknown-unknown

# Build the contracts
RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown -p aln_bridge -p aln_registry -p aln_auet -p aln_csp -p energy_router

# Optimize with wasm-opt (binaryen)
wasm-opt -Oz -o artifacts/aln_bridge.optimized.wasm artifacts/aln_bridge.wasm

# Optionally compute provenance
./target/release/did_provenance prove-wasm artifacts/aln_bridge.wasm aln_bridge
```

---

## 4. Measure Gas for Merkle Verify Locally
If you want to measure the gas of `verify_merkle` or the bridge claim flows:
1) Run a local wasmd/gaia node and ensure your CLI tools are configured.
2) Use the explorer to store & instantiate the contract and call the contract on a specific block.

The repo offers a simple shell example in `docs/` (or follow the generic playbook commands). For an automated bench, use the Rust `benches/` harness (if present) and `cargo bench -p aln_bridge`.

---

## 5. Off-chain Signatures & Proof Verification
To emulate the off-chain proof flow (e.g., Merkle root signing + on-chain verification), use the `tools/aln_tools` subcommands to compute snapshot roots and per-leaf proofs; sign the canonical proof using a local private key and use `cw-multi-test` to submit the proof to the bridge contract. The repo's `tests/integration` holds examples of these flows.

Example flow (pseudocode):
```
# Generate artifacts
cargo run -p aln_tools -- snapshot-hash --input snapshots.csv --artifacts artifacts/snapshots
# Sign message using your key
# Submit in cw-multi-test or in local chain
.
```

---

## 6. Indexer Migrations & Tests (Local + CI)
The `crates/aln_indexer` crate includes SQL migrations and a fixture file to populate a Postgres DB for testing.

To run migrations locally:
```
psql -U postgres -d aln_indexer_test -f crates/aln_indexer/migrations/V1__init.sql
psql -U postgres -d aln_indexer_test -f crates/aln_indexer/migrations/V2__rollup_unique_index.sql

# Load fixtures
psql -U postgres -d aln_indexer_test -f crates/aln_indexer/fixtures/fixture.sql

# Run indexer tests
DATABASE_URL=postgres://postgres:password@localhost:5432/aln_indexer_test cargo test -p aln_indexer
```

CI mirrors this behavior in the `indexer-tests` job using a Postgres service. Ensure the DATABASE_URL environment variable points to the service.

---

## 7. Useful Repo-specific Commands
- Run all tests (Rust + Node): `./scripts/run_all_tests.sh` or PowerShell variant
- Build all wasm & compute provenance: `./scripts/build_wasm.sh`
- Run indexer locally against a DB: `DATABASE_URL=... cargo run -p aln_indexer -- follow-chain`
 - Run the CEM calibration CLI:
	 - `cargo run -p cem -- --subject 1 --session 1 --input examples/sample_dataset.json`
 - Run the CEM calibration CLI and expose metrics:
	 - `cargo run -p cem -- --subject 1 --session 1 --input examples/sample_dataset.json --metrics-addr 127.0.0.1:9889`
		 - The `--metrics-addr` flag enables a Prometheus scrape endpoint `/metrics` and is optional: if omitted, the metrics server is not started.
		 - To enable structured JSON logs for CEM and to control verbosity, set `RUST_LOG`, e.g., `export RUST_LOG=info`.
 - Run the indexer migration and local follow-chain with DB and RPC tuneable via env variables:
	 - `./scripts/migrate_db.sh && DATABASE_URL=postgres://postgres:password@localhost:5432/aln_indexer_test RPC_URL=http://127.0.0.1:26657 ./scripts/run_indexer_local.sh`

---

## 8. Tips & Best Practices
- Keep `artifacts/` generated files in .gitignore; CI uploads them as action artifacts for traceability.
- Ensure `RUSTFLAGS='-C link-arg=-s'` when building for release to strip small amounts of data for consistent WASM sizes.
- For deterministic tests, avoid time-based random data; prefer fixed block heights and key pairs in test fixtures.
- If you need to run a local `wasmd`, use the `aln/core` scripts or `docker-compose` to set up a local dev chain for repeatable gas measurements.

---

If you'd like, I can add a PowerShell-friendly variant of `scripts/build_wasm.sh` and add a small `README` under `crates/aln_indexer/README.md` describing how to run the indexer and follow-chain locally.
