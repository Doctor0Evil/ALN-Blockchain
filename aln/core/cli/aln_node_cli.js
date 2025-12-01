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
const { AugmentedPolicyEngine } = require('../../augmented_policy/policy_engine');
const { ReputationSystem } = require('../../reputation/reputation_system');
const { EnergyIntegration } = require('../../energy/energy_integration');
const {
  MAX_CHAT_CONTEXT_ID_LENGTH,
  MAX_TRANSCRIPT_HASH_LENGTH,
  MAX_JURISDICTION_TAGS,
  isValidJurisdiction
} = require('../config/constants');
const {
  buildTransferTx,
  buildGovernanceProposalTx,
  buildGovernanceVoteTx,
  withChatMetadata,
  signTx
} = require('../../wallet/tx_builder');
const { KeyCustodian } = require('../../security/key_custodian');

const DEFAULT_PROFILE_PATH = path.join(__dirname, '../config/nanotopologene_profile.aln');

class AlnNodeCLI {
  constructor() {
    this.profile = null;
    this.logger = new Logger('cli');
    this.policyEngine = null;
    this.energyIntegration = null;
    this.reputationSystem = null;
  }

  /**
   * Build transactions via CLI with chat metadata flags
   */
  async buildTx(argv = []) {
    if (!argv.length) {
      this.printTxHelp();
      return;
    }

    const [subcommand, ...rest] = argv;
    const flags = parseFlags(rest);

    try {
      let tx;
      switch (subcommand) {
        case 'transfer':
          ensureRequired(flags, ['from', 'to', 'amount', 'nonce']);
          tx = buildTransferTx(
            flags.from,
            flags.to,
            flags.amount,
            parseInt(flags.nonce, 10),
            flags.fee ? parseInt(flags.fee, 10) : 100
          );
          break;
        case 'governance-vote':
          ensureRequired(flags, ['from', 'proposal', 'support', 'nonce']);
          validateSupport(flags.support);
          tx = buildGovernanceVoteTx(
            flags.from,
            flags.proposal,
            flags.support,
            parseInt(flags.nonce, 10)
          );
          break;
        case 'governance-proposal':
          ensureRequired(flags, ['from', 'proposal-id', 'title', 'description', 'category', 'execution-route', 'nonce']);
          tx = buildGovernanceProposalTx(flags.from, {
            proposal_id: flags['proposal-id'],
            title: flags.title,
            description: flags.description,
            category: flags.category,
            execution_route: flags['execution-route'],
            quorum: flags.quorum ? parseFloat(flags.quorum) : undefined,
            threshold: flags.threshold ? parseFloat(flags.threshold) : undefined,
            duration_blocks: flags['duration-blocks'] ? parseInt(flags['duration-blocks'], 10) : undefined,
            constraints: flags.constraints ? flags.constraints.split(',').map(v => v.trim()).filter(Boolean) : [],
            nonce: parseInt(flags.nonce, 10)
          });
          break;
        default:
          this.printTxHelp();
          return;
      }

      // TODO(ci): Add CLI integration test to assert chat metadata flags are validated before tx serialization.
      const metadata = buildChatMetadata(flags);
      const augmentedTx = withChatMetadata(tx, metadata);
      const finalTx = await maybeSignWithCustodian(augmentedTx, flags);
      console.log(JSON.stringify(finalTx, null, 2));
    } catch (error) {
      console.error(`❌ ${error.message}`);
      process.exit(1);
    }
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

    this.reputationSystem = new ReputationSystem(stateStore);
    this.energyIntegration = new EnergyIntegration();
    this.policyEngine = new AugmentedPolicyEngine(this.reputationSystem, this.energyIntegration);

    // Initialize consensus
    console.log('⚙️  Initializing consensus engine...');
    const consensus = new SoloConsensus(stateStore, {
      blockTime: consensusConfig.blockTime,
      nodeId: this.profile.getNodeId(),
      policyEngine: this.policyEngine
    });

    await consensus.initialize();

    // Start HTTP API
    if (networkConfig.apiEnabled) {
      console.log('🌐 Starting HTTP API server...');
      const httpServer = new HttpServer(consensus, stateStore, {
        apiPort: networkConfig.apiPort,
        wsPort: networkConfig.wsPort,
        nodeId: this.profile.getNodeId(),
        policyEngine: this.policyEngine,
        energyIntegration: this.energyIntegration
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

    case 'tx':
      cli.buildTx(args.slice(1)).catch(console.error);
      break;

    default:
      console.log('ALN Node CLI\n');
      console.log('Usage:');
      console.log('  aln init [--force]     Initialize node data directory');
      console.log('  aln start              Start the node');
      console.log('  aln status             Check node status');
      console.log('  aln tx <type> [flags]  Build transaction with optional chat metadata\n');
      break;
  }
}

function parseFlags(argv) {
  const flags = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg.startsWith('--')) {
      const key = arg.replace(/^--/, '');
      const next = argv[i + 1];
      if (!next || next.startsWith('--')) {
        flags[key] = true;
      } else {
        flags[key] = next;
        i++;
      }
    }
  }
  return flags;
}

function ensureRequired(flags, keys) {
  const missing = keys.filter(key => !flags[key]);
  if (missing.length) {
    throw new Error(`Missing required flags: ${missing.join(', ')}`);
  }
}

function validateSupport(value) {
  const normalized = (value || '').toLowerCase();
  if (!['for', 'against', 'abstain'].includes(normalized)) {
    throw new Error('support must be one of: for, against, abstain');
  }
}

function buildChatMetadata(flags) {
  const metadata = {};

  if (flags['chat-context-id']) {
    if (flags['chat-context-id'].length > MAX_CHAT_CONTEXT_ID_LENGTH) {
      throw new Error(`chat-context-id exceeds ${MAX_CHAT_CONTEXT_ID_LENGTH} characters`);
    }
    metadata.chat_context_id = flags['chat-context-id'];
  }

  if (flags['transcript-hash']) {
    if (flags['transcript-hash'].length > MAX_TRANSCRIPT_HASH_LENGTH) {
      throw new Error(`transcript-hash exceeds ${MAX_TRANSCRIPT_HASH_LENGTH} characters`);
    }
    metadata.transcript_hash = flags['transcript-hash'];
  }

  if (flags['jurisdiction-tags']) {
    const tags = flags['jurisdiction-tags']
      .split(',')
      .map(tag => tag.trim())
      .filter(Boolean);

    if (tags.length > MAX_JURISDICTION_TAGS) {
      throw new Error(`jurisdiction-tags exceeds ${MAX_JURISDICTION_TAGS}`);
    }

    for (const tag of tags) {
      if (!isValidJurisdiction(tag)) {
        throw new Error(`Invalid jurisdiction tag: ${tag}`);
      }
    }

    metadata.jurisdiction_tags = tags;
  }

  return metadata;
}

async function maybeSignWithCustodian(tx, flags) {
  if (!flags['custodian-root'] || !flags['custodian-label']) {
    return tx;
  }

  // TODO(ci): Add automated CLI test covering custodian-backed signing once Node runtime is available in CI.
  const envKey = flags['custodian-passphrase-env'];
  if (!envKey || !process.env[envKey]) {
    throw new Error('custodian-passphrase-env must point to populated environment variable');
  }

  const custodianDir = path.isAbsolute(flags['custodian-root'])
    ? flags['custodian-root']
    : path.join(process.cwd(), flags['custodian-root']);

  const custodian = new KeyCustodian(custodianDir);
  await custodian.initialize();
  const keyBytes = await custodian.ensureKey(flags['custodian-label'], process.env[envKey]);
  signTx(tx, Buffer.from(keyBytes).toString('hex'));
  return tx;
}

AlnNodeCLI.prototype.printTxHelp = function printTxHelp() {
  console.log('\nTransaction Builder Usage:');
  console.log('  aln tx transfer --from <addr> --to <addr> --amount <amt> --nonce <n> [--fee <gas_price>] [metadata flags]');
  console.log('  aln tx governance-vote --from <addr> --proposal <id> --support <for|against|abstain> --nonce <n> [metadata flags]');
  console.log('  aln tx governance-proposal --from <addr> --proposal-id <id> --title "..." --description "..." --category <type> --execution-route <route> --nonce <n> [metadata flags]\n');
  console.log('Metadata flags:');
  console.log('  --chat-context-id <uuid>');
  console.log('  --transcript-hash <sha256hex>');
  console.log('  --jurisdiction-tags tag1,tag2');
  console.log('  --constraints tagA,tagB (governance-proposal only)');
  console.log('\nCustodian options:');
  console.log('  --custodian-root <dir> --custodian-label <name> --custodian-passphrase-env <ENV_VAR>');
  console.log('    (Automatically encrypts keys on disk and signs output)');
};

module.exports = AlnNodeCLI;
