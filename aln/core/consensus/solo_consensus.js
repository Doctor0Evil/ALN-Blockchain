/**
 * ALN Solo Consensus Engine
 * 
 * Development-mode consensus that periodically packs pending transactions
 * into blocks. Designed to be swapped for BFT/PoS in production.
 */

const { Block, BlockHeader, applyBlock } = require('../ledger/block_model');
const { validateChainlexemes } = require('../runtime/aln_parser');
const { verifyConservation, verifyLimits } = require('../safety/qpu_math_hooks');
const EventEmitter = require('events');

class SoloConsensus extends EventEmitter {
  constructor(stateStore, options = {}) {
    super();
    this.stateStore = stateStore;
    const { policyEngine = null, ...config } = options;
    this.policyEngine = policyEngine;
    this.config = {
      blockTime: config.blockTime || 5000, // 5 seconds
      maxTxPerBlock: config.maxTxPerBlock || 1000,
      nodeId: config.nodeId || 'solo_node_001',
      ...config
    };

    this.mempool = []; // Pending transactions
    this.blockchain = []; // Array of blocks
    this.currentHeight = 0;
    this.isRunning = false;
    this.blockTimer = null;
  }

  /**
   * Initialize genesis block
   */
  async initialize() {
    console.log('[SoloConsensus] Initializing with genesis block...');
    
    const genesisHeader = new BlockHeader({
      version: 1,
      height: 0,
      timestamp: Math.floor(Date.now() / 1000),
      parentHash: '0'.repeat(64),
      proposer: this.config.nodeId
    });

    const genesisStateRoot = await this.stateStore.computeStateRoot();
    const genesisBlock = new Block(genesisHeader, []);
    genesisBlock.finalize(genesisStateRoot);

    this.blockchain.push(genesisBlock);
    this.currentHeight = 0;

    console.log(`[SoloConsensus] Genesis block created: ${genesisBlock.header.hash}`);
    return genesisBlock;
  }

  /**
   * Start consensus engine
   */
  async start() {
    if (this.isRunning) {
      console.log('[SoloConsensus] Already running');
      return;
    }

    console.log(`[SoloConsensus] Starting with ${this.config.blockTime}ms block time...`);
    this.isRunning = true;

    // Start block production timer
    this.blockTimer = setInterval(() => {
      this._produceBlock().catch(err => {
        console.error('[SoloConsensus] Block production error:', err);
      });
    }, this.config.blockTime);
  }

  /**
   * Stop consensus engine
   */
  stop() {
    console.log('[SoloConsensus] Stopping...');
    this.isRunning = false;
    if (this.blockTimer) {
      clearInterval(this.blockTimer);
      this.blockTimer = null;
    }
  }

  /**
   * Submit transaction to mempool
   * @param {Object} chainlexeme - Parsed chainlexeme
   * @returns {Object} Result with transaction hash
   */
  submitTransaction(chainlexeme) {
    // Validate chainlexeme structure
    const validationReport = validateChainlexemes(chainlexeme);
    if (!validationReport.isValid) {
      return {
        success: false,
        error: 'Invalid chainlexeme structure',
        details: validationReport.errors
      };
    }

    if (this.policyEngine) {
      const policyResult = this.policyEngine.validateTransaction(chainlexeme);
      if (!policyResult.allowed) {
        return {
          success: false,
          error: 'Policy check failed',
          details: policyResult.reason
        };
      }
    }

    // Verify safety constraints
    const conservationCheck = verifyConservation(chainlexeme);
    if (!conservationCheck.valid) {
      return {
        success: false,
        error: 'Conservation check failed',
        details: conservationCheck.errors
      };
    }

    const limitsCheck = verifyLimits(chainlexeme);
    if (!limitsCheck.valid) {
      return {
        success: false,
        error: 'Limits check failed',
        details: limitsCheck.errors
      };
    }

    // Add to mempool
    const txHash = this._computeTxHash(chainlexeme);
    this.mempool.push({ chainlexeme, txHash, timestamp: Date.now() });

    console.log(`[SoloConsensus] Transaction added to mempool: ${txHash}`);

    return {
      success: true,
      txHash,
      mempoolSize: this.mempool.length
    };
  }

  /**
   * Produce new block from mempool
   */
  async _produceBlock() {
    if (this.mempool.length === 0) {
      // No transactions to include
      return;
    }

    console.log(`[SoloConsensus] Producing block ${this.currentHeight + 1} with ${this.mempool.length} pending tx...`);

    // Take transactions from mempool
    const txCount = Math.min(this.mempool.length, this.config.maxTxPerBlock);
    const selectedTxs = this.mempool.splice(0, txCount);
    const transactions = selectedTxs.map(item => item.chainlexeme);

    // Get parent block
    const parentBlock = this.blockchain[this.blockchain.length - 1];

    // Create new block header
    const header = new BlockHeader({
      version: 1,
      height: this.currentHeight + 1,
      timestamp: Math.floor(Date.now() / 1000),
      parentHash: parentBlock.header.hash,
      proposer: this.config.nodeId
    });

    const block = new Block(header, transactions);

    // Apply block to state
    const applyResult = await applyBlock(block, this.stateStore);
    
    if (!applyResult.success) {
      console.error('[SoloConsensus] Failed to apply block:', applyResult.errors);
      // Return failed transactions to mempool
      this.mempool.unshift(...selectedTxs);
      return;
    }

    // Finalize block
    block.finalize(applyResult.stateRoot);

    // Add to blockchain
    this.blockchain.push(block);
    this.currentHeight++;

    console.log(`[SoloConsensus] Block ${block.header.height} created: ${block.header.hash}`);
    console.log(`[SoloConsensus] Applied ${applyResult.appliedTxCount} transactions, ${applyResult.failedTxCount} failed`);

    // Emit event
    this.emit('newBlock', {
      block: block.toJSON(),
      appliedTxCount: applyResult.appliedTxCount,
      failedTxCount: applyResult.failedTxCount
    });
  }

  /**
   * Compute transaction hash
   */
  _computeTxHash(chainlexeme) {
    const crypto = require('crypto');
    const data = JSON.stringify(chainlexeme);
    return crypto.createHash('sha256').update(data).digest('hex');
  }

  /**
   * Get current blockchain status
   */
  getStatus() {
    const latestBlock = this.blockchain[this.blockchain.length - 1];
    return {
      nodeId: this.config.nodeId,
      height: this.currentHeight,
      latestBlockHash: latestBlock ? latestBlock.header.hash : null,
      latestBlockTime: latestBlock ? latestBlock.header.timestamp : null,
      mempoolSize: this.mempool.length,
      isRunning: this.isRunning
    };
  }

  /**
   * Get block by height
   */
  getBlock(height) {
    if (height < 0 || height >= this.blockchain.length) {
      return null;
    }
    return this.blockchain[height];
  }

  /**
   * Register callback for new blocks
   */
  onNewBlock(callback) {
    this.on('newBlock', callback);
  }
}

module.exports = SoloConsensus;
