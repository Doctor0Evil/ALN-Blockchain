# ALN Governance - Chat-First DAO Model

## Overview

ALN uses a **chat-native governance model** where all decisions originate in AI-chat sessions and are settled on-chain as high-TPS, auditable transactions. The blockchain acts as a **settlement and audit layer** over intelligent conversations, not as the primary interface for governance.

## Chat-First Architecture

### Core Principle

> **"Governance happens in conversations, settlements happen on-chain."**

Unlike traditional blockchains where governance requires manual on-chain voting through web interfaces, ALN governance:

1. **Starts in AI chat** - Proposals, discussions, and votes occur in natural language conversations
2. **Encoded as transactions** - Intent is extracted and encoded into deterministic on-chain operations
3. **Settled at high TPS** - Batched writes ensure low latency and high throughput
4. **Audited immutably** - Every settlement references its chat context for explainability

### Transaction Structure

Every governance transaction includes:

```typescript
interface GovernanceTransaction {
  // Standard transaction fields
  from: Address;
  to: Address;
  amount: bigint;
  nonce: number;
  
  // Chat-native governance fields
  chat_context_id: string;        // UUID linking to chat session
  transcript_hash: string;         // SHA-256 of conversation
  jurisdiction_tags: string[];     // Compliance routing tags
  safety_profile: SafetyProfile;   // BCI/nanoswarm/AI constraints
  
  // Treasury routing
  treasury_route?: {
    destination: typeof ALN_TREASURY_ADDRESS;
    purpose: 'governance_vote' | 'protocol_fee' | 'treasury_refill';
  };
}
```

## On-Chain Settlement Model

### High-TPS Pipeline

ALN's chat-native model requires **ultra-high transaction throughput**:

- **Baseline**: 10,000 TPS for chat-router module
- **Burst**: 100,000 TPS during high-volume governance events
- **Latency**: <100ms from chat intent to on-chain confirmation

### Settlement Flow

```
┌─────────────┐
│ AI Chat     │  1. User proposes governance action
│ Session     │     ("Transfer 1000 CHATAI to treasury for...")
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Intent      │  2. Extract structured intent
│ Extraction  │     - Action type, amounts, recipients
└──────┬──────┘     - Jurisdiction, safety constraints
       │
       ▼
┌─────────────┐
│ Policy      │  3. Validate against compliance rules
│ Validation  │     - Multi-jurisdictional routing
└──────┬──────┘     - Nanoswarm/BCI/AI safety checks
       │
       ▼
┌─────────────┐
│ Transaction │  4. Sign and encode as ALN transaction
│ Builder     │     - Include chat_context_id
└──────┬──────┘     - Include transcript_hash
       │
       ▼
┌─────────────┐
│ Settlement  │  5. Submit to ALN network
│ Node        │     - Batched writes, async API
└──────┬──────┘     - High-TPS settlement
       │
       ▼
┌─────────────┐
│ Blockchain  │  6. Immutable audit trail
│ Ledger      │     - Credits/debits ALN_TREASURY_ADDRESS
└─────────────┘     - Links to chat context forever
```

## Treasury Routing

All governance fees, votes, and refills conceptually route through:

```
ALN18sd2ujv24ual9c9pshtxys6j8knh6xaek9z83t
```

**Status**: Reserved placeholder (not yet live)

### Routing Rules

1. **Governance Votes** → Treasury address (recorded for vote weight)
2. **Protocol Fees** → Treasury address (community fund)
3. **Treasury Spends** ← Treasury address (DAO-approved transfers)
4. **Refills** → Treasury address (external contributions)

## CHATAI Token Governance

### Voting Power

Voting power is calculated as:

```
voting_power = token_balance + delegated_votes
```

### Proposal Types

ALN supports three governogram templates:

| Type | Quorum | Threshold | Examples |
|------|--------|-----------|----------|
| **Parameter Change** | 40% | 66% | Block time, gas limits, TPS targets |
| **Treasury Spend** | 50% | 75% | Grants, audits, incentives |
| **Contract Upgrade** | 60% | 80% | Protocol upgrades, new features |

### Vote Lifecycle

1. **Proposal in chat** - Natural language proposal with rationale
2. **Encoding** - Convert to governogram format with all metadata
3. **Submission** - Post to chain with `chat_context_id`
4. **Voting period** - Community votes via chat (1,000-100,000 blocks)
5. **Settlement** - Final tally recorded on-chain
6. **Execution** - Approved proposals execute automatically

## Auditability & Compliance

### Chat Context Preservation

Every governance transaction **MUST** include:

- `chat_context_id`: UUID of the chat session where decision originated
- `transcript_hash`: Cryptographic hash of relevant conversation excerpts
- `jurisdiction_tags`: Applicable regulatory regimes (e.g., US_federal, EU_GDPR)
- `safety_profile`: Constraints for nanoswarm, BCI, AI, and other future-tech

### Audit Trail Requirements

```typescript
interface AuditRow {
  transaction_hash: string;
  block_height: number;
  timestamp: number;
  
  // Governance-specific
  chat_context_id: string;
  transcript_hash: string;
  proposal_id?: string;
  vote_result?: 'pass' | 'fail';
  
  // Compliance
  jurisdictions: string[];
  safety_checks_passed: string[];
  policy_engine_version: string;
}
```

### Retention & Access

- **Minimum retention**: 7 years (financial regulations)
- **Access control**: Public for transparency, encrypted for privacy
- **Tamper-evidence**: Merkle-tree anchored, immutable append-only log

## Multi-Jurisdictional Routing

ALN governance supports **compliance routing** across jurisdictions:

- **US Federal** (JFMIP-24-01 internal controls)
- **EU** (GDPR data protection, MiCA regulations)
- **Cross-border** (Basel III, FATF recommendations)

### Routing Engine

```typescript
function routeGovernanceTransaction(tx: GovernanceTransaction): ComplianceDecision {
  const jurisdictions = identifyJurisdictions(tx.jurisdiction_tags);
  const policies = loadPolicies(jurisdictions);
  
  for (const policy of policies) {
    const result = policy.validate(tx);
    if (!result.compliant) {
      return { approved: false, reason: result.violations };
    }
  }
  
  return { 
    approved: true, 
    routing: determineOptimalRoute(jurisdictions, tx.safety_profile) 
  };
}
```

## Safety Constraints

### Nanoswarm Governance

- **Biohazard classification** must be declared
- **Material restrictions** enforced at protocol level
- **Telemetry requirements** for deployed swarms

### BCI/Neuromorphic Governance

- **Signal guardrails** block coercive or subliminal patterns
- **Dual human oversight** required for write operations
- **Audit stream retention** ≥90 days for medical compliance

### Superintelligence Governance

- **Capability ceilings** encoded as hard limits in language
- **Ethical compliance gates** can fatally halt unsafe proposals
- **Predictive risk analysis** via AI models before settlement

## Dev-Tunnel & Agent-to-Agent Governance

ALN supports **automated governance flows** from:

- AI agents inside dev-tunnels
- Agent-to-agent negotiation meshes
- Autonomous systems with ALN integration

### Agent Flow Example

```typescript
// AI agent proposes governance action from dev-tunnel
const proposal = {
  type: 'governance_vote',
  proposal_id: 'ALN-PROP-2025-042',
  vote: 'yes',
  agent_id: 'agent-7f3a9c',
  tunnel_id: 'tunnel-bce401',
  chat_context_id: generateUUID(),
  transcript_hash: hashAgentLogs(),
  safety_profile: {
    jurisdictions: ['US_federal'],
    constraints: ['no_bci_override', 'no_nanoswarm_deployment']
  }
};

// Policy engine validates
const validation = await policyEngine.validate(proposal);

if (validation.approved) {
  // Sign and dispatch through ALN async API
  const signedTx = await wallet.signTransaction(proposal);
  await alnClient.submitGovernanceVote(signedTx);
}
```

## Integration with Traditional DAOs

ALN governance can bridge to traditional on-chain DAOs:

1. **Chat-native proposal** created in ALN
2. **Cross-chain bridge** to target DAO (e.g., Snapshot, Aragon)
3. **Vote aggregation** from both ALN chat and traditional interface
4. **Settlement on ALN** with full audit trail
5. **Execution on target chain** if approved

## Future Enhancements

- **Multi-signature chat governance** - Threshold signatures from chat sessions
- **Quadratic voting** - Chat-native weighted voting
- **Conviction voting** - Time-weighted commitment from conversations
- **Futarchy** - Prediction markets integrated with chat

---

**Document Version**: 1.0  
**Last Updated**: November 27, 2025  
**Status**: Pre-mainnet specification  
**Related**: TREASURY.md, TPS_TARGETS.md, SECURITY_TPS.md
