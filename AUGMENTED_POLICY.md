# Augmented User Policy Model

- Anchors: `AugmentedUserProfile`, `AugmentedEnergyState`, `AugmentedUserPolicy`
- Engine: On-chain policy checks, auditable events, jurisdictional compliance

## Data Types
- AugmentedUserProfile: `user_did`, `linked_wallets[]`, `augmentation_ids[]`, `jurisdiction_tags[]`, `compliance_mode`
- AugmentedEnergyState: `cognitive_level`, `physical_level`, `grid_power_draw_watts`, `device_ids[]`, `last_update_block`
- AugmentedUserPolicy: `max_grid_draw_watts`, `max_device_class_permitted[]`, `allowed_capability_levels`, `audit_required`, `jurisdiction_constraints[]`

## Actions & Auditing
- Registry of `AugmentationAction` ids; UE adapter enforces `CheckAugmentationActionAllowed`
- Emit events for allow/deny with energy deltas, timestamp, policy version

## Enforcement
- Energy-aware throttling: downgrade capabilities when above thresholds
- Human-rights safeguards: no opaque social credit; appeals and transparency required

## Law-Enforcement Assist Tier
- Requires authority credential, consent flags, minimum reputation, jurisdiction compliance
- Secure channels routed to registered authority endpoints; hash-only logs on-chain

## Overrides & Privacy
- Per-user/system policy overrides; device caps
- Data minimization: store hashes and summaries only; respect jurisdiction privacy rules
