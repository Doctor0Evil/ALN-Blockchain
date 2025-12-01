/**
 * Parser and Chat Metadata Tests
 * Verify module syntax parsing and chat-native metadata warnings
 */

const { ALNParser } = require('../../core/runtime/aln_parser');
const { GOVERNANCE_ADDRESSES } = require('../../core/config/governance');

const GOVERNANCE_ADDRESS = GOVERNANCE_ADDRESSES.COUNCIL;

describe('ALNParser - Module Syntax', () => {
  let parser;

  beforeEach(() => {
    parser = new ALNParser();
  });

  test('recognizes aln_module block', () => {
    const chainlexeme = `
aln_module "test_module" {
  version: "1.0"
  description: "Test module"
}
    `;

    const result = parser.parse(chainlexeme);
    expect(result.valid).toBe(true);
    expect(result.metadata.modules).toContainEqual(
      expect.objectContaining({ name: 'test_module' })
    );
  });

  test('recognizes aln_policy block', () => {
    const chainlexeme = `
aln_policy "test_policy" {
  version: "1.0"
  description: "Test policy"
}
    `;

    const result = parser.parse(chainlexeme);
    expect(result.valid).toBe(true);
    expect(result.metadata.modules).toContainEqual(
      expect.objectContaining({ name: 'test_policy', type: 'policy' })
    );
  });

  test('warns on missing chat_context_id for governance_vote', () => {
    const chainlexeme = `
chainlexeme {
  header: {
    op_code: "governance_vote",
    from: "aln1voter",
    to: "${GOVERNANCE_ADDRESS}",
    nonce: 1
  },
  data: {
    proposal_id: "prop_123",
    support: true
  },
  footer: {
    signature: "sig",
    timestamp: 1234567890,
    gas_limit: 100000,
    gas_price: 150
  }
}
    `;

    const result = parser.parse(chainlexeme);
    expect(result.warnings).toContainEqual(
      expect.stringContaining('governance_vote without chat_context_id')
    );
  });

  test('warns on missing transcript_hash for migration_ingest', () => {
    const chainlexeme = `
chainlexeme {
  header: {
    op_code: "migration_ingest",
    from: "aln1user",
    to: "aln1migration",
    nonce: 1
  },
  data: {
    contract_address: "0xabc123"
  },
  footer: {
    signature: "sig",
    timestamp: 1234567890,
    gas_limit: 500000,
    gas_price: 200
  }
}
    `;

    const result = parser.parse(chainlexeme);
    expect(result.warnings).toContainEqual(
      expect.stringContaining('migration_ingest without transcript_hash')
    );
  });

  test('validates jurisdiction_tags array length', () => {
    const chainlexeme = `
chainlexeme {
  header: {
    op_code: "transfer",
    from: "aln1sender",
    to: "aln1receiver",
    nonce: 1,
    jurisdiction_tags: ["US_federal", "EU", "UK", "cross_border", "US_state_CA", "US_state_NY", "GDPR", "CCPA", "JFMIP", "extra1", "extra2"]
  },
  data: {
    asset: "ALN",
    amount: "1000"
  },
  footer: {
    signature: "sig",
    timestamp: 1234567890,
    gas_limit: 21000,
    gas_price: 100
  }
}
    `;

    const result = parser.parse(chainlexeme);
    expect(result.valid).toBe(false);
    expect(result.errors).toContainEqual(
      expect.stringContaining('jurisdiction_tags exceeds max')
    );
  });

  test('accepts valid chat metadata', () => {
    const chainlexeme = `
chainlexeme {
  header: {
    op_code: "governance_vote",
    from: "aln1voter",
    to: "${GOVERNANCE_ADDRESS}",
    nonce: 1,
    chat_context_id: "550e8400-e29b-41d4-a716-446655440000",
    transcript_hash: "a".repeat(64),
    jurisdiction_tags: ["US_federal"]
  },
  data: {
    proposal_id: "prop_123",
    support: true
  },
  footer: {
    signature: "sig",
    timestamp: 1234567890,
    gas_limit: 100000,
    gas_price: 150
  }
}
    `;

    const result = parser.parse(chainlexeme);
    expect(result.valid).toBe(true);
    expect(result.warnings).toHaveLength(0);
  });
});
