/**
 * Token Creation Pipeline Tests
 * Verify malicious blueprints blocked, economic/naming constraints enforced
 */

const { TokenCreationPipeline } = require('../../token_creation/pipeline');
const { MLThreatHooks } = require('../../security/ml_hooks');

describe('TokenCreationPipeline', () => {
  let pipeline;

  beforeEach(() => {
    pipeline = new TokenCreationPipeline(null);
  });

  test('rejects blueprint with insufficient fee', async () => {
    const submitter = 'aln1test000000000000000000000000000000000';
    const blueprint = {
      name: 'TestToken',
      symbol: 'TST',
      code: 'contract TestToken {}',
      fee: 500 // Below minimum 1000
    };

    await expect(pipeline.submitTokenBlueprint(submitter, blueprint))
      .rejects.toThrow('Insufficient fee');
  });

  test('blocks impersonation in token name', async () => {
    const submitter = 'aln1test000000000000000000000000000000000';
    const blueprint = {
      name: 'US Treasury Token',
      symbol: 'USDT',
      code: 'contract USTreasury {}',
      fee: 1000
    };

    const blueprintId = await pipeline.submitTokenBlueprint(submitter, blueprint);
    const report = await pipeline.analyzeAndRefactor(blueprintId);

    expect(report.safe).toBe(false);
    expect(report.rejectedReasons).toContain('Prohibited term in name: US Treasury');
  });

  test('auto-refactors selfdestruct pattern', async () => {
    const submitter = 'aln1test000000000000000000000000000000000';
    const blueprint = {
      name: 'SafeToken',
      symbol: 'SAFE',
      code: 'contract Token { function drain() { selfdestruct(admin); } }',
      fee: 1000
    };

    const blueprintId = await pipeline.submitTokenBlueprint(submitter, blueprint);
    const report = await pipeline.analyzeAndRefactor(blueprintId);

    expect(report.transformedCode).toContain('/* REMOVED: selfdestruct */');
    expect(report.changes).toContain('Removed selfdestruct call');
  });

  test('accepts clean blueprint', async () => {
    const submitter = 'aln1test000000000000000000000000000000000';
    const blueprint = {
      name: 'GoodToken',
      symbol: 'GOOD',
      code: 'contract GoodToken { mapping(address => uint256) balances; }',
      fee: 1000
    };

    const blueprintId = await pipeline.submitTokenBlueprint(submitter, blueprint);
    const report = await pipeline.analyzeAndRefactor(blueprintId);

    expect(report.safe).toBe(true);
    expect(report.rejectedReasons).toHaveLength(0);
  });

  test('enforces approval before deployment', async () => {
    const submitter = 'aln1test000000000000000000000000000000000';
    const blueprint = {
      name: 'Token',
      symbol: 'TKN',
      code: 'contract Token {}',
      fee: 1000
    };

    const blueprintId = await pipeline.submitTokenBlueprint(submitter, blueprint);
    await pipeline.analyzeAndRefactor(blueprintId);

    // Attempt deploy without approval
    await expect(pipeline.deployToken(blueprintId))
      .rejects.toThrow('Blueprint not approved for deployment');

    // Approve and deploy
    await pipeline.approveOrReject(blueprintId, 'approve', 'dao_reviewer');
    const result = await pipeline.deployToken(blueprintId);

    expect(result.success).toBe(true);
  });
});

describe('MLThreatHooks', () => {
  let mlHooks;

  beforeEach(async () => {
    mlHooks = new MLThreatHooks();
    await mlHooks.loadSignatures();
  });

  test('detects selfdestruct drainer pattern', () => {
    const code = 'function malicious() { selfdestruct(attacker); }';
    const score = mlHooks.scorePayload(code);

    expect(score.score).toBeGreaterThan(0);
    expect(score.domain).toBe('drainer');
  });

  test('detects delegatecall supply chain risk', () => {
    const code = 'contract Proxy { function exec(address target) { target.delegatecall(data); } }';
    const score = mlHooks.scorePayload(code);

    expect(score.score).toBeGreaterThan(0);
    expect(score.domain).toBe('supply_chain');
  });

  test('clean code scores zero', () => {
    const code = 'contract Safe { uint256 balance; function deposit() payable {} }';
    const score = mlHooks.scorePayload(code);

    expect(score.score).toBe(0);
    expect(score.domain).toBe('none');
  });
});
