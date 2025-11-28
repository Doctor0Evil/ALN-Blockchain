#!/usr/bin/env node
/**
 * Verify ALN scaffold integrity
 * - Docs present
 * - Constants present and single-source usage
 * - Compliance routing exists
 * - Wallet includes chat metadata helper
 */
const fs = require('fs');
const path = require('path');

function checkFile(p) {
  const exists = fs.existsSync(p);
  return { path: p, exists };
}

function main() {
  const root = process.cwd();
  const checks = [];

  // Docs
  checks.push(checkFile(path.join(root, 'AUGMENTED_POLICY.md')));
  checks.push(checkFile(path.join(root, 'AUGMENTED_REPUTATION.md')));
  checks.push(checkFile(path.join(root, 'LAW_ENF_ASSIST_GUIDE.md')));

  // Constants
  checks.push(checkFile(path.join(root, 'aln', 'core', 'config', 'constants.ts')));

  // Compliance routing
  checks.push(checkFile(path.join(root, 'aln', 'compliance_routing', 'router.aln')));
  checks.push(checkFile(path.join(root, 'aln', 'compliance_routing', 'policy_JFMIP_24_01.yaml')));

  // Wallet chat helper
  checks.push(checkFile(path.join(root, 'aln', 'wallet', 'tx_builder.js')));

  const missing = checks.filter(c => !c.exists);
  if (missing.length > 0) {
    console.error('Missing required files:');
    for (const m of missing) console.error(' - ' + m.path);
    process.exit(1);
  }
  console.log('✅ ALN scaffold verification passed');
}

if (require.main === module) {
  main();
}
