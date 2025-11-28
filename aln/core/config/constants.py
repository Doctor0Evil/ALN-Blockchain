"""
ALN Blockchain - Canonical Constants (Python SDK)

This module defines the single source of truth for all ALN protocol constants.
All Python SDKs, clients, and tools MUST import from this file.

NEVER hard-code these values elsewhere in the codebase.
"""

from enum import Enum
from typing import List

# ============================================================================
# TREASURY ADDRESS (RESERVED, NOT YET LIVE)
# ============================================================================

ALN_TREASURY_ADDRESS = "ALN18sd2ujv24ual9c9pshtxys6j8knh6xaek9z83t"
"""
Canonical ALN treasury address for governance, fees, and refills.

STATUS: reserved_future_address
LIVE ON-CHAIN: False

This address is a PLACEHOLDER and reserved for future use until ALN mainnet
genesis is announced. Do NOT send funds to this address until official launch.

ROUTING USES:
- governance_votes: CHATAI token voting power and proposal fees
- protocol_fees: Transaction fees, bridge fees, contract deployment
- treasury_refills: Community contributions, grants, incentives

See: TREASURY.md for complete documentation
"""

ALN_TREASURY_LIVE = False
"""Treasury status flag - set to True only after mainnet genesis"""

# ============================================================================
# NETWORK CONFIGURATION
# ============================================================================

BLOCK_TIME_MS = 5000
"""Block time in milliseconds (5 seconds for solo consensus)"""

MAX_TRANSACTIONS_PER_BLOCK = 1000
"""Maximum transactions per block"""

GAS_LIMITS = {
    "MIN": 21000,
    "MAX": 10000000,
    "BLOCK_LIMIT": 30000000
}

GAS_PRICE = {
    "DEFAULT": 100,
    "MINIMUM": 1,
    "MAXIMUM": 1000000
}

# ============================================================================
# GOVERNANCE PARAMETERS
# ============================================================================

GOVERNANCE_PERIODS = {
    "MIN_VOTING": 1000,
    "MAX_VOTING": 100000,
    "DEFAULT_VOTING": 17280
}

PROPOSAL_REQUIREMENTS = {
    "PARAMETER_CHANGE": {"quorum": 0.4, "threshold": 0.66},
    "TREASURY_SPEND": {"quorum": 0.5, "threshold": 0.75},
    "CONTRACT_UPGRADE": {"quorum": 0.6, "threshold": 0.8}
}

# ============================================================================
# CHAT-NATIVE TRANSACTION FIELDS
# ============================================================================

MAX_CHAT_CONTEXT_ID_LENGTH = 36
MAX_TRANSCRIPT_HASH_LENGTH = 64
MAX_JURISDICTION_TAGS = 10

JURISDICTIONS: List[str] = [
    "US_federal",
    "EU",
    "UK",
    "cross_border",
    "US_state_CA",
    "US_state_NY",
    "GDPR",
    "CCPA",
    "JFMIP"
]

# ============================================================================
# TPS TARGETS
# ============================================================================

TPS_TARGETS = {
    "CHAT_ROUTER": {"baseline": 10000, "burst": 100000},
    "WALLET": {"baseline": 15000, "burst": 150000},
    "GOVERNANCE": {"baseline": 5000, "burst": 50000},
    "AGENT": {"baseline": 8000, "burst": 80000},
    "MIGRATION": {"baseline": 2000, "burst": 10000},
    "TOTAL_NETWORK": {"baseline": 200000, "burst": 500000}
}

# ============================================================================
# SAFETY CONSTRAINTS
# ============================================================================

SAFETY_LIMITS = {
    "MAX_UINT64": 18446744073709551615,
    "MIN_AMOUNT": 0,
    "MAX_TRANSFER_AMOUNT": 1000000000000000000,
    "MAX_DELEGATION": 10000000000000000000
}

NANOSWARM_CLASSES = [
    "class_1_minimal",
    "class_2_controlled",
    "class_3_contained",
    "class_4_restricted"
]

BCI_SAFETY_LEVELS = [
    "read_only",
    "write_supervised",
    "write_emergency"
]

# ============================================================================
# AUDIT & COMPLIANCE
# ============================================================================

AUDIT_RETENTION_MS = 220752000000  # 7 years

POLICY_VERSIONS = {
    "JFMIP": "24-01",
    "GDPR": "2016/679",
    "MICA": "2023/1114",
    "FATF": "R16-2019"
}

# ============================================================================
# NETWORK ENDPOINTS
# ============================================================================

PORTS = {
    "HTTP_API": 3000,
    "WEBSOCKET": 3001,
    "EXPLORER": 8080,
    "METRICS": 9090
}

RATE_LIMITS = {
    "PUBLIC_API": 1000,
    "AUTHENTICATED_API": 10000,
    "AGENT_API": 50000,
    "WEBSOCKET_SUBSCRIPTIONS": 100
}

# ============================================================================
# ENUMS
# ============================================================================

class TreasuryRoutingPurpose(Enum):
    """Treasury routing purpose enum"""
    GOVERNANCE_VOTE = "governance_vote"
    PROTOCOL_FEE = "protocol_fee"
    TREASURY_REFILL = "treasury_refill"

# ============================================================================
# VALIDATION HELPERS
# ============================================================================

def is_treasury_live() -> bool:
    """Check if treasury is live and ready for transactions"""
    return ALN_TREASURY_LIVE

def is_valid_treasury_address(address: str) -> bool:
    """Validate treasury address format"""
    return address == ALN_TREASURY_ADDRESS

def is_valid_jurisdiction(tag: str) -> bool:
    """Validate jurisdiction tag"""
    return tag in JURISDICTIONS
