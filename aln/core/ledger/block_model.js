/**
 * ALN Block and Ledger Structures
 * 
 * Defines block format and ledger management for the ALN blockchain.
 */

const crypto = require('crypto');

class BlockHeader {
  constructor(data = {}) {
    this.version = data.version || 1;
    this.height = data.height || 0;
    this.timestamp = data.timestamp || Math.floor(Date.now() / 1000);
    this.parentHash = data.parentHash || '0'.repeat(64);
    this.stateRoot = data.stateRoot || '0'.repeat(64);
    this.txRoot = data.txRoot || '0'.repeat(64);
    this.validatorSetHash = data.validatorSetHash || '0'.repeat(64);
    this.proposer = data.proposer || '';
    this.hash = data.hash || null;
  }

  /**
   * Compute hash of block header
   * @returns {string} SHA-256 hash
   */
  computeHash() {
    const data = [
      this.version,
      this.height,
      this.timestamp,
      this.parentHash,
      this.stateRoot,
      this.txRoot,
      this.validatorSetHash,
      this.proposer
    ].join('|');

    this.hash = crypto.createHash('sha256').update(data).digest('hex');
    return this.hash;
  }

  /**
   * Serialize to JSON
   */
  toJSON() {
    return {
      version: this.version,
      height: this.height,
      timestamp: this.timestamp,
      parentHash: this.parentHash,
      stateRoot: this.stateRoot,
      txRoot: this.txRoot,
      validatorSetHash: this.validatorSetHash,
      proposer: this.proposer,
      hash: this.hash
    };
  }

  /**
   * Deserialize from JSON
   */
  static fromJSON(json) {
    return new BlockHeader(json);
  }
}

class Block {
  constructor(header, transactions = []) {
    this.header = header instanceof BlockHeader ? header : new BlockHeader(header);
    this.transactions = transactions; // Array of chainlexemes
  }

  /**
   * Compute transaction root (Merkle root of all tx hashes)
   * @returns {string} Transaction root hash
   */
  computeTxRoot() {
    if (this.transactions.length === 0) {
      return '0'.repeat(64);
    }

    // Hash each transaction
    const txHashes = this.transactions.map(tx => {
      const txData = JSON.stringify(tx);
      return crypto.createHash('sha256').update(txData).digest('hex');
    });

    // Compute Merkle root
    return this._computeMerkleRoot(txHashes);
  }

  /**
   * Compute Merkle root from list of hashes
   */
  _computeMerkleRoot(hashes) {
    if (hashes.length === 0) return '0'.repeat(64);
    if (hashes.length === 1) return hashes[0];

    const newLevel = [];
    for (let i = 0; i < hashes.length; i += 2) {
      if (i + 1 < hashes.length) {
        const combined = crypto.createHash('sha256')
          .update(hashes[i] + hashes[i + 1])
          .digest('hex');
        newLevel.push(combined);
      } else {
        newLevel.push(hashes[i]);
      }
    }

    return this._computeMerkleRoot(newLevel);
  }

  /**
   * Finalize block by computing all hashes
   * @param {string} stateRoot - State root from state store
   * @returns {Block} This block (for chaining)
   */
  finalize(stateRoot) {
    this.header.txRoot = this.computeTxRoot();
    this.header.stateRoot = stateRoot;
    this.header.computeHash();
    return this;
  }

  /**
   * Serialize to JSON
   */
  toJSON() {
    return {
      header: this.header.toJSON(),
      transactions: this.transactions
    };
  }

  /**
   * Deserialize from JSON
   */
  static fromJSON(json) {
    const header = BlockHeader.fromJSON(json.header);
    return new Block(header, json.transactions);
  }
}

/**
 * Compute block hash from header data
 * @param {BlockHeader} header - Block header
 * @returns {string} Block hash
 */
function computeBlockHash(header) {
  if (header.hash) return header.hash;
  return header.computeHash();
}

/**
 * Apply block to state store
 * @param {Block} block - Block to apply
 * @param {StateStore} stateStore - State store instance
 * @returns {Promise<Object>} Result with new state root and applied tx count
 */
async function applyBlock(block, stateStore) {
  const result = {
    success: false,
    stateRoot: null,
    appliedTxCount: 0,
    failedTxCount: 0,
    errors: []
  };

  try {
    // Apply each transaction
    for (const tx of block.transactions) {
      const txResult = await stateStore.applyTransaction(tx);
      
      if (txResult.success) {
        result.appliedTxCount++;
      } else {
        result.failedTxCount++;
        result.errors.push({
          tx: tx.header,
          error: txResult.error
        });
      }
    }

    // Compute new state root
    result.stateRoot = await stateStore.computeStateRoot();
    
    // Clear cache after block application
    stateStore.clearCache();
    
    result.success = true;
    return result;

  } catch (err) {
    result.errors.push({ error: err.message });
    return result;
  }
}

/**
 * Verify block validity
 * @param {Block} block - Block to verify
 * @param {Block} parentBlock - Parent block
 * @returns {Object} Validation result
 */
function verifyBlock(block, parentBlock) {
  const errors = [];

  // Check height
  if (block.header.height !== parentBlock.header.height + 1) {
    errors.push(`Invalid height: expected ${parentBlock.header.height + 1}, got ${block.header.height}`);
  }

  // Check parent hash
  if (block.header.parentHash !== parentBlock.header.hash) {
    errors.push(`Invalid parent hash`);
  }

  // Check timestamp
  if (block.header.timestamp <= parentBlock.header.timestamp) {
    errors.push(`Block timestamp must be greater than parent`);
  }

  // Verify block hash
  const computedHash = computeBlockHash(block.header);
  if (computedHash !== block.header.hash) {
    errors.push(`Invalid block hash`);
  }

  // Verify tx root
  const computedTxRoot = block.computeTxRoot();
  if (computedTxRoot !== block.header.txRoot) {
    errors.push(`Invalid transaction root`);
  }

  return {
    valid: errors.length === 0,
    errors
  };
}

module.exports = {
  BlockHeader,
  Block,
  computeBlockHash,
  applyBlock,
  verifyBlock
};
