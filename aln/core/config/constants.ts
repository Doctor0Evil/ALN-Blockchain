/**
 * ALN Blockchain - Canonical Constants
 * 
 * This module defines the single source of truth for all ALN protocol constants.
 * All contracts, SDKs, CLIs, and services MUST import from this file.
 * 
 * NEVER hard-code these values elsewhere in the codebase.
 */

// ============================================================================
// TREASURY ADDRESS (RESERVED, NOT YET LIVE)
// ============================================================================

/**
 * Canonical ALN treasury address for governance, fees, and refills.
 * 
 * STATUS: reserved_future_address
 * LIVE ON-CHAIN: false
 * 
 * This address is a PLACEHOLDER and reserved for future use until ALN mainnet
 * genesis is announced. Do NOT send funds to this address until official launch.
 * 
 * ROUTING USES:
 * - governance_votes: CHATAI token voting power and proposal fees
 * - protocol_fees: Transaction fees, bridge fees, contract deployment
 * - treasury_refills: Community contributions, grants, incentives
 * 
 * See: TREASURY.md for complete documentation
 */
export const ALN_TREASURY_ADDRESS = 'ALN18sd2ujv24ual9c9pshtxys6j8knh6xaek9z83t';

/**
 * Treasury status flag - set to true only after mainnet genesis
 */
export const ALN_TREASURY_LIVE = false;

// ============================================================================
// NETWORK CONFIGURATION
// ============================================================================

/**
 * Block time in milliseconds (5 seconds for solo consensus)
 */
export const BLOCK_TIME_MS = 5000;

/**
 * Maximum transactions per block
 */
export const MAX_TRANSACTIONS_PER_BLOCK = 1000;

/**
 * Gas limits
 */
export const GAS_LIMITS = {
  MIN: 21000,          // Minimum gas for simple transfer
  MAX: 10000000,       // Maximum gas per transaction
  BLOCK_LIMIT: 30000000 // Maximum gas per block
};

/**
 * Gas pricing (in smallest unit)
 */
export const GAS_PRICE = {
  DEFAULT: 100,
  MINIMUM: 1,
  MAXIMUM: 1000000
};

// ============================================================================
// GOVERNANCE PARAMETERS
// ============================================================================

/**
 * Governance voting periods (in blocks)
 */
export const GOVERNANCE_PERIODS = {
  MIN_VOTING: 1000,     // ~1.4 hours at 5s blocks
  MAX_VOTING: 100000,   // ~5.8 days
  DEFAULT_VOTING: 17280 // 1 day
};

/**
 * Proposal quorum and threshold requirements
 */
export const PROPOSAL_REQUIREMENTS = {
  PARAMETER_CHANGE: {
    quorum: 0.4,     // 40% of voting power must participate
    threshold: 0.66  // 66% approval required
  },
  TREASURY_SPEND: {
    quorum: 0.5,     // 50% participation
    threshold: 0.75  // 75% approval
  },
  CONTRACT_UPGRADE: {
    quorum: 0.6,     // 60% participation
    threshold: 0.8   // 80% approval
  }
};

// ============================================================================
// CHAT-NATIVE TRANSACTION FIELDS
// ============================================================================

/**
 * Maximum length for chat context ID (UUID format)
 */
export const MAX_CHAT_CONTEXT_ID_LENGTH = 36;

/**
 * Maximum length for transcript hash (SHA-256 hex)
 */
export const MAX_TRANSCRIPT_HASH_LENGTH = 64;

/**
 * Maximum number of jurisdiction tags per transaction
 */
export const MAX_JURISDICTION_TAGS = 10;

/**
 * Supported jurisdiction tags
 */
export const JURISDICTIONS = [
  'US_federal',
  'EU',
  'UK',
  'cross_border',
  'US_state_CA',
  'US_state_NY',
  'GDPR',
  'CCPA',
  'JFMIP'
];

// ============================================================================
// TPS TARGETS
// ============================================================================

/**
 * Target transactions per second by module
 * See: TPS_TARGETS.md for details
 */
export const TPS_TARGETS = {
  CHAT_ROUTER: {
    baseline: 10000,
    burst: 100000
  },
  WALLET: {
    baseline: 15000,
    burst: 150000
  },
  GOVERNANCE: {
    baseline: 5000,
    burst: 50000
  },
  AGENT: {
    baseline: 8000,
    burst: 80000
  },
  MIGRATION: {
    baseline: 2000,
    burst: 10000
  },
  TOTAL_NETWORK: {
    baseline: 200000,
    burst: 500000
  }
};

// ============================================================================
// SAFETY CONSTRAINTS
// ============================================================================

/**
 * QPU.Math+ safety thresholds
 */
export const SAFETY_LIMITS = {
  MAX_UINT64: 18446744073709551615n,
  MIN_AMOUNT: 0n,
  MAX_TRANSFER_AMOUNT: 1000000000000000000n, // 1B tokens max per tx
  MAX_DELEGATION: 10000000000000000000n      // 10B tokens max delegation
};

/**
 * Nanoswarm safety classifications
 */
export const NANOSWARM_CLASSES = [
  'class_1_minimal',
  'class_2_controlled',
  'class_3_contained',
  'class_4_restricted'
];

/**
 * BCI safety levels
 */
export const BCI_SAFETY_LEVELS = [
  'read_only',
  'write_supervised',
  'write_emergency'
];

// ============================================================================
// AUDIT & COMPLIANCE
// ============================================================================

/**
 * Audit log retention period (in milliseconds)
 * 7 years = 220752000000 ms
 */
export const AUDIT_RETENTION_MS = 220752000000;

/**
 * Compliance policy versions
 */
export const POLICY_VERSIONS = {
  JFMIP: '24-01',
  GDPR: '2016/679',
  MICA: '2023/1114',
  FATF: 'R16-2019'
};

// ============================================================================
// NETWORK ENDPOINTS
// ============================================================================

/**
 * Default API ports
 */
export const PORTS = {
  HTTP_API: 3000,
  WEBSOCKET: 3001,
  EXPLORER: 8080,
  METRICS: 9090
};

/**
 * API rate limits (requests per minute)
 */
export const RATE_LIMITS = {
  PUBLIC_API: 1000,
  AUTHENTICATED_API: 10000,
  AGENT_API: 50000,
  WEBSOCKET_SUBSCRIPTIONS: 100
};

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/**
 * Check if treasury is live and ready for transactions
 */
export function isTreasuryLive(): boolean {
  return ALN_TREASURY_LIVE;
}

/**
 * Validate treasury address format
 */
export function isValidTreasuryAddress(address: string): boolean {
  return address === ALN_TREASURY_ADDRESS;
}

/**
 * Validate jurisdiction tag
 */
export function isValidJurisdiction(tag: string): boolean {
  return JURISDICTIONS.includes(tag);
}

/**
 * Get treasury routing purpose enum
 */
export enum TreasuryRoutingPurpose {
  GOVERNANCE_VOTE = 'governance_vote',
  PROTOCOL_FEE = 'protocol_fee',
  TREASURY_REFILL = 'treasury_refill'
}

// ============================================================================
// EXPORTS
// ============================================================================

export default {
  ALN_TREASURY_ADDRESS,
  ALN_TREASURY_LIVE,
  BLOCK_TIME_MS,
  MAX_TRANSACTIONS_PER_BLOCK,
  GAS_LIMITS,
  GAS_PRICE,
  GOVERNANCE_PERIODS,
  PROPOSAL_REQUIREMENTS,
  MAX_CHAT_CONTEXT_ID_LENGTH,
  MAX_TRANSCRIPT_HASH_LENGTH,
  MAX_JURISDICTION_TAGS,
  JURISDICTIONS,
  TPS_TARGETS,
  SAFETY_LIMITS,
  NANOSWARM_CLASSES,
  BCI_SAFETY_LEVELS,
  AUDIT_RETENTION_MS,
  POLICY_VERSIONS,
  PORTS,
  RATE_LIMITS,
  isTreasuryLive,
  isValidTreasuryAddress,
  isValidJurisdiction,
  TreasuryRoutingPurpose
};
