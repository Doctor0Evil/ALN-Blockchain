# ALN Blockchain – Augmented User Architecture

## Executive Summary

ALN is a **compliance-first, security-native blockchain language and platform** designed to:
- Automatically enforce security and fairness at the protocol level
- Gate token creation through malware detection and compliance policies
- Model ALN tokens as stable $1.00-equivalent credits backed by diversified reserves
- Provide non-transferable reputation and DID-linked good-deed records
- Enable augmented-user energy accounting and capability control
- Support Canto migration with malware hardening
- Integrate with Unreal Engine for game-based augmentation UX

---

## Architecture Overview

### Core Modules

1. **Compliance Routing** (`aln/compliance_routing/`)
   - Maps op_codes to required jurisdiction tags and policies
   - Enforces JFMIP-24-01 and other compliance policies
   - Auto-refactors unsafe patterns (honeypots, drainers)

2. **Security & Malware** (`aln/security/`)
   - Taxonomy: ransomware, keylogger, drainer, phishing, supply_chain, logic_timebomb
   - ML threat hooks: signature loading, payload scoring, policy updates
   - Quarantine workflow for flagged contracts

3. **Token Creation Pipeline** (`aln/token_creation/`)
   - Phases: submit_blueprint → analyze_and_refactor → review → approve_or_reject → deploy
   - Enforces prepaid ALN fees (from `ALN_TREASURY_ADDRESS`)
   - Rejects impersonation, blocks malicious patterns

4. **Stability & Reserve** (`aln/stability/`)
   - Diversified backing: BTC (40%), USD basket (30%), EUR (15%), JPY (10%), GBP (5%)
   - Commit-reveal rebalancing with TWAP anti-front-run guards
   - Proof-of-reserve publishing (Merkle tree + signed attestation)

5. **Canto Migration** (`aln/migration/`)
   - Ingests Canto contracts via RPC
   - Analyzes for malware and compliance
   - Generates ALN-native adapters with hardened security

6. **Reputation & Credit** (`aln/reputation/`)
   - Non-transferable `GoodDeedRecord` linked to DIDs
   - Transparent reputation scoring (positive/negative events)
   - Appeal workflow with human-rights safeguards

7. **Energy & Web5** (`aln/energy/`)
   - Personal energy state (cognitive/physical levels, grid draw)
   - Grid allocation fairness (max 10% per actor)
   - Fraud detection and reversal

8. **Augmented Policy Engine** (`aln/augmented_policy/`)
   - On-chain policy checks: `isActionAllowed(user, action_id, energy_state)`
   - Capability tiers: BASIC, ADVANCED, LAW_ENF_ASSIST
   - Reputation and energy budget enforcement

9. **Neuromorphic Adapters** (`aln/neuromorphic/`)
   - Opt-in sandbox bridge with consent and restricted write paths
   - Channels: read_telemetry, propose_energy_adjustment, submit_governance_suggestion
   - Rate-limited, reputation-gated

10. **UE Integration** (`aln/ue_integration/`)
    - Blueprint/C++ nodes: `GetAugmentedUserPolicy`, `CheckAugmentationActionAllowed`
    - RPC adapter with 30s caching
    - Audit transparency UI in Unreal Engine

---

## Constants & Treasury

**Single Source of Truth**: `/aln/core/config/constants.ts`

```typescript
export const ALN_TREASURY_ADDRESS = 'ALN18sd2ujv24ual9c9pshtxys6j8knh6xaek9z83t';
export const ALN_TREASURY_LIVE = false; // Reserved until mainnet genesis
```

**Never hard-code** treasury addresses, gas limits, governance periods, or TPS targets. Import from `constants.ts`.

---

## Chat-Native Metadata

All transactions MAY include optional header fields:
- `chat_context_id` (UUID, max 36 chars)
- `transcript_hash` (SHA-256 hex, max 64 chars)
- `jurisdiction_tags` (array, max 10 from `JURISDICTIONS`)

**Parser warnings** if governance/migration ops lack `chat_context_id` or `transcript_hash`.

**State store** captures metadata and appends audit records: `audit:<from>:<timestamp>:<op_code>`.

**Wallet helper**:
```javascript
const tx = buildGovernanceVoteTx(voter, proposalId, support, nonce);
withChatMetadata(tx, { 
  chat_context_id: uuid, 
  transcript_hash: hash, 
  jurisdiction_tags: ['US_federal', 'JFMIP'] 
});
```

---

## Token Creation Workflow

1. **Submit Blueprint** – Validate structure, check prepaid ALN fee
2. **Analyze & Refactor** – ML scoring, naming checks, auto-refactor unsafe patterns
3. **Review** (if needed) – DAO escalation for edge cases
4. **Approve/Reject** – Emit audit event with reasons
5. **Deploy** – Deploy transformed contract, emit deployment event

**Tests**: `aln/tests/unit/token_pipeline.test.js`

---

## Augmented User Policies (25-Step Model)

### Data Types
- **AugmentedUserProfile**: `user_did`, `linked_wallets[]`, `augmentation_ids[]`, `jurisdiction_tags[]`, `compliance_mode`
- **AugmentedEnergyState**: `cognitive_level`, `physical_level`, `grid_power_draw_watts`, `device_ids[]`, `last_update_block`
- **AugmentedUserPolicy**: `max_grid_draw_watts`, `max_device_class_permitted[]`, `allowed_capability_levels`, `audit_required`, `jurisdiction_constraints[]`

### Capability Tiers
- **BASIC**: Open to all compliant users
- **ADVANCED**: Requires `reputation_score >= 50` and zero severe violations
- **LAW_ENF_ASSIST**: Requires `reputation >= 70`, authority sponsorship, consent, jurisdiction compliance

### Policy Checks (UE Integration)
1. **GetAugmentedUserPolicy(DID)** → Fetch effective policy
2. **GetAugmentedEnergyState(DID)** → Current energy levels
3. **CheckAugmentationActionAllowed(ActionId, User, EnergyState)** → `(allowed, reason)`

### Audit Events
Every action emits:
- `user`, `action_id`, `allowed`, `energy_before/after`, `timestamp`, `jurisdiction`, `policy_version`

### Human-Rights Safeguards
- No opaque social credit penalties
- Transparent reason codes, appeal hooks
- Cannot drop capabilities below safe baseline

---

## Reputation System

### GoodDeedRecord
- `user`, `deed_type`, `verifying_authority`, `timestamp`, `jurisdiction`, `evidence_refs[]`
- Non-transferable, DID-linked

### Reputation Score
- Formula: `max(10, min(100, positive*10 - negative*20))`
- Cannot drop below baseline (10)

### Appeals
- User submits appeal with reason
- Authority resolves: APPROVED | DENIED
- Emit transparency event

**Tests**: `aln/tests/unit/augmented_policy.test.js`

---

## Canto Migration

1. **Ingest** – Fetch contract from Canto RPC
2. **Analyze** – Run malware hooks, identify vulnerabilities
3. **Generate Adapter** – Transform to ALN chainlexeme, apply refactors
4. **Deploy** – Emit `migration_scan_result` for explorer

**Threat Intel**: Continuously updated from Canto incident data

---

## Guardrail Verification

**Run before commit**:
```powershell
npm run verify:scaffold
```

**Checks**:
- Presence of `AUGMENTED_POLICY.md`, `AUGMENTED_REPUTATION.md`, `LAW_ENF_ASSIST_GUIDE.md`
- Single-source constants usage
- Compliance routing files
- Wallet chat metadata helper

---

## Tests

**Run all tests**:
```powershell
npm test
```

**Coverage**:
- Token pipeline (malicious name/code rejection, auto-refactoring, fee enforcement)
- Augmented policy (reputation thresholds, energy budgets, capability checks)
- Parser (module syntax, chat metadata warnings, jurisdiction tag validation)

---

## Unreal Engine Integration

**RPC Adapter**: `aln/ue_integration/ue_rpc_adapter.js`

**Blueprint Nodes**:
- `GetAugmentedUserPolicy(DID)`
- `GetAugmentedEnergyState(DID)`
- `CheckAugmentationActionAllowed(ActionId)`

**Example Flow**:
1. Player activates augmentation (e.g., ENHANCED_VISION)
2. UE calls `CheckAugmentationActionAllowed`
3. If denied, display reason and block activation
4. If allowed, proceed and emit audit event

**Caching**: Policy/energy cached 30s; action checks real-time

---

## Neuromorphic Adapters

**Opt-in Bridge** with consent and rate limits:
- **Read Telemetry** – Privacy-preserving summaries (hashes only)
- **Propose Energy Adjustment** – Queued for user review, no direct writes
- **Submit Governance Suggestion** – Reputation-gated, audit logged

**Safety**: No direct state writes, `reputation >= 50` required

---

## Stability & Economics

### ALN Token Model
- **Peg**: $1.00 equivalent credit (not speculative asset)
- **Backing**: Diversified reserve (BTC + fiat baskets)
- **Rebalancing**: Commit-reveal every ~1 day, TWAP guards

### Proof-of-Reserve
- Merkle tree of assets
- Signed by custodian keys
- Published on-chain periodically

### Volatility Cap
- Max deviation: 2% from $1.00
- Circuit breakers if exceeded

---

## Law-Enforcement Assist

**Requirements**:
- Verified authority credential (DID/VC)
- Explicit user consent
- Minimum reputation threshold
- Jurisdictional compliance flags

**Channels**:
- Encrypted, pinned-key routing to registered authorities
- On-chain hash logs (no raw content)

**Tasks**:
- Authority issues task with `task_id`, `allowed_actions[]`, `reward_model`, `expiry`
- User accepts/declines
- Completion updates `GoodDeedRecord` and reputation

**Governance**: Clear boundaries, no hidden surveillance

---

## Copilot Guidance

When generating ALN code:
1. **Import constants** – Never hard-code addresses/limits
2. **Use chat metadata** – Add `chat_context_id`/`transcript_hash` for governance/migration
3. **Enforce policies** – Route through compliance_routing
4. **Test security** – Write tests for malware detection, refactoring
5. **Audit transparency** – Emit events for all sensitive actions
6. **Human-rights constraints** – No opaque penalties, allow appeals

**See**: `.github/copilot-instructions.md` for full context

---

## Quick Start

1. **Install dependencies** (requires Node.js):
   ```powershell
   npm install
   ```

2. **Verify scaffold**:
   ```powershell
   npm run verify:scaffold
   ```

3. **Run tests**:
   ```powershell
   npm test
   ```

4. **Start node**:
   ```powershell
   npm run start:node
   ```

5. **Start explorer**:
   ```powershell
   npm run start:explorer
   ```

### CLI Transaction Builder

Use the node CLI to craft transactions with mandatory chat metadata and optional custodian-backed signing:

```powershell
# Transfer with metadata
aln tx transfer --from <addr> --to <addr> --amount 10 --nonce 0 \
   --chat-context-id 550e8400-e29b-41d4-a716-446655440000 \
   --transcript-hash <sha256> --jurisdiction-tags US_federal

# Governance vote
aln tx governance-vote --from <addr> --proposal gov_001 --support for --nonce 1 \
   --chat-context-id <uuid> --transcript-hash <sha256>

# Optional encrypted custodian signing
setx ALN_CUST_PASS "super-secret"
aln tx transfer ... --custodian-root ./.aln_keys --custodian-label wallet1 --custodian-passphrase-env ALN_CUST_PASS
```

`KeyCustodian` seals Ed25519 keys with AES-256-GCM + scrypt, so the CLI can sign without exposing raw private material. Metadata flags align with `AugmentedPolicyEngine.validateTransaction`, ensuring governance and migration ops are rejected if context headers are missing.

---

## Documentation

- **AUGMENTED_POLICY.md** – Full policy model, capability tiers, enforcement
- **AUGMENTED_REPUTATION.md** – Reputation formation, safeguards, thresholds
- **LAW_ENF_ASSIST_GUIDE.md** – Requirements, channels, task workflow
- **TREASURY.md** – Treasury address, governance, routing purposes
- **GOVERNANCE.md** – Proposal requirements, voting, execution
- **TPS_TARGETS.md** – Module-level throughput targets
- **SECURITY_TPS.md** – Security constraints and TPS engineering
- **docs/policy_mapping.md** – Mapping of ALN policies to runtime modules, CLI, and tests
- **docs/ci_checklist.md** – Canonical checklist for CI jobs covering governance, keys, and threat feeds

---

## Contributing

1. **Scaffold verification** – Run `npm run verify:scaffold` before commit
2. **Tests required** – All new features must include Jest tests
3. **No hard-coded values** – Import from `constants.ts`
4. **Audit transparency** – Emit events for state changes
5. **Human-rights compliance** – Design with appeals, transparency, fairness

---

## License

MIT
