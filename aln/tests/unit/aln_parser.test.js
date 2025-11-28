/**
 * ALN Parser Unit Tests
 */

const { parseAlnDocument, validateChainlexemes } = require('../../core/runtime/aln_parser');

describe('ALN Parser', () => {
  test('should parse valid ALN document', () => {
    const alnDoc = `
[header]
op_code: transfer
from: aln1abc123
to: aln1xyz789
nonce: 1

[data]
amount: 1000
asset: ALN

[footer]
signature: ed25519:0xdeadbeef
timestamp: 1732723200
    `;

    const parsed = parseAlnDocument(alnDoc);

    expect(parsed.header.op_code).toBe('transfer');
    expect(parsed.header.from).toBe('aln1abc123');
    expect(parsed.header.nonce).toBe(1);
    expect(parsed.data.amount).toBe(1000);
  });

  test('should validate chainlexeme with all required fields', () => {
    const doc = {
      header: {
        op_code: 'transfer',
        from: 'aln1abc123',
        to: 'aln1xyz789',
        nonce: 1
      },
      data: {
        amount: 1000,
        asset: 'ALN'
      },
      footer: {
        signature: 'ed25519:0xdeadbeef',
        timestamp: 1732723200
      }
    };

    const report = validateChainlexemes(doc);
    expect(report.isValid).toBe(true);
    expect(report.errors.length).toBe(0);
  });

  test('should fail validation for missing header fields', () => {
    const doc = {
      header: {
        op_code: 'transfer'
        // missing from, to, nonce
      },
      data: {},
      footer: {}
    };

    const report = validateChainlexemes(doc);
    expect(report.isValid).toBe(false);
    expect(report.errors.length).toBeGreaterThan(0);
  });

  test('should reject invalid op_code', () => {
    const doc = {
      header: {
        op_code: 'invalid_operation',
        from: 'aln1abc123',
        to: 'aln1xyz789',
        nonce: 1
      },
      data: {},
      footer: {
        signature: 'ed25519:0xdeadbeef',
        timestamp: 1732723200
      }
    };

    const report = validateChainlexemes(doc);
    expect(report.isValid).toBe(false);
  });
});
