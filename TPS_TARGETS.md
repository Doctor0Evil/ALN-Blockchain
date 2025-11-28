# ALN TPS Targets - Performance Requirements

## Overview

ALN is designed for **ultra-high transaction throughput** to support AI-chat-native governance, dev-tunnel automation, and real-time agent-to-agent transactions. This document defines non-functional TPS requirements that constrain the system architecture.

## Core Principle

> **"Throughput may NEVER be increased by skipping authentication, logging, or compliance routing."**

Every transaction, regardless of volume, must:
- ✅ Pass authentication and authorization checks
- ✅ Be logged to immutable audit trail
- ✅ Route through compliance policy engine
- ✅ Include chat context and jurisdiction metadata

## TPS Targets by Module

### Chat Router Module

**Purpose**: Route transactions from AI chat sessions to blockchain settlement

| Metric | Target |
|--------|--------|
| **Baseline TPS** | 10,000 transactions/second |
| **Burst TPS** | 100,000 transactions/second |
| **Latency (p50)** | <50ms from chat intent to settlement |
| **Latency (p99)** | <200ms |
| **Batch Size** | 100-1,000 transactions per batch |

**Techniques**:
- In-memory transaction queues with async I/O
- Batched writes to state store (LevelDB)
- Parallel signature verification
- Pipelined transaction validation

### Governance Module

**Purpose**: Process votes, proposals, and DAO operations

| Metric | Target |
|--------|--------|
| **Baseline TPS** | 5,000 transactions/second |
| **Burst TPS** | 50,000 transactions/second |
| **Vote Processing** | 10,000 votes/second during governance events |
| **Proposal Submission** | 100 proposals/second |

**Techniques**:
- Merkle-tree vote aggregation
- Snapshot-based voting to reduce state reads
- Cached voting power calculations
- Async proposal validation pipeline

### Dev-Tunnel Agent Module

**Purpose**: Automated transactions from AI agents in dev-tunnels

| Metric | Target |
|--------|--------|
| **Baseline TPS** | 8,000 transactions/second |
| **Burst TPS** | 80,000 transactions/second |
| **Agent-to-Agent** | 12,000 direct agent transactions/second |
| **Tunnel-to-Chain** | <30ms average latency |

**Techniques**:
- Dedicated agent transaction pools
- Precomputed agent signatures (Ed25519)
- Fast-path validation for trusted agent IDs
- Async dispatch to settlement nodes

### Migration Bridge Module

**Purpose**: Cross-chain asset migrations (e.g., Canto→ALN)

| Metric | Target |
|--------|--------|
| **Baseline TPS** | 2,000 migrations/second |
| **Burst TPS** | 10,000 migrations/second |
| **Verification Time** | <1 second per migration |
| **Finality** | 2 blocks (~10 seconds) |

**Techniques**:
- Parallel proof verification
- Merkle inclusion proofs for source chain
- Batched mint/burn operations
- Optimistic verification with fraud proofs

### Wallet Transaction Module

**Purpose**: User-initiated transfers and contract calls

| Metric | Target |
|--------|--------|
| **Baseline TPS** | 15,000 transactions/second |
| **Burst TPS** | 150,000 transactions/second |
| **Signature Verification** | 20,000 signatures/second |
| **State Update** | <10ms per account state write |

**Techniques**:
- Parallel signature verification pools
- Account-level state sharding
- Write-ahead logging for durability
- In-memory cache for hot accounts

### Explorer & Analytics Module

**Purpose**: Real-time blockchain data queries and visualization

| Metric | Target |
|--------|--------|
| **Query TPS** | 50,000 queries/second |
| **WebSocket Events** | 100,000 events/second |
| **Block Indexing** | <100ms per block |
| **Historical Queries** | <500ms for 1M block range |

**Techniques**:
- Read replicas for explorer queries
- Materialized views for common aggregations
- WebSocket connection pooling
- GraphQL subscription multiplexing

## System-Wide TPS Budget

### Aggregate Target

**Total Network Capacity**: **200,000+ TPS** sustained

| Component | % of Total | TPS Allocation |
|-----------|-----------|----------------|
| Chat Router | 35% | 70,000 TPS |
| Wallet Transactions | 30% | 60,000 TPS |
| Governance | 15% | 30,000 TPS |
| Dev-Tunnel Agents | 10% | 20,000 TPS |
| Migration Bridge | 5% | 10,000 TPS |
| Other | 5% | 10,000 TPS |

### Burst Capacity

During high-traffic events (e.g., major governance votes, airdrops):
- **Burst Mode**: 500,000+ TPS for short durations (<60 seconds)
- **Sustained Burst**: 300,000 TPS for up to 5 minutes
- **Graceful Degradation**: Non-critical operations throttled to maintain critical paths

## Performance Techniques

### In-Memory Batching

```typescript
class TransactionBatcher {
  private queue: Transaction[] = [];
  private readonly BATCH_SIZE = 1000;
  private readonly BATCH_TIMEOUT_MS = 50;

  async addTransaction(tx: Transaction): Promise<void> {
    this.queue.push(tx);
    
    if (this.queue.length >= this.BATCH_SIZE) {
      await this.flush();
    }
  }

  private async flush(): Promise<void> {
    if (this.queue.length === 0) return;
    
    const batch = this.queue.splice(0, this.BATCH_SIZE);
    
    // Parallel processing
    await Promise.all([
      this.validateBatch(batch),
      this.verifySignatures(batch),
      this.checkCompliance(batch)
    ]);
    
    await this.commitToState(batch);
    await this.emitAuditLogs(batch);
  }
}
```

### Parallel Execution

```typescript
async function processTransactionBatch(txs: Transaction[]): Promise<void> {
  // 1. Parallel signature verification
  const sigPromises = txs.map(tx => verifySignature(tx));
  await Promise.all(sigPromises);
  
  // 2. Parallel policy validation
  const policyPromises = txs.map(tx => validateCompliance(tx));
  await Promise.all(policyPromises);
  
  // 3. Sequential state updates (for consistency)
  for (const tx of txs) {
    await stateStore.applyTransaction(tx);
  }
  
  // 4. Parallel audit logging
  const auditPromises = txs.map(tx => auditLog.append(tx));
  await Promise.all(auditPromises);
}
```

### Async APIs

```typescript
// High-throughput async submission
async function submitTransactionAsync(tx: Transaction): Promise<TransactionReceipt> {
  // Non-blocking submission
  const receiptPromise = transactionPool.submit(tx);
  
  // Return immediately with pending receipt
  return {
    transaction_hash: tx.hash(),
    status: 'pending',
    submitted_at: Date.now()
  };
}

// Background processing
setInterval(async () => {
  const batch = await transactionPool.getBatch(1000);
  await processTransactionBatch(batch);
}, 50); // 50ms batch interval = 20 batches/second
```

### Low-Latency Pipelines

```
┌─────────────┐
│ Receive TX  │  <1ms: Parse and validate format
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Verify Sig  │  5-10ms: Ed25519 signature verification
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Policy Check│  10-20ms: Compliance routing validation
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ State Update│  5-15ms: Apply to LevelDB with batching
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Audit Log   │  5ms: Append to immutable log
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Broadcast   │  <5ms: Notify WebSocket subscribers
└─────────────┘

Total: <50ms end-to-end
```

## Monitoring & Observability

### Key Metrics

```typescript
interface TPSMetrics {
  // Throughput
  current_tps: number;
  peak_tps_1min: number;
  peak_tps_1hour: number;
  
  // Latency
  latency_p50_ms: number;
  latency_p95_ms: number;
  latency_p99_ms: number;
  
  // Health
  error_rate: number;
  queue_depth: number;
  cpu_utilization: number;
  memory_utilization: number;
  
  // Safety
  compliance_checks_passed: number;
  compliance_checks_failed: number;
  audit_logs_written: number;
}
```

### Alerts

- 🔴 **Critical**: TPS drops below 50% of baseline
- 🟠 **Warning**: Latency p99 exceeds 500ms
- 🟡 **Info**: Queue depth exceeds 10,000 transactions
- 🚨 **Fatal**: Compliance check failure rate >0.1%

## Constraints & Guarantees

### What We NEVER Sacrifice

Even at maximum TPS:
- ✅ Every transaction has an audit log entry
- ✅ Every transaction passes compliance routing
- ✅ Every transaction includes chat_context_id and transcript_hash
- ✅ Every signature is cryptographically verified
- ✅ Every state transition is deterministic and reproducible

### What We MAY Throttle

During extreme load:
- ⏸️ Non-critical explorer queries
- ⏸️ Historical data exports
- ⏸️ Analytics dashboard updates
- ⏸️ Low-priority WebSocket subscriptions

### What We NEVER Throttle

- 🔒 Authentication and authorization
- 🔒 Compliance policy validation
- 🔒 Audit log writes
- 🔒 Critical governance votes
- 🔒 Emergency safety operations

## Benchmarking

### Test Scenarios

1. **Sustained Load** - 10,000 TPS for 1 hour
2. **Burst Load** - 100,000 TPS for 60 seconds
3. **Mixed Workload** - 70% transfers, 20% governance, 10% bridge
4. **Agent Swarm** - 50,000 agent-to-agent transactions/second
5. **Governance Event** - 30,000 simultaneous votes

### Success Criteria

- ✅ Zero authentication bypasses
- ✅ Zero compliance check skips
- ✅ 100% audit log coverage
- ✅ <1% transaction failures
- ✅ Latency p99 <200ms

---

**Document Version**: 1.0  
**Last Updated**: November 27, 2025  
**Status**: Pre-mainnet specification  
**Related**: SECURITY_TPS.md, GOVERNANCE.md, TREASURY.md
