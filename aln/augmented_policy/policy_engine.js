/**
 * Augmented User Policy Engine
 * Enforces capability checks, energy budgets, and compliance modes
 */

const {
  MAX_CHAT_CONTEXT_ID_LENGTH,
  MAX_TRANSCRIPT_HASH_LENGTH,
  MAX_JURISDICTION_TAGS,
  isValidJurisdiction
} = require('../core/config/constants');

class AugmentedPolicyEngine {
  constructor(reputationSystem, energyIntegration) {
    this.reputationSystem = reputationSystem;
    this.energyIntegration = energyIntegration;
    this.policies = new Map();
    this.profiles = new Map();
    this.actionRegistry = new Map([
      ['ENHANCED_VISION', { id: 'ENHANCED_VISION', capabilityLevel: 'ADVANCED' }],
      ['RAPID_MOBILITY', { id: 'RAPID_MOBILITY', capabilityLevel: 'ADVANCED' }],
      ['SECURE_COMMS', { id: 'SECURE_COMMS', capabilityLevel: 'LAW_ENF_ASSIST' }],
      ['DATA_ACCESS_LEVEL_X', { id: 'DATA_ACCESS_LEVEL_X', capabilityLevel: 'LAW_ENF_ASSIST' }]
    ]);
  }

  /**
   * Validate transaction metadata requirements before consensus admission
   */
  validateTransaction(chainlexeme) {
    if (!chainlexeme || !chainlexeme.header) {
      return { allowed: false, reason: 'Missing chainlexeme header' };
    }

    const { header } = chainlexeme;

    const metaCheck = this._validateMetadata(header);
    if (!metaCheck.allowed) {
      return metaCheck;
    }

    switch (header.op_code) {
      case 'governance_vote':
      case 'governance_proposal':
        if (!header.chat_context_id) {
          return { allowed: false, reason: 'Governance transactions require chat_context_id' };
        }
        if (!header.transcript_hash) {
          return { allowed: false, reason: 'Governance transactions require transcript_hash' };
        }
        break;
      case 'migration_ingest':
        if (!header.transcript_hash) {
          return { allowed: false, reason: 'Migration ingest requires transcript_hash' };
        }
        break;
      default:
        break;
    }

    return { allowed: true };
  }

  _validateMetadata(header) {
    if (header.chat_context_id && header.chat_context_id.length > MAX_CHAT_CONTEXT_ID_LENGTH) {
      return { allowed: false, reason: 'chat_context_id exceeds allowed length' };
    }

    if (header.transcript_hash && header.transcript_hash.length > MAX_TRANSCRIPT_HASH_LENGTH) {
      return { allowed: false, reason: 'transcript_hash exceeds allowed length' };
    }

    if (header.jurisdiction_tags) {
      if (!Array.isArray(header.jurisdiction_tags)) {
        return { allowed: false, reason: 'jurisdiction_tags must be array' };
      }

      if (header.jurisdiction_tags.length > MAX_JURISDICTION_TAGS) {
        return { allowed: false, reason: 'jurisdiction_tags exceeds allowed length' };
      }

      for (const tag of header.jurisdiction_tags) {
        if (!isValidJurisdiction(tag)) {
          return { allowed: false, reason: `Invalid jurisdiction tag ${tag}` };
        }
      }
    }

    return { allowed: true };
  }

  /**
   * Get effective policy for user
   */
  getEffectivePolicy(user) {
    // TODO: Apply jurisdiction overlays
    return this.policies.get(user) || {
      user,
      maxGridDrawWatts: 1000,
      maxDeviceClassPermitted: ['BASIC'],
      allowedCapabilityLevels: ['BASIC'],
      auditRequired: false,
      jurisdictionConstraints: []
    };
  }

  /**
   * Check if action is allowed
   */
  isActionAllowed(user, actionId, energyState) {
    const action = this.actionRegistry.get(actionId);
    if (!action) return { allowed: false, reason: 'Unknown action' };

    const policy = this.getEffectivePolicy(user);
    const reputation = this.reputationSystem.computeReputation(user);

    // Check capability level
    if (!policy.allowedCapabilityLevels.includes(action.capabilityLevel)) {
      return { allowed: false, reason: 'Capability level not permitted' };
    }

    // Check reputation threshold for advanced capabilities
    if (action.capabilityLevel === 'ADVANCED' && reputation < 50) {
      return { allowed: false, reason: 'Insufficient reputation' };
    }
    if (action.capabilityLevel === 'LAW_ENF_ASSIST' && reputation < 70) {
      return { allowed: false, reason: 'Insufficient reputation for LE assist' };
    }

    // Check energy budget
    const energyCheck = this.energyIntegration.checkEnergyBudget(user, energyState?.gridPowerDrawWatts || 0);
    if (!energyCheck) {
      return { allowed: false, reason: 'Energy budget exceeded' };
    }

    // Emit audit event
    this.emitActionAudit(user, actionId, true, energyState, policy);

    return { allowed: true, reason: '' };
  }

  /**
   * Check if action requires law enforcement mode
   */
  requireLawEnfModeFor(actionId) {
    const action = this.actionRegistry.get(actionId);
    return action?.capabilityLevel === 'LAW_ENF_ASSIST';
  }

  /**
   * Emit action audit event
   */
  emitActionAudit(user, actionId, allowed, energyState, policy) {
    // TODO: Emit on-chain event
    const audit = {
      user,
      actionId,
      allowed,
      energyBefore: energyState?.cognitiveLevel || 0,
      timestamp: Date.now(),
      policyVersion: policy?.version || '1.0'
    };
    console.log('AUDIT:', JSON.stringify(audit));
  }
}

module.exports = { AugmentedPolicyEngine };
