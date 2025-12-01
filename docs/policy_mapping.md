# ALN Policy → Implementation Mapping

This document links each ALN policy domain to the runtime modules, CLI entry points, and tests that enforce it. Use it to confirm coverage before shipping or enabling CI jobs.

## Governance & Roles

| Policy Source | Runtime / Config | CLI / Surfaces | Tests / CI | Notes |
|---------------|------------------|----------------|------------|-------|
| `aln/dao/spec/governogram.aln`, `aln/core/spec/aln-syntax.aln` (governance op codes) | `aln/core/config/constants.ts` (quorum/threshold), `aln/core/config/governance.js` (`GOVERNANCE_ADDRESSES`, `GOVERNANCE_POLICY_TAGS`), `aln/core/runtime/aln_parser.js` (metadata warnings), `aln/augmented_policy/policy_engine.js` (`validateTransaction`) | `aln/core/cli/aln_node_cli.js` (`tx governance-proposal`, `tx governance-vote`) | `aln/tests/unit/parser_metadata.test.js`, `aln/tests/unit/augmented_policy.test.js` | CLI currently exposes council routing + metadata flags; proposal categories map directly to governance constants. Need future CLI verbs for council-specific policy tags (TODO). |

## Transactions & Chat Metadata

| Policy Source | Runtime / Config | CLI / Surfaces | Tests / CI | Notes |
|---------------|------------------|----------------|------------|-------|
| `.github/copilot-instructions.md` (chat metadata rules), `aln/core/spec/aln-syntax.aln` (header fields) | `aln/core/api/http_server.js` (metadata normalization + policy checks), `aln/core/consensus/solo_consensus.js` (policy gate before mempool), `aln/core/state/state_store.js` (audit log) | `aln/core/cli/aln_node_cli.js` (`--chat-context-id`, `--transcript-hash`, `--jurisdiction-tags`) | `aln/tests/unit/parser_metadata.test.js` | CLI help documents metadata flags; TODO(ci) comments added for future integration tests capturing runtime validation. |

## Key Custody & Signing

| Policy Source | Runtime / Config | CLI / Surfaces | Tests / CI | Notes |
|---------------|------------------|----------------|------------|-------|
| `aln/security/key_custodian.aln` (implicit via docs), `.github/copilot-instructions.md` (custodian requirements) | `aln/security/key_custodian.js` (AES-256-GCM + scrypt sealing), `aln/wallet/tx_builder.js` (Ed25519 signing), `aln/core/cli/aln_node_cli.js` (custodian flags) | CLI custodian options (`--custodian-root`, `--custodian-label`, `--custodian-passphrase-env`) | `aln/tests/unit/key_custodian.test.js`, `aln/tests/unit/tx_builder.test.js` | TODO(policy) note in custodian module to require usage scopes; CLI TODO(ci) marker requests runtime coverage. |

## Threat Feeds & Malware Scoring

| Policy Source | Runtime / Config | CLI / Surfaces | Tests / CI | Notes |
|---------------|------------------|----------------|------------|-------|
| `aln/security/malware.aln`, `aln/token_creation/pipeline.aln`, `aln/migration/analyzer.aln` | `aln/security/threat_feed.js` (fetch → normalize), `aln/security/ml_hooks.js` (score + evidence), `aln/token_creation/pipeline.js`, `aln/migration/ingest.js` | No direct CLI yet (future `aln tx analyze-blueprint` planned) | `aln/tests/unit/token_pipeline.test.js` (malware scoring logic) | TODO(policy) comment in `ThreatFeedIngestor` reminds to gate feeds per jurisdiction; CI coverage pending for ingestion path. |

## Execution / UE Integration Policies

| Policy Source | Runtime / Config | CLI / Surfaces | Tests / CI | Notes |
|---------------|------------------|----------------|------------|-------|
| `aln/augmented_policy/policy_engine.aln`, `aln/energy/grid_allocation.aln`, `aln/neuromorphic/adapter_sandbox.aln` | `aln/augmented_policy/policy_engine.js` (capability checks), `aln/energy/energy_integration.js`, `aln/ue_integration/ue_rpc_adapter.js` | HTTP endpoints `/policy/user/:did`, `/policy/check`, `/energy/state/:did` | `aln/tests/unit/augmented_policy.test.js` | CLI currently does not expose UE hooks; HTTP API wiring now enforces policies before tx acceptance. |

## Gaps / TODO Summary

1. **Governance CLI Coverage** – add subcommands/tests for policy tags and law-enforcement proposals once runtime endpoints exist.
2. **Custodian Usage Scope** – extend `KeyCustodian.ensureKey()` signature to require usage metadata + audit logging.
3. **Threat Feed CI** – add ingestion unit tests that mock HTTPS/file feeds and verify dedupe/risk tiering.
4. **Policy CI Scripts** – describe canonical CI flows in `docs/ci_checklist.md` (see companion document) so DevOps can wire jobs without local runtime.
