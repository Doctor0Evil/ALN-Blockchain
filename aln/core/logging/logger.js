/**
 * ALN Structured Logger
 * 
 * Implements structured JSON logging with error code mapping
 * from /aln/core/logging/errors.aln
 */

class Logger {
  constructor(nodeId = 'unknown', context = {}) {
    this.nodeId = nodeId;
    this.context = context;
  }

  /**
   * Log with specified level
   */
  _log(level, message, context = {}) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level: level.toUpperCase(),
      node_id: this.nodeId,
      message,
      context: {
        ...this.context,
        ...context
      }
    };

    // Output to console (in production, send to log aggregation service)
    const output = JSON.stringify(logEntry);
    
    switch (level.toLowerCase()) {
      case 'error':
      case 'fatal':
        console.error(output);
        break;
      case 'warn':
        console.warn(output);
        break;
      default:
        console.log(output);
    }

    return logEntry;
  }

  debug(message, context = {}) {
    return this._log('DEBUG', message, context);
  }

  info(message, context = {}) {
    return this._log('INFO', message, context);
  }

  warn(message, context = {}) {
    return this._log('WARN', message, context);
  }

  error(message, context = {}) {
    return this._log('ERROR', message, context);
  }

  fatal(message, context = {}) {
    return this._log('FATAL', message, context);
  }

  /**
   * Log with error code from errors.aln
   */
  logError(errorCode, message, context = {}) {
    return this.error(message, {
      error_code: errorCode,
      ...context
    });
  }

  /**
   * Create child logger with additional context
   */
  child(additionalContext) {
    return new Logger(this.nodeId, {
      ...this.context,
      ...additionalContext
    });
  }
}

// Error code constants from errors.aln
const ErrorCodes = {
  // Syntax errors
  SYNTAX_INVALID_STRUCTURE: 'ALN_ERR_SYNTAX_001',
  SYNTAX_MISSING_SECTION: 'ALN_ERR_SYNTAX_002',
  SYNTAX_INVALID_OPCODE: 'ALN_ERR_SYNTAX_003',
  SYNTAX_MALFORMED_FIELD: 'ALN_ERR_SYNTAX_004',
  SYNTAX_SIZE_EXCEEDED: 'ALN_ERR_SYNTAX_005',

  // Balance errors
  BALANCE_INSUFFICIENT: 'ALN_ERR_BALANCE_001',
  BALANCE_NEGATIVE: 'ALN_ERR_BALANCE_002',
  BALANCE_EXCEEDS_MAX: 'ALN_ERR_BALANCE_003',
  BALANCE_CONSERVATION: 'ALN_ERR_BALANCE_004',
  BALANCE_TOKEN_NOT_FOUND: 'ALN_ERR_BALANCE_005',

  // Governance errors
  GOVERNANCE_NOT_FOUND: 'ALN_ERR_GOVERNANCE_001',
  GOVERNANCE_INVALID_PERIOD: 'ALN_ERR_GOVERNANCE_002',
  GOVERNANCE_INSUFFICIENT_POWER: 'ALN_ERR_GOVERNANCE_003',
  GOVERNANCE_NO_QUORUM: 'ALN_ERR_GOVERNANCE_004',
  GOVERNANCE_ALREADY_EXECUTED: 'ALN_ERR_GOVERNANCE_005',
  GOVERNANCE_INVALID_STRUCTURE: 'ALN_ERR_GOVERNANCE_006',

  // Migration errors
  MIGRATION_INVALID_PROOF: 'ALN_ERR_MIGRATION_001',
  MIGRATION_UNSUPPORTED_CHAIN: 'ALN_ERR_MIGRATION_002',
  MIGRATION_ALREADY_PROCESSED: 'ALN_ERR_MIGRATION_003',
  MIGRATION_ESCROW_FAILED: 'ALN_ERR_MIGRATION_004',
  MIGRATION_CONTRACT_NOT_FOUND: 'ALN_ERR_MIGRATION_005',

  // Consensus errors
  CONSENSUS_INVALID_HEIGHT: 'ALN_ERR_CONSENSUS_001',
  CONSENSUS_INVALID_PARENT: 'ALN_ERR_CONSENSUS_002',
  CONSENSUS_INVALID_TIMESTAMP: 'ALN_ERR_CONSENSUS_003',
  CONSENSUS_STATE_MISMATCH: 'ALN_ERR_CONSENSUS_004',
  CONSENSUS_TX_ROOT_MISMATCH: 'ALN_ERR_CONSENSUS_005',

  // Network errors
  NETWORK_PEER_FAILED: 'ALN_ERR_NETWORK_001',
  NETWORK_INVALID_SIGNATURE: 'ALN_ERR_NETWORK_002',
  NETWORK_RATE_LIMIT: 'ALN_ERR_NETWORK_003',
  NETWORK_INVALID_FORMAT: 'ALN_ERR_NETWORK_004',
  NETWORK_PEER_BLACKLISTED: 'ALN_ERR_NETWORK_005',

  // Validation errors
  VALIDATION_INVALID_NONCE: 'ALN_ERR_VALIDATION_001',
  VALIDATION_INVALID_SIGNATURE: 'ALN_ERR_VALIDATION_002',
  VALIDATION_GAS_TOO_LOW: 'ALN_ERR_VALIDATION_003',
  VALIDATION_GAS_TOO_HIGH: 'ALN_ERR_VALIDATION_004',
  VALIDATION_CONSTRAINT_FAILED: 'ALN_ERR_VALIDATION_005',
  VALIDATION_TIMESTAMP_OUT_OF_RANGE: 'ALN_ERR_VALIDATION_006'
};

module.exports = {
  Logger,
  ErrorCodes
};
