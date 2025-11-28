#!/usr/bin/env node

/**
 * ALN Node CLI
 * 
 * Command-line interface for running ALN blockchain node
 */

const fs = require('fs');
const path = require('path');
const StateStore = require('../state/state_store');
const SoloConsensus = require('../consensus/solo_consensus');
const HttpServer = require('../api/http_server');
const { loadProfileWithDefaults } = require('../config/nanotopologene_loader');
const { Logger } = require('../logging/logger');

const DEFAULT_PROFILE_PATH = path.join(__dirname, '../config/nanotopologene_profile.aln');

class AlnNodeCLI {
  constructor() {
    this.profile = null;
    this.logger = new Logger('cli');
  }

  /**
   * Initialize node data directory
   */
  async init(options = {}) {
    console.log('=== ALN Node Initialization ===\n');

    const dataDir = options.dataDir || './data';
    const force = options.force || false;

    // Check if already initialized
    if (fs.existsSync(dataDir) && !force) {
      console.log(`❌ Data directory already exists: ${dataDir}`);
      console.log('Use --force to reinitialize\n');
      return;
    }

    // Create directories
    console.log(`📁 Creating data directory: ${dataDir}`);
    fs.mkdirSync(path.join(dataDir, 'state'), { recursive: true });
    fs.mkdirSync(path.join(dataDir, 'blocks'), { recursive: true });

    // Load profile
    this.profile = loadProfileWithDefaults(
      options.profile || (fs.existsSync(DEFAULT_PROFILE_PATH) ? DEFAULT_PROFILE_PATH : null)
    );

    console.log(`✅ Node ID: ${this.profile.getNodeId()}`);
    console.log(`✅ Compliance Level: ${this.profile.getComplianceLevel()}`);
    console.log(`✅ Firmware: ${this.profile.getFirmwareVersion()}`);

    // Initialize state store with genesis
    console.log('\n📦 Initializing state store...');
    const stateStore = new StateStore(path.join(dataDir, 'state'));
    await stateStore.open();

    // Create genesis accounts (optional)
    if (options.genesisAccounts) {
      console.log('\n💰 Creating genesis accounts...');
      for (const account of options.genesisAccounts) {
        await stateStore.setAccount(account.address, {
          address: account.address,
          nonce: 0,
          balance: account.balance || '0',
          token_balances: {},
          voting_power: '0',
          delegated_to: null,
          code_hash: null,
          storage_root: null
        });
        console.log(`  ✓ ${account.address}: ${account.balance} ALN`);
      }
    }

    await stateStore.close();

    console.log('\n✅ Initialization complete!\n');
    console.log('Next steps:');
    console.log('  1. Start node: aln start');
    console.log('  2. Check status: aln status\n');
  }

  /**
   * Start node
   */
  async start(options = {}) {
    console.log('=== Starting ALN Node ===\n');

    // Load profile
    this.profile = loadProfileWithDefaults(
      options.profile || (fs.existsSync(DEFAULT_PROFILE_PATH) ? DEFAULT_PROFILE_PATH : null)
    );

    const validation = this.profile.validate();
    if (!validation.valid) {
      console.error('❌ Invalid profile:');
      validation.errors.forEach(err => console.error(`  - ${err}`));
      process.exit(1);
    }

    const storageConfig = this.profile.getStorageConfig();
    const networkConfig = this.profile.getNetworkConfig();
    const consensusConfig = this.profile.getConsensusConfig();

    console.log(`🔧 Node ID: ${this.profile.getNodeId()}`);
    console.log(`🔧 Consensus: ${consensusConfig.mode}`);
    console.log(`🔧 Block Time: ${consensusConfig.blockTime}ms`);
    console.log(`🔧 API Port: ${networkConfig.apiPort}`);
    console.log(`🔧 WS Port: ${networkConfig.wsPort}\n`);

    // Initialize state store
    console.log('📦 Opening state store...');
    const stateStore = new StateStore(storageConfig.stateDir);
    await stateStore.open();

    // Initialize consensus
    console.log('⚙️  Initializing consensus engine...');
    const consensus = new SoloConsensus(stateStore, {
      blockTime: consensusConfig.blockTime,
      nodeId: this.profile.getNodeId()
    });

    await consensus.initialize();

    // Start HTTP API
    if (networkConfig.apiEnabled) {
      console.log('🌐 Starting HTTP API server...');
      const httpServer = new HttpServer(consensus, stateStore, {
        apiPort: networkConfig.apiPort,
        wsPort: networkConfig.wsPort,
        nodeId: this.profile.getNodeId()
      });
      httpServer.start();
    }

    // Start consensus
    console.log('🚀 Starting block production...\n');
    await consensus.start();

    console.log('✅ ALN Node is running!\n');
    console.log('API Endpoints:');
    console.log(`  - Status: http://localhost:${networkConfig.apiPort}/status`);
    console.log(`  - Submit TX: http://localhost:${networkConfig.apiPort}/tx`);
    console.log(`  - WebSocket: ws://localhost:${networkConfig.wsPort}/events\n`);

    // Handle graceful shutdown
    process.on('SIGINT', () => {
      console.log('\n\n⏹️  Shutting down...');
      consensus.stop();
      stateStore.close().then(() => {
        console.log('✅ Node stopped gracefully\n');
        process.exit(0);
      });
    });
  }

  /**
   * Show node status
   */
  async status(options = {}) {
    const apiPort = options.apiPort || 3000;
    
    try {
      const fetch = (await import('node-fetch')).default;
      const response = await fetch(`http://localhost:${apiPort}/status`);
      const data = await response.json();

      if (data.success) {
        console.log('=== ALN Node Status ===\n');
        console.log(`Node ID:      ${data.data.nodeId}`);
        console.log(`Height:       ${data.data.height}`);
        console.log(`Latest Block: ${data.data.latestBlockHash || 'N/A'}`);
        console.log(`Mempool:      ${data.data.mempoolSize} pending tx`);
        console.log(`Status:       ${data.data.isRunning ? '🟢 Running' : '🔴 Stopped'}\n`);
      } else {
        console.log('❌ Failed to get status\n');
      }
    } catch (err) {
      console.log('❌ Node not reachable. Is it running?\n');
      console.log(`Error: ${err.message}\n`);
    }
  }
}

// CLI entry point
if (require.main === module) {
  const cli = new AlnNodeCLI();
  const args = process.argv.slice(2);
  const command = args[0];

  switch (command) {
    case 'init':
      cli.init({
        force: args.includes('--force'),
        genesisAccounts: [
          { address: 'aln1qpzry9x8gf2tvdw0s3jn54khce6mua7l5tgj3e', balance: '1000000000000000000000' }
        ]
      }).catch(console.error);
      break;

    case 'start':
      cli.start().catch(console.error);
      break;

    case 'status':
      cli.status().catch(console.error);
      break;

    default:
      console.log('ALN Node CLI\n');
      console.log('Usage:');
      console.log('  aln init [--force]     Initialize node data directory');
      console.log('  aln start              Start the node');
      console.log('  aln status             Check node status\n');
      break;
  }
}

module.exports = AlnNodeCLI;
