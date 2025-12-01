# ALN Blockchain Workspace Instructions

## Progress Checklist

- [x] Verify that the copilot-instructions.md file in the .github directory is created
- [x] Clarify Project Requirements
- [x] Scaffold the Project
- [x] Customize the Project
- [x] Install Required Extensions (None required)
- [ ] Compile the Project (Requires Node.js installation)
- [ ] Create and Run Task
- [ ] Launch the Project
- [x] Ensure Documentation is Complete

## Project Overview

ALN blockchain implementation with:
- Core blockchain runtime (consensus, state, ALN syntax)
- Explorer web UI with migragraph and activity charts
- Non-custodial browser wallet
- DAO governance with CHATAI token
- Canto→ALN migration bridge
- Node.js/JavaScript only (no Python)

## Development Guidelines

- Use npm workspaces for monorepo structure
- All chainlexemes follow `/aln/core/spec/aln-syntax.aln` format
- Non-custodial wallet: keys never leave browser
- Governance via governograms encoded in ALN
- Migration events tracked as migragraph data points
- Safety constraints via QPU.Math+ hooks

## Workspace Structure

```
/aln/
  core/          # Consensus, state, ALN runtime
  explorer/      # Web UI with charts
  wallet/        # Browser wallet integration
  migration/     # Canto bridge logic
  dao/           # Governance and CHATAI
  tests/         # Unit and e2e tests
  tools/         # Linter and dev tools
  docs/          # Documentation
```

## Coding Standards

- Use deterministic state transitions
- All writes via chainlexemes, not direct DB updates
- Error codes from `/aln/core/logging/errors.aln`
- Structured logging with node_id, block_height, tx_hash
- Rate limiting on public endpoints

## Treasury & Constants

- **Single Source of Truth**: Import `ALN_TREASURY_ADDRESS` and all protocol constants from `/aln/core/config/constants.ts`
- **Governance Addresses**: Import `GOVERNANCE_ADDRESSES` and `GOVERNANCE_POLICY_TAGS` from `/aln/core/config/governance.js` instead of hard-coding council destinations
- **NEVER** hard-code treasury addresses, gas limits, governance periods, or TPS targets
- Use helper functions: `isTreasuryLive()`, `isValidJurisdiction(tag)`, `isValidTreasuryAddress(addr)`
- Treasury is reserved; `ALN_TREASURY_LIVE = false` until mainnet genesis

## Chat-Native Metadata

- All transactions MAY include optional header fields:
  - `chat_context_id` (UUID, max 36 chars)
  - `transcript_hash` (SHA-256 hex, max 64 chars)
  - `jurisdiction_tags` (array, max 10 tags from `JURISDICTIONS` constant)
- Parser WARNS if governance/migration ops lack `chat_context_id` or `transcript_hash`
- State store captures metadata and appends audit records: `audit:<from>:<timestamp>:<op_code>`
- Use `withChatMetadata(tx, meta)` helper in wallet builder
- CLI: `aln tx` supports `--chat-context-id`, `--transcript-hash`, and `--jurisdiction-tags` plus custodian signing flags (`--custodian-root`, `--custodian-label`, `--custodian-passphrase-env`)

## Compliance Routing

- All token deployments, migrations, and sensitive ops MUST route through `/aln/compliance_routing/router.aln`
- Compliance policies (e.g., `JFMIP-24-01`) enforce:
  - Prepaid ALN fees before deployment
  - Naming/impersonation checks
  - Auto-refactoring of unsafe patterns (honeypots, self-destruct drainers)
  - Audit event emission with transcript hashes
- Validation reports include `metadata.policy` field indicating applied policy

## Augmented User Policies

- Core data types: `AugmentedUserProfile`, `AugmentedEnergyState`, `AugmentedUserPolicy`
- On-chain policy engine checks: `isActionAllowed(user, action_id, energy_state)`
- All augmentation activations in UE MUST call policy check first
- Emit audit events for allow/deny with energy deltas, timestamps, policy versions
- Law-Enforcement Assist tier requires: authority credential, consent, reputation threshold, jurisdiction compliance
- Reputation is non-transferable; derived from `GoodDeedRecord` + violations
- See `AUGMENTED_POLICY.md`, `AUGMENTED_REPUTATION.md`, `LAW_ENF_ASSIST_GUIDE.md`

## Future-Tech Domain Scaffolding

- Domain folders with policy/type definitions:
  - `/aln/nanoswarm` – safety classes and swarm policies
  - `/aln/bci_neuromorphic` – BCI safety levels, compliance routes
  - `/aln/superintelligence` – capability constraints
  - `/aln/augmented_city` – node types and governance models
  - `/aln/compliance_routing` – op_code routing and jurisdiction enforcement
- Module syntax: `aln_module "name" { }` and `aln_policy "name" { }`
- Parser extracts module metadata; integrates with QPU.Math+ hooks

## Security & Malware

- Malware domains: ransomware, keyloggers, drainers, phishing, supply_chain, logic_timebomb
- All contracts/transactions pass through malware detection before commit
- ML threat hooks: `loadSignatures()`, `scorePayload()`, `updatePolicies()` now hydrate from `ThreatFeedIngestor` (file/HTTPS sources) with fallbacks
- Quarantine workflow for flagged contracts; cannot deploy until cleared

## Custodian & Signing

- Wallet + CLI signing uses **Ed25519** (`@noble/ed25519`) via `signTx` / `verifyTxSignature`
- Use `/aln/security/key_custodian.js` to encrypt validator/wallet keys at rest (AES-256-GCM + scrypt)
- CLI custodian flags auto-create/load sealed keys and sign without exposing private material
- Runtime policy/consensus MUST reject tx lacking required metadata before mempool admission (`AugmentedPolicyEngine.validateTransaction`)

## Token Creation Pipeline

- Workflow: `submit_token_blueprint` → `analyze_and_refactor` → `review` → `approve_or_reject`
- Enforce prepaid ALN fee (from constants) before finalization
- Auto-refactor unsafe patterns; reject impersonation/scam names
- All token ops require compliance routing validation

## Guardrail Verification

- Run `npm run verify:scaffold` to check:
  - Presence of core docs (AUGMENTED_POLICY.md, AUGMENTED_REPUTATION.md, LAW_ENF_ASSIST_GUIDE.md)
  - Single-source constants usage
  - Compliance routing files
  - Wallet chat metadata helper
- CI should run this before merge
