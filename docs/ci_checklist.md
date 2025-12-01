# ALN CI Checklist

> Source of truth for CI orchestration until automated jobs are provisioned. Reference this file when defining GitHub Actions, Azure Pipelines, or MCP workflows.

## 1. Governance & CLI Metadata

- **Job Name**: `ci-governance-cli`
- **Steps**:
  1. `npm install`
  2. `npm run test --workspace=aln/tests -- parser_metadata.test.js augmented_policy.test.js`
  3. Smoke-test CLI help (non-network): `node aln/core/cli/aln_node_cli.js tx --help` (expect usage output)
  4. TODO(ci): add scripted invocation that builds a governance vote tx with metadata and asserts JSON schema.

## 2. Key Custodian & Signing

- **Job Name**: `ci-keys-signing`
- **Steps**:
  1. `npm install`
  2. `npm run test --workspace=aln/tests -- key_custodian.test.js tx_builder.test.js`
  3. TODO(ci): add CLI integration test creating sealed key via `aln tx transfer ... --custodian-*` with env var.

## 3. Threat Feed Ingestion

- **Job Name**: `ci-threat-feeds`
- **Steps**:
  1. `npm install`
  2. Run targeted Jest suite once we add tests: `npm run test --workspace=aln/tests -- threat_feed.test.js`
  3. TODO(ci): implement mocked HTTPS + local file feed tests asserting dedupe + risk tier gating.

## 4. Policy Engine Runtime

- **Job Name**: `ci-policy-runtime`
- **Steps**:
  1. Launch node in mock mode (future) or run unit tests that exercise `AugmentedPolicyEngine.validateTransaction`.
  2. Extend test coverage to include `/policy/check` HTTP handler with supertest (TODO).

## 5. Verify Scaffold Guardrails

- **Job Name**: `ci-verify-scaffold`
- **Steps**:
  1. `npm run verify:scaffold`
  2. Fail build if any doc/constant/compliance file missing.

Keep this checklist updated whenever new ALN policies or critical modules are introduced.
