# ALN Blockchain - Complete Implementation

## Overview

ALN (Advanced Ledger Network) is a production-grade blockchain implementation featuring:

- **Core Runtime**: Deterministic state machine with ALN syntax (chainlexemes)
- **Explorer UI**: Real-time block/transaction viewer with migragraph charts
- **Non-Custodial Wallet**: Browser-based wallet with local key management
- **DAO Governance**: CHATAI token-based voting via governograms
- **Migration Bridge**: Secure Canto→ALN asset migration with proof verification

## Architecture

```
/aln/
├── core/          # Blockchain consensus, state, and runtime
├── explorer/      # Web-based block explorer UI
├── wallet/        # Browser wallet integration
├── migration/     # Cross-chain bridge logic
├── dao/           # Governance and CHATAI token
├── tools/         # Development utilities and linter
├── tests/         # Unit and e2e test suites
└── docs/          # Comprehensive documentation
```

## Quick Start

### Prerequisites

- Node.js >= 18.0.0
- npm >= 9.0.0
- LevelDB or compatible key-value store

### Installation

```bash
# Install all workspace dependencies
npm run install-all

# Build all modules
npm run build

# Run tests
npm run test
```

### Running a Local Node

```bash
# Initialize node with default nanotopologene profile
npm run start:node -- init

# Start solo consensus node
npm run start:node -- start

# Check node status
npm run start:node -- status
```

### Running the Explorer

```bash
# Start explorer UI (default: http://localhost:3001)
npm run start:explorer
```

### Development Mode

```bash
# Run node + explorer concurrently
npm run dev
```

## Core Concepts

### Chainlexemes

Minimal ALN instruction units encoding state changes, compliance context, and verification constraints:

```aln
[header]
op_code: transfer
from: aln1abc...
to: aln1xyz...
nonce: 42

[data]
asset: ALN
amount: 1000000
constraints: ["max_daily_limit", "kyc_verified"]

[footer]
signature: 0x...
timestamp: 1732723200
```

### Migragraph

Time-ordered visualization of cross-chain asset migrations:

- Tracks Canto→ALN transfers
- Cryptographically verifiable state transitions
- Real-time chart updates in explorer

### Governogram

Structured DAO proposal format:

```aln
[header]
proposal_id: gov_001
title: "Increase Block Size to 2MB"
proposer: aln1gov...

[metadata]
category: parameter_change
execution_route: core.config.setBlockSize
quorum: 0.4
threshold: 0.66
duration_blocks: 10000

[data]
current_value: 1048576
proposed_value: 2097152

[audit]
safety_class: low_risk
review_notes: "Benchmarked on testnet"

[footer]
created_at: 1732723200
```

## API Endpoints

### Node API (default: http://localhost:3000)

- `GET /status` - Node status and block height
- `GET /account/:addr` - Account balance and nonce
- `POST /tx` - Submit transaction (chainlexeme)
- `GET /block/:height` - Block details
- `WS /events` - Real-time block and migration events

### Governance API

- `GET /governance/proposals` - List all proposals
- `GET /governance/proposals/:id` - Proposal details
- `POST /governance/proposals` - Submit new proposal
- `POST /governance/proposals/:id/vote` - Cast vote

### Migration API

- `POST /migration/request` - Initiate Canto→ALN migration
- `GET /migration/history/:addr` - Migration history (migragraph data)

## Security

### Non-Custodial Wallet

- Private keys generated and stored in browser only
- Uses Web Crypto API (`crypto.subtle`)
- Never transmitted to backend
- Deterministic key derivation from user seed

### QPU.Math+ Safety

All transactions validated via:

- `verifyConservation(chainlexeme)` - Token balance conservation
- `verifyLimits(chainlexeme)` - Per-tx and per-account limits

### Governance Safety

- Upgrade proposals require DAO vote
- Module-level upgrade policies enforced
- Quorum and threshold requirements
- Audit trail for all governance actions

## Development

### Workspace Structure

Each module is an npm workspace with its own `package.json`:

```bash
aln/
├── core/package.json
├── explorer/package.json
├── wallet/package.json
└── ...
```

### Adding a New Module

```bash
# Create module directory
mkdir -p aln/mymodule

# Initialize package.json
cd aln/mymodule
npm init -w aln/mymodule
```

### Running Tests

```bash
# All tests
npm test

# Specific workspace
npm test --workspace=aln/core

# E2E tests only
npm test --workspace=aln/tests
```

### Linting

```bash
# Lint all workspaces
npm run lint

# Lint and fix
npm run lint -- --fix
```

## Configuration

### Nanotopologene Profile

Node configuration via `/aln/core/config/nanotopologene_profile.aln`:

```aln
[header]
node_id: node_001
profile_version: 1.0

[capabilities]
ops_threshold_TOPS: 1000
topology_matrix: [[1,0,0],[0,1,0],[0,0,1]]
compliance_level: surgical_grade
ai_firmware_version: ALN.QPU.Math+

[network]
p2p_port: 26656
rpc_port: 26657
api_port: 3000

[footer]
created_at: 1732723200
```

## Migration from Canto

### User Flow

1. Lock assets in Canto escrow contract
2. Submit Canto transaction proof to ALN
3. ALN verifies proof and mints bridged assets
4. Track migration in migragraph

### Operator Flow

1. Configure bridge validator in nanotopologene profile
2. Monitor Canto events via `canto_adapter.js`
3. Validate proofs using `verifyCantoProof()`
4. Record events via `migration_recorder.js`

## Governance Lifecycle

1. **Draft**: Create governogram with proposal details
2. **Submit**: Post governogram via API (becomes chainlexeme)
3. **Vote**: Token holders cast votes (snapshot at submission block)
4. **Execute**: If passed, execution route triggered automatically
5. **Archive**: Proposal and results stored in state

## Troubleshooting

### Node Won't Start

```bash
# Check DB permissions
ls -la ~/.aln/data

# Re-initialize
npm run start:node -- init --force
```

### Explorer Connection Failed

```bash
# Verify node API is running
curl http://localhost:3000/status

# Check CORS settings in http_server.js
```

### Migration Proof Invalid

- Ensure Canto node is synced
- Verify proof format matches expected schema
- Check bridge contract address configuration

## Contributing

See `/aln/docs/developer_onboarding.aln` for detailed setup instructions.

## License

MIT License - See LICENSE file for details

## Support

- Documentation: `/aln/docs/`
- Issues: GitHub Issues
- Community: [Discord/Forum link]

---

**aln_version**: 1.0  
**nanotopologene_profile**: surgical-grade  
**ai_firmware_version**: ALN.QPU.Math+
