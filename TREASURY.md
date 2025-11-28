# ALN Treasury - Canonical Address & Routing Rules

## Overview

The ALN blockchain uses a single, canonical treasury address for all protocol-level governance, fee routing, and treasury refills. This document establishes the **official treasury address** and explains its role in the ALN ecosystem.

## Canonical Address (Reserved)

```
ALN18sd2ujv24ual9c9pshtxys6j8knh6xaek9z83t
```

**Status**: `reserved_future_address`  
**Live on-chain**: ❌ **NOT YET LIVE**  
**Purpose**: Designated governance + fee-routing treasury for ALN

### Important Disclaimer

⚠️ **DO NOT SEND FUNDS TO THIS ADDRESS**

This address is a **placeholder** and **reserved for future use**. It is treated as a documentation and code constant until ALN mainnet genesis is announced. No on-chain funds should be sent to this address until the official mainnet launch.

## Usage: Fees, Votes, and Refills

The ALN treasury address serves three primary functions:

### 1. **Protocol Fees**
- Transaction fees collected from all on-chain operations
- Smart contract deployment fees
- High-frequency transaction batching fees
- Cross-chain bridge fees (e.g., Canto→ALN migration)

### 2. **Governance Votes**
- CHATAI token voting power routing
- DAO proposal execution costs
- Governance parameter change fees
- Treasury spend authorization

### 3. **Treasury Refills**
- Community incentive pools
- Developer grants and bounties
- Security audits and bug bounties
- Ecosystem development funds

## Integration Guidelines

### For Developers

All code that interacts with fees, governance, or treasury routing **MUST**:

1. **Import from canonical config** - Never hard-code the address
2. **Use the constant name** - `ALN_TREASURY_ADDRESS` across all codebases
3. **Check live status** - Validate that mainnet is live before sending real funds
4. **Include audit metadata** - Attach `chat_context_id` and `transcript_hash` to all treasury transactions

### Example Integration

```typescript
// ✅ CORRECT - Import from config
import { ALN_TREASURY_ADDRESS } from './config/constants';

function routeFees(amount: bigint): Transaction {
  return {
    to: ALN_TREASURY_ADDRESS,
    amount,
    chat_context_id: getCurrentChatContext(),
    transcript_hash: hashTranscript()
  };
}

// ❌ WRONG - Hard-coded address
const treasury = "ALN18sd2ujv24ual9c9pshtxys6j8knh6xaek9z83t"; // DON'T DO THIS
```

## Design Precedent

ALN's treasury model follows the **NEAR Intents** design pattern for transparent, auditable treasury routing:

- All flows are deterministic and policy-governed
- Multi-jurisdictional compliance routing is enforced
- Immutable audit trails link every transaction to its chat context
- Treasury operations are subject to DAO governance votes

Reference: [NEAR Protocol Treasury Documentation](https://docs.near.org/concepts/protocol/transaction-execution)

## Migration Plan

Once ALN mainnet is live:

1. **Genesis Block** will instantiate the treasury account
2. **DAO Proposal** will authorize initial treasury parameters
3. **Audit & Verification** will confirm the address matches this specification
4. **Public Announcement** will declare the treasury live and ready
5. **Integration Updates** will activate fee routing and governance flows

## Not-Yet-Live Checklist

- [ ] ALN mainnet genesis completed
- [ ] Treasury account instantiated on-chain
- [ ] DAO vote approved treasury parameters
- [ ] Security audit completed and published
- [ ] Official announcement from ALN core team
- [ ] SDK and node software updated to production mode

## Contact & Governance

For questions about treasury operations:
- **GitHub**: Open an issue in the ALN repository
- **Governance**: Submit proposals via CHATAI DAO
- **Security**: Report vulnerabilities via responsible disclosure

---

**Document Version**: 1.0  
**Last Updated**: November 27, 2025  
**Status**: Pre-mainnet specification  
**Compliance**: JFMIP-24-01, Multi-jurisdictional financial regulations
