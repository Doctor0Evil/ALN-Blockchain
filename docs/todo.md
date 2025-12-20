# Project TODOs (canonical snapshot)

1. Draft ALN energy spec & canonical config (aln_energy_profile_v1.json + aln-energy-spec.md) — **completed**
2. Implement asset compression and property tests (compress_asset, to_u128_floor, proptest) — **completed**
3. Create `aln-energy-ledger` crate with EnergyEvent, invariants, and unit tests — **completed**
4. Implement `sealed_refactor` module + `UBSecurity` classifier and replay protection DB/table — **in-progress**
5. Add `aln-nanoneuro` crate: `NanoNeuroChannel`, biophysical checks, apply_nano_actuation and tests — not started
6. Implement `CrateAsset` build energy model and CI/CD energy sinks — not started
7. Implement ALN chat grading and search verification modules (hashstamping, grading pipeline) — not started
8. Design API and SDKs (ingest, epochs/finalize, search proofs, verify endpoints) and docs — not started
9. CI: epoch Merkle root generation, anchoring jobs, testnet anchor and ledger audits — not started
10. Write formal proofs (LaTeX appendices) and property-based proofs for monotonicity/non-minting — not started
11. Integration tests and smoke tests (virtual wetware mode, simulated hardware safety tests) — not started
12. Release artifacts: checksums, hex validity stamps, governance proposal + audit report — not started
13. Add CI job to run `cargo test` for `aln-energy` and create a developer setup document — **in-progress**

> Last updated: 2025-12-20
