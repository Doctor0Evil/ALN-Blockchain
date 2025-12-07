# ALN Contracts Overview

This repository includes the CosmWasm contracts and off-chain tooling to convert orphan Kujira tokens into ALN-20 AU.ET and CSP tokens.

Contracts:
- `contracts/aln20_auet`: AU.ET non-mintable CW20 contract (instantiate-only mint).
- `contracts/aln20_csp`: CSP non-mintable CW20 contract with transfer restrictions.
- `contracts/bridge`: Bridge contract containing `claim` with replay protection.

Tools:
- `tools/kujira_orphan_scanner`: Detect orphan IBC denoms and produce `artifacts/orphan_ibc.json`.
- `tools/aln_tools`: CLI for `snapshot-hash` and `allocations` to compute H_i and ALN allocations.

Build & Test:
1) Build all contracts to `artifacts/`:
   `bash scripts/build_wasm.sh`
2) Build tools:
   `bash scripts/build_tools.sh`
3) Tests (per crate): e.g. for `contracts/aln_auet`:
   `cd contracts/auet && cargo test`

Deployment (localnet example):
- Follow the runbook in `RUNBOOK_kujira_to_aln20.md` for step-by-step snapshot, allocation, and deployment.


Notes:
- All AU.ET/CSP allocations are defined at instantiate time by `aln_tools` outputs and are non-mintable afterward.
- The bridge contract in `contracts/bridge` depends on off-chain verification of Merkle proofs; in this minimal scaffold, `claim` is trusted to a caller but in production you must verify Merkle proofs or IBC light-client proofs.
- CSP is transfer-restricted by default unless a `transfer_whitelist` is provided at instantiate.
