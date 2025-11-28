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
