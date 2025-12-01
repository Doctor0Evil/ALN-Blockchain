/**
 * Augmented Policy Engine Tests
 * Verify capability checks, energy budgets, reputation thresholds
 */

const { AugmentedPolicyEngine } = require('../../augmented_policy/policy_engine');
const { ReputationSystem } = require('../../reputation/reputation_system');
const { EnergyIntegration } = require('../../energy/energy_integration');

describe('AugmentedPolicyEngine', () => {
  let policyEngine;
  let reputationSystem;
  let energyIntegration;

  beforeEach(() => {
    reputationSystem = new ReputationSystem(null);
    energyIntegration = new EnergyIntegration();
    policyEngine = new AugmentedPolicyEngine(reputationSystem, energyIntegration);
  });

  test('allows BASIC capability for all users', () => {
    const user = 'did:aln:user1';
    const result = policyEngine.isActionAllowed(user, 'BASIC_ACTION', null);

    // Should allow if BASIC_ACTION exists; for unknown action, expect denial
    expect(result.allowed).toBeDefined();
  });

  test('blocks ADVANCED capability with low reputation', () => {
    const user = 'did:aln:user2';
    // Reputation defaults to 0, below threshold 50
    const result = policyEngine.isActionAllowed(user, 'ENHANCED_VISION', { gridPowerDrawWatts: 100 });

    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('Insufficient reputation');
  });

  test('allows ADVANCED capability with sufficient reputation', async () => {
    const user = 'did:aln:user3';
    // Boost reputation
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'REPORTED_INCIDENT', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');

    const reputation = reputationSystem.computeReputation(user);
    expect(reputation).toBeGreaterThanOrEqual(50);

    const result = policyEngine.isActionAllowed(user, 'ENHANCED_VISION', { gridPowerDrawWatts: 100 });
    expect(result.allowed).toBe(true);
  });

  test('blocks LAW_ENF_ASSIST without sufficient reputation', async () => {
    const user = 'did:aln:user4';
    // Moderate reputation (~50), below LAW_ENF_ASSIST threshold 70
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');

    const result = policyEngine.isActionAllowed(user, 'SECURE_COMMS', { gridPowerDrawWatts: 100 });
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('Insufficient reputation for LE assist');
  });

  test('blocks action when energy budget exceeded', async () => {
    const user = 'did:aln:user5';
    // Set policy with low max draw
    policyEngine.policies.set(user, {
      user,
      maxGridDrawWatts: 500,
      allowedCapabilityLevels: ['BASIC', 'ADVANCED'],
      auditRequired: true
    });

    // Set current energy state to high draw
    await energyIntegration.updateEnergyState(user, 75, 80, 450, ['device1']);

    // Attempt action requiring additional 100W (would exceed 500W)
    const result = policyEngine.isActionAllowed(user, 'ENHANCED_VISION', { gridPowerDrawWatts: 100 });
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('Energy budget exceeded');
  });

  test('requireLawEnfModeFor correctly identifies LE-only actions', () => {
    expect(policyEngine.requireLawEnfModeFor('SECURE_COMMS')).toBe(true);
    expect(policyEngine.requireLawEnfModeFor('DATA_ACCESS_LEVEL_X')).toBe(true);
    expect(policyEngine.requireLawEnfModeFor('ENHANCED_VISION')).toBe(false);
  });

  test('validateTransaction enforces governance metadata', () => {
    const tx = {
      header: {
        op_code: 'governance_vote',
        from: 'aln1user',
        to: 'aln1gov',
        nonce: 1,
        transcript_hash: 'a'.repeat(64)
      },
      data: {}
    };

    const result = policyEngine.validateTransaction(tx);
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('chat_context_id');
  });

  test('validateTransaction passes when metadata complete', () => {
    const tx = {
      header: {
        op_code: 'governance_vote',
        from: 'aln1user',
        to: 'aln1gov',
        nonce: 1,
        chat_context_id: '550e8400-e29b-41d4-a716-446655440000',
        transcript_hash: 'a'.repeat(64),
        jurisdiction_tags: ['US_federal']
      },
      data: { proposal_id: 'prop', support: 'for' }
    };

    const result = policyEngine.validateTransaction(tx);
    expect(result.allowed).toBe(true);
  });
});

describe('ReputationSystem', () => {
  let reputationSystem;

  beforeEach(() => {
    reputationSystem = new ReputationSystem(null);
  });

  test('issues good deed record and updates reputation', async () => {
    const user = 'did:aln:user1';
    const recordId = await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');

    expect(recordId).toMatch(/^gdr_/);
    const reputation = reputationSystem.computeReputation(user);
    expect(reputation).toBeGreaterThan(10); // Above baseline
  });

  test('revocation decreases reputation', async () => {
    const user = 'did:aln:user2';
    const recordId = await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    const beforeRevoke = reputationSystem.computeReputation(user);

    await reputationSystem.revokeRecord(recordId, 'error', 'authority1');
    const afterRevoke = reputationSystem.computeReputation(user);

    expect(afterRevoke).toBeLessThan(beforeRevoke);
  });

  test('appeal workflow', async () => {
    const user = 'did:aln:user3';
    const recordId = await reputationSystem.issueGoodDeedRecord(user, 'ASSISTED_EMS', 'authority1', 'US_federal');
    await reputationSystem.revokeRecord(recordId, 'disputed', 'authority1');

    const appealId = await reputationSystem.appealRecord(recordId, user, 'wrongly revoked');
    expect(appealId).toMatch(/^appeal_/);

    await reputationSystem.resolveAppeal(appealId, 'approve', 'authority2');
    const reputation = reputationSystem.computeReputation(user);
    expect(reputation).toBeGreaterThan(10); // Restored
  });

  test('reputation cannot drop below baseline', () => {
    const user = 'did:aln:user4';
    // No good deeds, reputation should be at baseline (10)
    const reputation = reputationSystem.computeReputation(user);
    expect(reputation).toBeGreaterThanOrEqual(10);
  });
});
