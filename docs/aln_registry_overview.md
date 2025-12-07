# ALN Registry Overview

The `aln_registry` stores metadata about assets that are eligible for ALN remapping.

Fields per asset:
- id (string)
- source_chain (e.g., kaiyo-1)
- source_denom (e.g., ibc/xxx)
- snapshot_height
- merkle_root
- ubs_report_hash
- scaling_profile_id
- activation_height
- sanitized_approved (bool)

Key flows:
- Governance `RegisterAsset` to add an asset to registry.
- Governance `ApproveSanitized` to set `sanitized_approved = true` and attach UBS report hash.
- ALN Bridge queries registry on claim to ensure asset is approved.

Security and audit:
- Only the governance address may register or approve assets.
- UBS reports are required prior to approval (off-chain analysis required).
- Activation height enforces a dispute-window for asset claims.
