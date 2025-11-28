# Law-Enforcement Assist Guide

- Capability tier for cooperative, consented assistance

## Requirements
- Verified authority credential (DID/VC)
- Explicit user consent
- Minimum reputation threshold
- Jurisdictional compliance flags

## Channels
- Encrypted, pinned-key routing to registered authority endpoints
- On-chain hash logs of channel usage; no raw content stored

## Tasks
- Authority-issued tasks: `task_id`, jurisdiction, allowed_actions[], reward model, expiry
- Signed completion updates `GoodDeedRecord` and reputation
