# ALN Security & TPS - Protecting High-Throughput Operations

## Overview

ALN achieves **ultra-high transaction throughput** (200,000+ TPS) while maintaining **defense-in-depth security**. This document defines mandatory security controls for every high-TPS component and maps them to relevant standards.

## Core Security Principle

> **"Throughput is achieved through optimization, not by bypassing security."**

Every transaction at every TPS level must satisfy:
1. **Authentication & Authorization** - Cryptographic identity verification
2. **Compliance Routing** - Multi-jurisdictional policy validation
3. **Immutable Audit Logging** - Tamper-evident transaction records
4. **Chat Context Preservation** - Explainability and legal review support

## Security Requirements by Component

### 1. Chat Router Module (70,000 TPS)

**Threat Model**:
- Malicious chat sessions attempting unauthorized transactions
- Replay attacks from cached chat contexts
- Injection attacks through transcript manipulation

**Mandatory Controls**:

| Control | Implementation | Standard Mapping |
|---------|---------------|-----------------|
| **Encryption in Transit** | TLS 1.3 for all chat-to-chain communication | NIST SP 800-52r2 |
| **RBAC** | Role-based access control per chat session with scoped permissions | NIST RBAC (ANSI INCITS 359-2012) |
| **Anomaly Detection** | ML-based pattern analysis for unusual transaction sequences | JFMIP-24-01 §4.2 |
| **Audit Logging** | Immutable append-only log linking tx_hash → chat_context_id → transcript_hash | JFMIP-24-01 §5.1 |

**Implementation**:

```typescript
class ChatRouterSecurity {
  async validateChatTransaction(tx: ChatTransaction): Promise<ValidationResult> {
    // 1. Verify TLS connection
    if (!tx.connection.isTLS13) {
      throw new SecurityError('ALN_ERR_INSECURE_CONNECTION');
    }
    
    // 2. Check RBAC permissions
    const permissions = await rbac.getPermissions(tx.chat_context_id);
    if (!permissions.includes(tx.action_type)) {
      throw new SecurityError('ALN_ERR_INSUFFICIENT_PERMISSIONS');
    }
    
    // 3. Anomaly detection
    const isAnomalous = await anomalyDetector.analyze(tx, {
      window: '5m',
      threshold: 3.0 // standard deviations
    });
    if (isAnomalous) {
      await securityLog.flagSuspicious(tx);
      // Continue but mark for review
    }
    
    // 4. Audit log entry
    await auditLog.append({
      transaction_hash: tx.hash(),
      chat_context_id: tx.chat_context_id,
      transcript_hash: tx.transcript_hash,
      timestamp: Date.now(),
      permissions_checked: permissions,
      anomaly_score: isAnomalous ? anomalyDetector.lastScore : 0
    });
    
    return { valid: true };
  }
}
```

### 2. Wallet Transaction Module (60,000 TPS)

**Threat Model**:
- Private key compromise
- Transaction replay attacks
- Front-running and MEV attacks

**Mandatory Controls**:

| Control | Implementation | Standard Mapping |
|---------|---------------|-----------------|
| **Encryption at Rest** | AES-256-GCM for private keys (browser-only, non-custodial) | FIPS 197, NIST SP 800-38D |
| **Nonce Management** | Strictly incrementing nonce with gap detection | Ethereum EIP-155 |
| **Signature Verification** | Ed25519 batch verification with parallel execution | RFC 8032 |
| **Audit Logging** | Per-transaction immutable record with sender, receiver, amount, nonce | JFMIP-24-01 §5.1 |

**Implementation**:

```typescript
class WalletTransactionSecurity {
  async verifyAndLog(tx: WalletTransaction): Promise<void> {
    // 1. Verify Ed25519 signature
    const isValidSig = await ed25519.verify(
      tx.signature,
      tx.serializeForSigning(),
      tx.from_public_key
    );
    if (!isValidSig) {
      throw new SecurityError('ALN_ERR_INVALID_SIGNATURE');
    }
    
    // 2. Check nonce ordering
    const expectedNonce = await stateStore.getAccountNonce(tx.from);
    if (tx.nonce !== expectedNonce) {
      throw new SecurityError('ALN_ERR_NONCE_MISMATCH');
    }
    
    // 3. Immutable audit log
    await auditLog.append({
      transaction_hash: tx.hash(),
      block_height: currentBlockHeight,
      from: tx.from,
      to: tx.to,
      amount: tx.amount.toString(),
      nonce: tx.nonce,
      gas_limit: tx.gas_limit,
      gas_price: tx.gas_price,
      signature: tx.signature,
      verified_at: Date.now()
    });
  }
}
```

### 3. Governance Module (30,000 TPS)

**Threat Model**:
- Vote manipulation and double-voting
- Governance attack via flash loans or vote buying
- Proposal spam and DoS

**Mandatory Controls**:

| Control | Implementation | Standard Mapping |
|---------|---------------|-----------------|
| **Vote Authentication** | Snapshot-based voting power at proposal creation block | ERC-20 Snapshot pattern |
| **Double-Vote Prevention** | Merkle accumulator tracking all votes with duplicate detection | JFMIP-24-01 §3.4 |
| **Rate Limiting** | Max 10 proposals per address per day, 100 votes per minute | OWASP API Security |
| **Audit Logging** | Full vote history with chat_context_id, transcript_hash, voting_power | JFMIP-24-01 §5.1 |

**Implementation**:

```typescript
class GovernanceSecurity {
  async validateVote(vote: GovernanceVote): Promise<void> {
    // 1. Snapshot-based voting power
    const votingPower = await governance.getVotingPowerAtSnapshot(
      vote.voter,
      vote.proposal_snapshot_block
    );
    if (votingPower === 0n) {
      throw new SecurityError('ALN_ERR_NO_VOTING_POWER');
    }
    
    // 2. Double-vote detection
    const hasVoted = await governance.hasVoted(vote.proposal_id, vote.voter);
    if (hasVoted) {
      throw new SecurityError('ALN_ERR_DOUBLE_VOTE');
    }
    
    // 3. Rate limiting
    const recentVotes = await governance.countRecentVotes(vote.voter, '1m');
    if (recentVotes >= 100) {
      throw new SecurityError('ALN_ERR_RATE_LIMIT_EXCEEDED');
    }
    
    // 4. Audit with chat context
    await auditLog.append({
      transaction_hash: vote.hash(),
      proposal_id: vote.proposal_id,
      voter: vote.voter,
      choice: vote.choice,
      voting_power: votingPower.toString(),
      chat_context_id: vote.chat_context_id,
      transcript_hash: vote.transcript_hash,
      timestamp: Date.now()
    });
  }
}
```

### Threat Intelligence Layer

**Objective**: Keep malware detection and compliance guardrails synchronized with external intel without blocking block production.

| Control | Implementation | Standard Mapping |
|---------|----------------|-----------------|
| **Feed Aggregation** | `ThreatFeedIngestor` loads signed JSON feeds (file/HTTPS) and hydrates `MLThreatHooks` | JFMIP-24-01 §4.2 |
| **Fallback Safety** | If feeds unavailable, revert to embedded signatures to avoid blind spots | ISO/IEC 27001 A.12.6 |
| **Update Propagation** | `ml_hooks.updatePolicies()` refreshes token pipeline + migration scanners before scoring payloads | CSA CCM IVS-09 |

```javascript
async function refreshThreatIntel() {
  const feed = new ThreatFeedIngestor({
    sources: [
      { type: 'https', url: 'https://intel.aln/payloads.json' },
      { type: 'file', path: path.join(__dirname, 'threat_feed_sample.json') }
    ]
  });

  const mlHooks = new MLThreatHooks(feed);
  await mlHooks.updatePolicies();
}
```

### Custodian Key Management

**Objective**: Ensure validator and wallet signing keys remain encrypted-at-rest and never leave secure hardware.

- `KeyCustodian` wraps AES-256-GCM + `scrypt` to seal Ed25519 keys under operator passphrases or enclave secrets
- CLI flags `--custodian-root`, `--custodian-label`, `--custodian-passphrase-env` auto-create/load sealed keys before signing
- Future HSM integration: replace local storage adapters while keeping same `signDigest()` interface

### 4. Dev-Tunnel Agent Module (20,000 TPS)

**Threat Model**:
- Rogue agents submitting malicious transactions
- Agent impersonation and credential theft
- Automated attack scripts abusing agent APIs

**Mandatory Controls**:

| Control | Implementation | Standard Mapping |
|---------|---------------|-----------------|
| **Agent Authentication** | Mutual TLS with client certificates for agent identity | RFC 8446, mTLS |
| **Capability Limits** | Per-agent transaction quotas and allowed operation whitelist | OWASP ASVS 4.0 §4.2 |
| **Behavioral Analysis** | Real-time monitoring of agent transaction patterns | JFMIP-24-01 §4.2 |
| **Audit Logging** | Agent ID, tunnel ID, transaction metadata in immutable log | JFMIP-24-01 §5.1 |

**Implementation**:

```typescript
class AgentSecurity {
  async authorizeAgentTransaction(tx: AgentTransaction): Promise<void> {
    // 1. Mutual TLS verification
    const agentCert = tx.connection.getPeerCertificate();
    if (!agentCert || !this.isValidAgentCert(agentCert)) {
      throw new SecurityError('ALN_ERR_INVALID_AGENT_CERT');
    }
    
    // 2. Capability check
    const capabilities = await agentRegistry.getCapabilities(tx.agent_id);
    if (!capabilities.operations.includes(tx.operation)) {
      throw new SecurityError('ALN_ERR_UNAUTHORIZED_OPERATION');
    }
    
    // 3. Quota enforcement
    const usage = await agentRegistry.getQuotaUsage(tx.agent_id, '1h');
    if (usage.transaction_count >= capabilities.quota.hourly_transactions) {
      throw new SecurityError('ALN_ERR_QUOTA_EXCEEDED');
    }
    
    // 4. Behavioral monitoring
    const isSuspicious = await behaviorAnalyzer.analyze(tx);
    if (isSuspicious) {
      await securityLog.flagAgent(tx.agent_id, 'suspicious_behavior');
    }
    
    // 5. Audit log
    await auditLog.append({
      transaction_hash: tx.hash(),
      agent_id: tx.agent_id,
      tunnel_id: tx.tunnel_id,
      operation: tx.operation,
      cert_fingerprint: this.getCertFingerprint(agentCert),
      behavioral_score: behaviorAnalyzer.lastScore,
      timestamp: Date.now()
    });
  }
}
```

### 5. Migration Bridge Module (10,000 TPS)

**Threat Model**:
- Double-spend attacks across chains
- Fake proof submission
- Replay attacks of old migrations

**Mandatory Controls**:

| Control | Implementation | Standard Mapping |
|---------|---------------|-----------------|
| **Proof Verification** | Merkle inclusion proof validation against source chain state | Ethereum Light Client Spec |
| **Replay Protection** | Nonce tracking per source transaction with uniqueness checks | EIP-155 |
| **Threshold Signatures** | Multi-sig validation from trusted bridge validators (3-of-5) | BIP-174 (PSBT) |
| **Audit Logging** | Full migration trail: source_tx_hash, proof_hash, mint_tx_hash | JFMIP-24-01 §5.1 |

**Implementation**:

```typescript
class BridgeSecurity {
  async validateMigration(migration: Migration): Promise<void> {
    // 1. Merkle proof verification
    const isValidProof = await merkleVerifier.verify(
      migration.proof,
      migration.source_tx_hash,
      migration.source_chain_state_root
    );
    if (!isValidProof) {
      throw new SecurityError('ALN_ERR_INVALID_PROOF');
    }
    
    // 2. Replay protection
    const hasBeenProcessed = await bridgeState.hasProcessed(migration.source_tx_hash);
    if (hasBeenProcessed) {
      throw new SecurityError('ALN_ERR_REPLAY_DETECTED');
    }
    
    // 3. Multi-sig validation
    const validSigs = await multisig.countValidSignatures(migration.validator_signatures);
    if (validSigs < 3) {
      throw new SecurityError('ALN_ERR_INSUFFICIENT_SIGNATURES');
    }
    
    // 4. Audit log
    await auditLog.append({
      migration_id: migration.id,
      source_tx_hash: migration.source_tx_hash,
      source_chain: migration.source_chain,
      proof_hash: migration.proof_hash,
      amount: migration.amount.toString(),
      validator_signatures: migration.validator_signatures.length,
      status: 'verified',
      timestamp: Date.now()
    });
  }
}
```

## Standards Compliance Mapping

### JFMIP-24-01 (Federal Financial Systems)

| Section | Requirement | ALN Implementation |
|---------|------------|-------------------|
| §3.4 | Internal controls for transaction authorization | RBAC, multi-sig, capability limits |
| §4.2 | Anomaly detection and fraud prevention | ML-based pattern analysis, behavioral monitoring |
| §5.1 | Immutable audit trails with tamper-evidence | Merkle-tree-anchored append-only logs |
| §6.3 | Encryption in transit and at rest | TLS 1.3, AES-256-GCM, Ed25519 signatures |

### Multi-Jurisdictional Financial Regulations

| Jurisdiction | Regulation | ALN Compliance |
|-------------|-----------|----------------|
| **US Federal** | Bank Secrecy Act (BSA), AML | Transaction monitoring, suspicious activity flagging |
| **EU** | GDPR | Privacy mode with encrypted audit hashes, right to erasure compliance |
| **EU** | MiCA (Markets in Crypto-Assets) | Full audit trails, governance transparency, investor protection |
| **Global** | FATF Recommendations | KYC/AML integration points, travel rule support for large transfers |

## Transaction-Level Audit Requirements

Every transaction in ALN **MUST** generate an immutable audit row:

```typescript
interface AuditRow {
  // Core transaction data
  transaction_hash: string;
  block_height: number;
  timestamp: number;
  
  // Chat-native context
  chat_context_id: string;
  transcript_hash: string;
  
  // Compliance metadata
  jurisdictions: string[];
  safety_checks_passed: string[];
  policy_engine_version: string;
  
  // Security metadata
  authentication_method: 'ed25519' | 'mtls' | 'rbac';
  permissions_verified: string[];
  anomaly_score: number;
  rate_limit_state: RateLimitState;
  
  // Auditability
  log_version: string;
  log_signature: string; // Signed by audit log service
}
```

### Audit Log Properties

- **Immutability**: Append-only, cryptographically signed
- **Tamper-Evidence**: Merkle tree with periodic root anchoring
- **Retention**: Minimum 7 years for financial compliance
- **Privacy**: Supports encrypted fields with hash-based indexing
- **Query Performance**: Indexed by transaction_hash, chat_context_id, timestamp

## Security at Maximum TPS

### What Remains Mandatory at 200,000+ TPS

✅ **NEVER Bypassed**:
1. Cryptographic signature verification (all 200k transactions)
2. Audit log write (one immutable row per transaction)
3. Compliance routing check (multi-jurisdictional policy validation)
4. Chat context preservation (chat_context_id + transcript_hash)

✅ **Always Enforced**:
- Nonce ordering and replay protection
- Rate limiting per address/agent
- RBAC permission checks
- Anomaly detection scoring

### Performance Optimizations (Security-Preserving)

✅ **Allowed**:
- Batch signature verification (Ed25519 supports efficient batching)
- Parallel policy validation (independent transaction checks)
- Async audit log writes (with write-ahead logging for durability)
- Cached RBAC lookups (with TTL expiration)

❌ **NEVER Allowed**:
- Skipping signature verification
- Disabling audit logs
- Bypassing compliance routing
- Removing chat context metadata

## Incident Response at High TPS

### Detection

```typescript
class SecurityMonitor {
  async monitorHighTPSThreats(): Promise<void> {
    const metrics = await tpsMonitor.getCurrentMetrics();
    
    // 1. Signature verification failure spike
    if (metrics.sig_verification_failure_rate > 0.01) {
      await incident.trigger('CRYPTO_ATTACK_SUSPECTED');
    }
    
    // 2. Anomaly detection alerts
    if (metrics.anomaly_alerts_per_second > 100) {
      await incident.trigger('ABNORMAL_TRANSACTION_PATTERN');
    }
    
    // 3. Compliance routing failures
    if (metrics.compliance_rejection_rate > 0.05) {
      await incident.trigger('JURISDICTIONAL_POLICY_VIOLATION');
    }
    
    // 4. Audit log write failures
    if (metrics.audit_log_write_failure_rate > 0.0001) {
      await incident.trigger('AUDIT_INTEGRITY_THREAT');
      await emergencyShutdown.triggerSafety();
    }
  }
}
```

### Response Playbook

| Threat Level | Indicator | Automated Response |
|-------------|-----------|-------------------|
| 🟢 **Info** | Anomaly score >2σ | Log and continue |
| 🟡 **Warning** | Sig failure rate >0.5% | Rate limit affected addresses |
| 🟠 **High** | Compliance failure >5% | Pause affected jurisdiction routing |
| 🔴 **Critical** | Audit log failures >0.01% | Emergency shutdown, manual recovery |

---

**Document Version**: 1.0  
**Last Updated**: November 27, 2025  
**Status**: Pre-mainnet specification  
**Related**: TPS_TARGETS.md, GOVERNANCE.md, TREASURY.md  
**Compliance**: JFMIP-24-01, GDPR, MiCA, Bank Secrecy Act, FATF
