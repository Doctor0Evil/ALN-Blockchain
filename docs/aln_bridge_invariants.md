# ALN Bridge Invariants (Testable)

This document defines testable invariants and the trust model for ALN bridge operations, AU.ET/CSP alues, and the registry.

Key invariants (formal/testable):

1) Non-tradeable policy (per-contract):
- AU.ET and CSP must not expose unrestricted `Transfer`, `Send`, or `TransferFrom` to arbitrary user addresses.
- Tests:
  - Attempting to call `Transfer` to a regular user should fail.
  - `Spend` API should permit sending to only whitelisted protocol modules.

2) Non-inflation (totalSupply fixed):
- No `Mint` ExecuteMsg exists and no external API can increase `total_supply` after instantiate.
- Tests:
  - `TokenInfo.mint` is `None` in AU.ET and CSP.
  - No admin or execute path can increase `total_supply`.

3) One-way bridge semantics:
- Once values are bridged into ALN, no on-chain or off-chain path allows creating outbound IBC transactions returning the remapped AU.ET/CSP to source chain ~ such flows are not implemented.
- Tests:
  - `aln_bridge` exposes no `Withdraw`/`Unlock`/`IBC` execute messages for outward transfers.

4) Sanitization and UBS verification
- For every registered asset, `ubs_report_hash` must be present and `sanitized_approved` must be true before claims are allowed.
- Tests:
  - Claim attempts fail before registry `sanitized_approved` state is set.
  - UBS report parsed and `sanitized_approved` toggled only by governance.

5) Replay protection and uniqueness
- Each claim for a specific `(address, snapshot_hash)` can only be executed once.
- Tests:
  - Attempt a second claim with same `snapshot_hash` for same address -> fails.

6) Governance-only admin operations
- Adding new registry asset entries, toggling `sanitized_approved`, and updating scaling profiles must be gated to `governance_addr`.
- Tests:
  - Non-governance addresses trying to register assets or mark them as sanitized must be rejected.

7) Immutable scaling profiles per asset
- A registered asset's `scaling_profile_id` cannot be changed; attempts to change must fail or be unimplemented.
- Tests:
  - Attempt to change scaling_profile_id -> fails.

- 8) Proof-of-lock/burn validation (Merkle)
- Bridge claims must be backed by a verified Merkle proof binding the snapshot entry H_i to the registry `merkle_root` for the asset. The bridge recomputes H_i from the snapshot entry fields and verifies the supplied proof using the registry merkle_root, the same hash concatenation order and left/right sibling ordering used by the tooling.
  - Implementation specifics:
    - Leaf hash H_i: SHA256(chain_id || height_be_bytes || denom || address || balance_be_bytes).
    - Node hashing: SHA256(left_child_bytes || right_child_bytes) using the deterministic ordering of leaves (sorted by address:denom:balance) before tree construction.
    - Proof format: an ordered list of { sibling: 32-byte hash, is_left: bool } entries.
    - Claim must carry: snapshot_entry, snapshot_hash (= H_i), merkle_proof.
  - Tests:
    - `claim_with_valid_merkle_proof_succeeds` (valid proof passes).
    - `claim_with_invalid_merkle_proof_fails` (invalid proof fails).
    - `claim_with_modified_snapshot_entry_fails` (tampered snapshot entry fails proof).
- Tests:
  - Without proof, claim should be rejected unless called by allowed operator.

9) Auditability
- All bridge claims should emit `RefactoredAsset` events containing: `{ source_denom, recipient, auet_amount, csp_amount, snapshot_hash }`.
- Tests:
  - Claim emits `RefactoredAsset` with correct values.

10) UBS analyzer integration
- Every registered asset must have a `ubs_report_hash`. A UBS report is a required precondition to set `sanitized_approved=true`.
- Tests:
  - Attempt to `sanitized_approve` an asset with no UBS and expect a governance-only rejection.

---

Trust model:
- The bridge and registry rely on Governance for sensitive operations; Governance must be a DAO/multisig/timelock.
- Proof verification is done off-chain until a light-client proof mechanism is implemented on ALN; claims should require a verified off-chain proof or operator signature.


Operational notes:
- Use `tools/ubs_analyzer` outputs as a required artifact before calling `RegisterAsset`.
- Include `activation_height` (UTC timestamp or block height) to enforce a dispute window (e.g., 60 days) prior to `sanitized_approved` being true and claims allowed.


Testing approach:
- Unit tests at contract level for each invariant.
- Integration (cw-multi-test) for end-to-end claim flows.
- Simulated attack scenarios in `tests/bridge_security.rs` mapping to invariants.
