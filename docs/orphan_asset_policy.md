# Orphan Asset Policy

This policy outlines the rules and timeline for classifying Kujira tokens as "orphan" and registering them for ALN remapping.

1) Candidate Identification
- The `tools/kujira_orphan_scanner` collects all IBC denom traces, identifies base denoms not in `config/known_bases.json` and outputs `artifacts/orphan_ibc.json`.
- Operators may also manually flag specific denoms.

2) Publication & Dispute Window
- Candidate assets are published to `artifacts/orphan_candidates.json` including: {denom, path, snapshot_height, candidate_timestamp}.
- There is a configurable dispute window (default 60 days): only after this window expires can the asset be registered on-chain for bridging.
- During the dispute window, asset owners can submit evidence of ownership or request exclusion.

3) UBS Analyzer Policy
- No asset may be registered or sanitized for bridging unless `tools/ubs_analyzer` generates a UBS report with severity 'low' or 'medium' (high severity rejects the asset).
- UBS report must be attached as `ubs_report_hash` in the registry entry.

4) Governance Approval
- Registration of asset in the on-chain `aln_registry` requires governance action (DAO/multisig/timelock).
- `sanitized_approved` is set by governance after reviewing UBS reports and dispute outcomes.

8) Merkle Binding
- Any registration for a batch snapshot must include a `merkle_root` for the snapshot and the CLI artifacts must include per-entry `merkle_proofs_<asset_id>.json`. Bridge claims are only accepted if a valid Merkle proof binds the snapshot entry to the `merkle_root` stored on-chain.

5) Activation Height
- Every registry entry must have an `activation_height` which is the earliest block height where claims are allowed (enforce dispute window on-chain).

6) Auditing
- All registrations and approvals are recorded on-chain with event logs.
- The pipeline publishes `artifacts/registry.json` summarizing on-chain registry entries for auditors.

7) Legal/Compliance
- Operators must ensure they do not claim or convert assets for balances they do not own.
- Any suspicious or potentially contested assets should be escalated to legal and placed in permanent hold (no registration).
