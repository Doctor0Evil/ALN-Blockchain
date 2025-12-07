# RUNBOOK: Kujira → ALN-20 (AU.ET & CSP)

This runbook lists exact, reproducible steps that transform orphaned Kujira tokens into ALN-20 AU.ET and CSP allocations.

1) Scan for orphan IBC denoms
- `cargo run --manifest-path tools/kujira_orphan_scanner/Cargo.toml -- scan-orphans` (future: CLI argument)
- Output: `artifacts/orphan_ibc.json`

2) Snapshot orphan balances
- Use the chain node or LCD endpoint:
  `GET https://kujira-api.polkachu.com/bank/balances/{address}`
- Filter balances where denom appears in `artifacts/orphan_ibc.json`.
- Save `snapshots_orphan.csv` with columns: chain_id,height,denom,address,balance

- 3) Compute snapshot H_i and Merkle Root (Merkle proof generation)
- `cargo run -- manifest-path <crate> -- snapshot-hash --input snapshots_orphan.csv` (future command)
- Output: `artifacts/snapshot_root.json` with per-entry H_i and global Merkle root.

  The tooling also provides per-entry proofs in `artifacts/merkle_proofs_<asset_id>.json` with an ordered list of sibling hashes and side information. The bridge contract requires claims to include the snapshot entry fields, the H_i, and this Merkle proof.

4) Generate ALN-20 allocations
- `cargo run -- generate-aln20-allocations --input snapshots_orphan.csv --config config/scaling.yaml`
- Output:
  - `artifacts/aln20_auet_allocations.json` (holders + amounts)
  - `artifacts/aln20_csp_allocations.json`
  - `artifacts/aln_init_auet.json` (CosmWasm InstantiateMsg)
  - `artifacts/aln_init_csp.json`

4.5) UBS analysis (required before registry/bridge)
- Run UBS analyzer for each candidate asset bytecode or contract source:
  ```bash
  cargo run --manifest-path tools/ubs_analyzer/Cargo.toml -- ibc_xxx /path/to/contract_source_or_bytecode
  ```
  Output: `artifacts/ubs_report_ibc_xxx.json`. Attach this report to registry entries in step 6.

5) Build contracts
 - `bash scripts/build_wasm.sh` (build `aln20_auet`, `aln20_csp`, `aln_bridge`, `aln_registry`)
  - The recommended CI run also optimizes wasm and stores artifacts in `artifacts/`.
  - See `SETUP.md` for local build steps and how to create `artifacts/`.

6) Deploy on ALN localnet/testnet
- Upload & instantiate AU.ET and CSP with `alnd tx wasm store` and `alnd tx wasm instantiate` referencing `aln_init_*.json`.

7) Fund bridge & enable claims
- Pre-fund bridge contract with AU.ET and CSP as needed (transfer to bridge account).
- Off-chain: verify H_i and merkle proofs for each claim.

8) Claim process
- Operator calls ALN bridge `claim(snapshot_hash, recipient)` with snapshot proof (or off-chain verification before submitting).
- Bridge checks `claimed[snapshot_hash] == false`, moves tokens and marks as claimed.

9) Auditing
- Recompute H_i and B_E, B_S from published snapshots and scaling config to confirm totals.

10) CI
- Add CI steps: `cargo fmt`, `cargo clippy`, `cargo test`, `bash scripts/build_wasm.sh`, and a `wasm-check` step.

Notes
- All commands that interact with chain RPCs must rely on environment variables for RPC URL and signer (do not store secrets in repo).
