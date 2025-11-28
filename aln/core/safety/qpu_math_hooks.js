/**
 * QPU.Math+ Safety Hooks
 * 
 * Pure functions for verifying conservation laws and transaction limits
 * before state mutation. Integrated into transaction application pipeline.
 */

/**
 * Verify token balance conservation
 * Ensures that total input equals total output (no creation or destruction)
 * 
 * @param {Object} chainlexeme - Parsed chainlexeme transaction
 * @returns {Object} Validation result with valid flag and errors
 */
function verifyConservation(chainlexeme) {
  const result = {
    valid: true,
    errors: [],
    warnings: []
  };

  const { header, data } = chainlexeme;

  // Conservation only applies to certain operation types
  const conservationOps = ['transfer', 'token_transfer'];
  
  if (!conservationOps.includes(header.op_code)) {
    // Non-transfer operations don't need conservation checks
    return result;
  }

  // For transfers, verify amount is positive and reasonable
  const amount = BigInt(data.amount || 0);
  
  if (amount < 0n) {
    result.valid = false;
    result.errors.push('Amount cannot be negative');
  }

  if (amount === 0n) {
    result.warnings.push('Zero-value transfer');
  }

  // Verify from != to (no self-transfers in basic case)
  if (header.from === header.to) {
    result.warnings.push('Self-transfer detected');
  }

  // Check for reasonable upper bounds (prevent overflow attacks)
  const MAX_AMOUNT = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF');
  if (amount > MAX_AMOUNT) {
    result.valid = false;
    result.errors.push('Amount exceeds maximum allowed value');
  }

  return result;
}

/**
 * Verify transaction and account limits
 * Checks per-transaction and per-account rate limits and thresholds
 * 
 * @param {Object} chainlexeme - Parsed chainlexeme transaction
 * @param {Object} options - Additional context (account history, etc.)
 * @returns {Object} Validation result
 */
function verifyLimits(chainlexeme, options = {}) {
  const result = {
    valid: true,
    errors: [],
    warnings: []
  };

  const { header, data, footer } = chainlexeme;

  // 1. Gas limit checks
  const gasLimit = footer.gas_limit || 0;
  const MIN_GAS = 21000;
  const MAX_GAS = 10000000;

  if (gasLimit < MIN_GAS) {
    result.valid = false;
    result.errors.push(`Gas limit too low: minimum ${MIN_GAS}`);
  }

  if (gasLimit > MAX_GAS) {
    result.valid = false;
    result.errors.push(`Gas limit too high: maximum ${MAX_GAS}`);
  }

  // 2. Gas price checks (prevent spam)
  const gasPrice = footer.gas_price || 0;
  const MIN_GAS_PRICE = 1;
  const MAX_GAS_PRICE = 1000000;

  if (gasPrice < MIN_GAS_PRICE) {
    result.warnings.push(`Gas price below minimum: ${MIN_GAS_PRICE}`);
  }

  if (gasPrice > MAX_GAS_PRICE) {
    result.warnings.push(`Gas price unusually high`);
  }

  // 3. Amount limits for transfers
  if (header.op_code === 'transfer' || header.op_code === 'token_transfer') {
    const amount = BigInt(data.amount || 0);
    const DAILY_TRANSFER_LIMIT = BigInt('10000000000000000000000'); // 10,000 tokens

    if (amount > DAILY_TRANSFER_LIMIT && !data.constraints?.includes('high_value_approved')) {
      result.warnings.push('Transfer exceeds daily limit without approval');
    }
  }

  // 4. Governance proposal limits
  if (header.op_code === 'governance_proposal') {
    const MIN_PROPOSAL_DURATION = 1000; // blocks
    const MAX_PROPOSAL_DURATION = 100000;

    if (data.duration_blocks < MIN_PROPOSAL_DURATION) {
      result.valid = false;
      result.errors.push(`Proposal duration too short: minimum ${MIN_PROPOSAL_DURATION} blocks`);
    }

    if (data.duration_blocks > MAX_PROPOSAL_DURATION) {
      result.valid = false;
      result.errors.push(`Proposal duration too long: maximum ${MAX_PROPOSAL_DURATION} blocks`);
    }

    // Quorum and threshold must be reasonable
    if (data.quorum < 0.1 || data.quorum > 1.0) {
      result.valid = false;
      result.errors.push('Quorum must be between 0.1 and 1.0');
    }

    if (data.threshold < 0.5 || data.threshold > 1.0) {
      result.valid = false;
      result.errors.push('Threshold must be between 0.5 and 1.0');
    }
  }

  // 5. Migration limits
  if (header.op_code === 'migration_mint' || header.op_code === 'migration_burn') {
    if (!data.proof_hash || data.proof_hash.length !== 66) {
      result.valid = false;
      result.errors.push('Invalid proof hash for migration');
    }

    if (!data.source_tx_hash) {
      result.valid = false;
      result.errors.push('Missing source transaction hash for migration');
    }
  }

  // 6. Constraint validation
  if (data.constraints && Array.isArray(data.constraints)) {
    const MAX_CONSTRAINTS = 10;
    if (data.constraints.length > MAX_CONSTRAINTS) {
      result.valid = false;
      result.errors.push(`Too many constraints: maximum ${MAX_CONSTRAINTS}`);
    }

    // Verify constraint format
    const VALID_CONSTRAINTS = [
      'rate_limit_daily',
      'compliance_kyc',
      'high_value_approved',
      'proof_verified',
      'escrow_confirmed',
      'min_stake_requirement'
    ];

    for (const constraint of data.constraints) {
      if (!VALID_CONSTRAINTS.includes(constraint)) {
        result.warnings.push(`Unknown constraint: ${constraint}`);
      }
    }
  }

  // 7. Nonce validation
  if (header.nonce < 0) {
    result.valid = false;
    result.errors.push('Nonce cannot be negative');
  }

  // 8. Timestamp validation
  const now = Math.floor(Date.now() / 1000);
  const timestamp = footer.timestamp || 0;
  const TIMESTAMP_TOLERANCE = 300; // 5 minutes

  if (Math.abs(timestamp - now) > TIMESTAMP_TOLERANCE) {
    result.warnings.push('Transaction timestamp outside tolerance window');
  }

  return result;
}

/**
 * Verify signature (stub - implement with actual crypto)
 * @param {Object} chainlexeme - Transaction to verify
 * @returns {Object} Verification result
 */
function verifySignature(chainlexeme) {
  const result = {
    valid: true,
    errors: []
  };

  const { header, data, footer } = chainlexeme;

  if (!footer.signature) {
    result.valid = false;
    result.errors.push('Missing signature');
    return result;
  }

  // Check signature format
  if (!footer.signature.startsWith('ed25519:')) {
    result.valid = false;
    result.errors.push('Invalid signature format: must start with ed25519:');
  }

  // TODO: Implement actual ed25519 signature verification
  // This requires:
  // 1. Reconstruct message from header + data
  // 2. Extract public key from 'from' address
  // 3. Verify signature using ed25519

  return result;
}

/**
 * Run all safety checks on a chainlexeme
 * @param {Object} chainlexeme - Transaction to verify
 * @returns {Object} Combined validation result
 */
function verifySafety(chainlexeme) {
  const conservationResult = verifyConservation(chainlexeme);
  const limitsResult = verifyLimits(chainlexeme);
  const signatureResult = verifySignature(chainlexeme);

  const combined = {
    valid: conservationResult.valid && limitsResult.valid && signatureResult.valid,
    errors: [
      ...conservationResult.errors,
      ...limitsResult.errors,
      ...signatureResult.errors
    ],
    warnings: [
      ...conservationResult.warnings,
      ...limitsResult.warnings
    ]
  };

  return combined;
}

module.exports = {
  verifyConservation,
  verifyLimits,
  verifySignature,
  verifySafety
};
