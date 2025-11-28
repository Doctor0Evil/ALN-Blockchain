# Augmented Reputation Model

- Non-transferable, DID-linked records
- Derived from `GoodDeedRecord` and policy violations; explicit formulas

## Records
- GoodDeedRecord: `user`, `deed_type`, `verifying_authority`, `timestamp`, `jurisdiction`
- AugmentedReputation: `positive_events_count`, `negative_events_count`, `law_enf_assist_events`, `last_review_block`, `computed_score`

## Safeguards
- No punitive social-credit penalties from arbitrary data
- Transparent reason codes, appeal hooks, jurisdiction tags

## Thresholds
- Capability tiers based on computed_score and violation history
- Law-enf assist requires higher thresholds and authority sponsorship
