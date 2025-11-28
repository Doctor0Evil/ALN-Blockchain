/**
 * QPU.Math+ Safety Hooks Unit Tests
 */

const { verifyConservation, verifyLimits } = require('../../core/safety/qpu_math_hooks');

describe('QPU.Math+ Safety Hooks', () => {
  describe('verifyConservation', () => {
    test('should accept valid transfer', () => {
      const chainlexeme = {
        header: {
          op_code: 'transfer',
          from: 'aln1abc',
          to: 'aln1xyz',
          nonce: 1
        },
        data: {
          amount: '1000000',
          asset: 'ALN'
        },
        footer: {}
      };

      const result = verifyConservation(chainlexeme);
      expect(result.valid).toBe(true);
    });

    test('should reject negative amount', () => {
      const chainlexeme = {
        header: {
          op_code: 'transfer',
          from: 'aln1abc',
          to: 'aln1xyz',
          nonce: 1
        },
        data: {
          amount: '-1000',
          asset: 'ALN'
        },
        footer: {}
      };

      const result = verifyConservation(chainlexeme);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });
  });

  describe('verifyLimits', () => {
    test('should accept valid gas limits', () => {
      const chainlexeme = {
        header: {
          op_code: 'transfer',
          from: 'aln1abc',
          to: 'aln1xyz',
          nonce: 1
        },
        data: {
          amount: '1000'
        },
        footer: {
          gas_limit: 21000,
          gas_price: 100,
          timestamp: Math.floor(Date.now() / 1000)
        }
      };

      const result = verifyLimits(chainlexeme);
      expect(result.valid).toBe(true);
    });

    test('should reject gas limit below minimum', () => {
      const chainlexeme = {
        header: {
          op_code: 'transfer',
          from: 'aln1abc',
          to: 'aln1xyz',
          nonce: 1
        },
        data: {
          amount: '1000'
        },
        footer: {
          gas_limit: 10000,  // Below 21000 minimum
          gas_price: 100,
          timestamp: Math.floor(Date.now() / 1000)
        }
      };

      const result = verifyLimits(chainlexeme);
      expect(result.valid).toBe(false);
    });

    test('should validate governance proposal limits', () => {
      const chainlexeme = {
        header: {
          op_code: 'governance_proposal',
          from: 'aln1abc',
          to: 'aln1gov',
          nonce: 1
        },
        data: {
          duration_blocks: 10000,
          quorum: 0.4,
          threshold: 0.66
        },
        footer: {
          gas_limit: 500000,
          gas_price: 200,
          timestamp: Math.floor(Date.now() / 1000)
        }
      };

      const result = verifyLimits(chainlexeme);
      expect(result.valid).toBe(true);
    });
  });
});
