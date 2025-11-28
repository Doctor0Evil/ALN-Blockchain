/**
 * Nanotopologene Profile Loader
 * 
 * Parses and validates nanotopologene profile configuration
 */

const fs = require('fs');
const path = require('path');
const { parseAlnDocument } = require('../runtime/aln_parser');

class NanotopologeneProfile {
  constructor(data = {}) {
    this.node_identity = data.node_identity || {};
    this.capabilities = data.capabilities || {};
    this.topology_matrix = data.topology_matrix || {};
    this.compliance = data.compliance || {};
    this.network = data.network || {};
    this.consensus = data.consensus || {};
    this.storage = data.storage || {};
    this.security = data.security || {};
    this.qpu_math_plus = data.qpu_math_plus || {};
    this.governance_parameters = data.governance_parameters || {};
    this.migration_parameters = data.migration_parameters || {};
    this.logging = data.logging || {};
  }

  /**
   * Get node ID
   */
  getNodeId() {
    return this.node_identity.node_id || 'unknown_node';
  }

  /**
   * Get operations threshold
   */
  getOpsThreshold() {
    return parseInt(this.capabilities.ops_threshold_TOPS || 1000);
  }

  /**
   * Check if governance is supported
   */
  supportsGovernance() {
    return this.capabilities.supports_governance === 'true' || this.capabilities.supports_governance === true;
  }

  /**
   * Check if migration is supported
   */
  supportsMigration() {
    return this.capabilities.supports_migration === 'true' || this.capabilities.supports_migration === true;
  }

  /**
   * Get compliance level
   */
  getComplianceLevel() {
    return this.compliance.compliance_level || 'standard';
  }

  /**
   * Get AI firmware version
   */
  getFirmwareVersion() {
    return this.compliance.ai_firmware_version || 'ALN.QPU.Math+';
  }

  /**
   * Get network configuration
   */
  getNetworkConfig() {
    return {
      apiPort: parseInt(this.network.api_port || 3000),
      wsPort: parseInt(this.network.ws_port || 3001),
      rpcPort: parseInt(this.network.rpc_port || 26657),
      p2pPort: parseInt(this.network.p2p_port || 26656),
      apiEnabled: this.network.api_enabled === 'true' || this.network.api_enabled === true,
      wsEnabled: this.network.ws_enabled === 'true' || this.network.ws_enabled === true
    };
  }

  /**
   * Get consensus configuration
   */
  getConsensusConfig() {
    return {
      mode: this.consensus.consensus_mode || 'solo',
      blockTime: parseInt(this.consensus.block_time_ms || 5000),
      minGasPrice: parseInt(this.consensus.min_gas_price || 1),
      maxGasPerBlock: parseInt(this.consensus.max_gas_per_block || 10000000)
    };
  }

  /**
   * Get storage configuration
   */
  getStorageConfig() {
    return {
      backend: this.storage.db_backend || 'leveldb',
      dataDir: this.storage.data_dir || './data',
      stateDir: this.storage.state_dir || './data/state',
      blocksDir: this.storage.blocks_dir || './data/blocks'
    };
  }

  /**
   * Get QPU.Math+ configuration
   */
  getQPUConfig() {
    return {
      enableConservation: this.qpu_math_plus.enable_conservation_checks !== 'false',
      enableLimits: this.qpu_math_plus.enable_limit_checks !== 'false',
      enableSignatureVerification: this.qpu_math_plus.enable_signature_verification !== 'false',
      maxAmountPerTx: this.qpu_math_plus.max_amount_per_tx || '1000000000000000000000000',
      maxDailyLimit: this.qpu_math_plus.max_daily_limit || '10000000000000000000000'
    };
  }

  /**
   * Validate profile
   */
  validate() {
    const errors = [];

    // Check required fields
    if (!this.node_identity.node_id) {
      errors.push('Missing node_id in node_identity');
    }

    if (!this.capabilities.ops_threshold_TOPS) {
      errors.push('Missing ops_threshold_TOPS in capabilities');
    }

    if (!this.compliance.compliance_level) {
      errors.push('Missing compliance_level in compliance');
    }

    // Validate ops threshold
    const opsThreshold = this.getOpsThreshold();
    if (opsThreshold < 100) {
      errors.push('ops_threshold_TOPS must be at least 100');
    }

    // Validate network ports
    const networkConfig = this.getNetworkConfig();
    if (networkConfig.apiPort < 1024 || networkConfig.apiPort > 65535) {
      errors.push('api_port must be between 1024 and 65535');
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }
}

/**
 * Load nanotopologene profile from ALN file
 * @param {string} filePath - Path to profile file
 * @returns {NanotopologeneProfile} Loaded profile
 */
function loadProfile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  const parsed = parseAlnDocument(content);

  // Extract sections (they're stored in parsed.data since sections become key-value pairs)
  const profileData = {
    node_identity: {},
    capabilities: {},
    topology_matrix: {},
    compliance: {},
    network: {},
    consensus: {},
    storage: {},
    security: {},
    qpu_math_plus: {},
    governance_parameters: {},
    migration_parameters: {},
    logging: {}
  };

  // Parse raw sections to reconstruct structure
  // Note: This is simplified - in production, improve parsing
  const allData = { ...parsed.header, ...parsed.data, ...parsed.footer };
  
  for (const [key, value] of Object.entries(allData)) {
    // Assign to appropriate section based on key prefix/context
    if (key.startsWith('node_id') || key.startsWith('profile_')) {
      profileData.node_identity[key] = value;
    } else if (key.startsWith('ops_') || key.startsWith('max_') || key.startsWith('supports_')) {
      profileData.capabilities[key] = value;
    } else if (key.includes('compliance') || key.includes('ai_firmware')) {
      profileData.compliance[key] = value;
    } else if (key.includes('port') || key.includes('p2p') || key.includes('rpc') || key.includes('api') || key.includes('ws')) {
      profileData.network[key] = value;
    } else if (key.includes('consensus') || key.includes('block_time') || key.includes('gas')) {
      profileData.consensus[key] = value;
    } else if (key.includes('db_') || key.includes('_dir')) {
      profileData.storage[key] = value;
    } else if (key.includes('log')) {
      profileData.logging[key] = value;
    }
  }

  return new NanotopologeneProfile(profileData);
}

/**
 * Load profile with defaults
 * @param {string} filePath - Path to profile file (optional)
 * @returns {NanotopologeneProfile} Profile with defaults
 */
function loadProfileWithDefaults(filePath = null) {
  if (filePath && fs.existsSync(filePath)) {
    return loadProfile(filePath);
  }

  // Return default profile
  return new NanotopologeneProfile({
    node_identity: {
      node_id: 'node_001',
      profile_version: '1.0'
    },
    capabilities: {
      ops_threshold_TOPS: 1000,
      max_block_size_bytes: 2097152,
      max_tx_per_block: 1000,
      supports_governance: true,
      supports_migration: true
    },
    compliance: {
      compliance_level: 'surgical_grade',
      ai_firmware_version: 'ALN.QPU.Math+'
    },
    network: {
      api_port: 3000,
      ws_port: 3001,
      rpc_port: 26657,
      api_enabled: true,
      ws_enabled: true
    },
    consensus: {
      consensus_mode: 'solo',
      block_time_ms: 5000
    },
    storage: {
      db_backend: 'leveldb',
      data_dir: './data',
      state_dir: './data/state'
    }
  });
}

module.exports = {
  NanotopologeneProfile,
  loadProfile,
  loadProfileWithDefaults
};
