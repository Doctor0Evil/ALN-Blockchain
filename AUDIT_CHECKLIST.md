# Audit Checklist: Kujira → ALN-20 AU.ET / CSP Pipeline

1) Orphan detection
- Run `cargo run --manifest-path tools/kujira_orphan_scanner/Cargo.toml`.
- Verify `artifacts/orphan_ibc.json` lists denoms and reasons.
- Confirm `base_denom` not in allowlist or registry.

2) Snapshot correctness
- Ensure `snapshots_orphan.csv` contains (chain_id,height,denom,address,balance).
- For each CSV row, recompute H_i using `tools/kujira_orphan_scanner/src/snapshot_hash.rs:hash_entry`.
- Confirm H_i values are consistent with on-chain data at height.

3) Merkle root
- Build a Merkle tree over H_i values and compare with stored `snapshot_root`.
- Verify branch proofs are consistent by recomputing branches for random entries.

4) Scaling math
- Recompute A_src and B_ALN = floor(A_src * c * 10^d_aln) for c_E and c_S in `config/scaling.yaml`.
- Sum B_E and B_S across addresses; they should equal AU.ET and CSP totalSupply.

5) Contract immutability
- Confirm `ALN20NonMintable` (Solidity) constructor-only _mint, no external mint function.
- For CosmWasm CW20 variant, ensure no Mint ExecuteMsg exists.
- Confirm metadata fields in the contract state after instantiate: sourceChainId, sourceDenom, snapshotHeight, snapshotRoot.

6) Claim and bridge
- Confirm `claimed[snapshot_hash]` mapping exists in the bridge contract.
- Attempt a double-claim test to ensure claiming fails on repeated proofs.

7) Security
- Ensure no hidden code paths that may call internal mint functions.
- Verify role access for any admin functions (finalize minting if present).
- Confirm that any allowed transfer for CSP is documented and restricted.

8) Legal
- Confirm conversion only applies to operator-held balances at snapshot height.
- Confirm publish date/height and allowlist criteria are preserved in docs.

9) CI
- Validate `cargo test` and `cosmwasm-check` run in CI.
- Confirm contract artifacts are reproducible and `artifacts/` created from them.

10) Reproducibility
- Verify `artifacts/aln_init_*.json` lead to the same contract total supply when instantiated on a local node; totals must match computed sums.


Actionables: share `artifacts/*` and proofs for independent auditors.
