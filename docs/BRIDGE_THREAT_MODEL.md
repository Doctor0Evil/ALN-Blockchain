# ALN Bridge Threat Model

This document enumerates common threats to the bridge, UBS, and token factory and maps each to code-level mitigations and tests.

## Threat categories

- Verification/Consensus:
  - Threat: Light client failure or malicious merkle roots.
  - Mitigation: Light-client verification (off-chain expected) and deterministic merkle proof checks in `contracts/bridge` using `verify_merkle_proof`.
  - Test: `contracts/bridge` unit tests cover valid/invalid merkle proofs.

- Contract Logic:
  - Threat: Escape path that allows origin tokens to be released by ALN contracts.
  - Mitigation: `contracts/bridge` exposes `Claim` only; there are no `Unlock` functions; `contracts/auet` blocks transfers except to whitelisted modules.
  - Test: `contracts/auet` tests assert transfer failures for non-allowed recipients.

- Key Management:
  - Threat: Governance key compromise enabling `ApproveSanitized` to be set arbitrarily.
  - Mitigation: Governance is expected to be multisig/DAO; Code restricts `ApproveSanitized` to governance caller.
  - Test: `contracts/registry` tests enforce allowed caller restrictions.

- Economic/Systemic:
  - Threat: Big inflows from malicious tokens minting large energy.
  - Mitigation: UBS classifies tokens and `aln_ubs` maps risk score to energy vector; bridge and indexer track toxic vs clean energy totals.
  - Test: Indexer and bridge tests for toxic cap enforcement and UBS classification.

## Tests & Invariants

- Replay protection: indexing and `REFACTORS` registry to prevent double-processing of origin events.
- No reverse bridges: `docs/aln_bridge_invariants.md` has a one-wayness invariant and contract level checks.
- UBS required for sanitized approval: `contracts/registry` enforces `ubs_report_hash` on registration or via `allow_missing_ubs` toggle.
