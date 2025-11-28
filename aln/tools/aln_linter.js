/**
 * ALN Chainlexeme Linter
 * 
 * Validates ALN documents for style, correctness, and best practices
 */

const fs = require('fs');
const path = require('path');
const { parseAlnDocument, validateChainlexemes } = require('../core/runtime/aln_parser');

class AlnLinter {
  constructor() {
    this.errors = [];
    this.warnings = [];
  }

  /**
   * Lint ALN file
   * @param {string} filePath - Path to ALN file
   */
  lintFile(filePath) {
    console.log(`\n📋 Linting: ${filePath}`);
    
    if (!fs.existsSync(filePath)) {
      this.errors.push(`File not found: ${filePath}`);
      return;
    }

    const content = fs.readFileSync(filePath, 'utf8');
    this.lintContent(content, filePath);
  }

  /**
   * Lint ALN content
   * @param {string} content - ALN document content
   * @param {string} source - Source identifier
   */
  lintContent(content, source = 'content') {
    this.errors = [];
    this.warnings = [];

    // Check 1: Parse document
    const parsed = parseAlnDocument(content);
    if (parsed.errors && parsed.errors.length > 0) {
      this.errors.push(...parsed.errors);
    }

    // Check 2: Validate structure
    const validation = validateChainlexemes(parsed);
    if (!validation.isValid) {
      this.errors.push(...validation.errors.map(e => e.message || e));
    }
    if (validation.warnings) {
      this.warnings.push(...validation.warnings.map(w => w.message || w));
    }

    // Check 3: Field ordering (header should come before data, data before footer)
    const sections = Object.keys(parsed.rawSections || {});
    const expectedOrder = ['header', 'data', 'footer'];
    let lastIndex = -1;
    for (const section of sections) {
      const index = expectedOrder.indexOf(section);
      if (index !== -1 && index < lastIndex) {
        this.warnings.push(`Section [${section}] should come after [${expectedOrder[lastIndex]}]`);
      }
      if (index > lastIndex) lastIndex = index;
    }

    // Check 4: Field naming conventions
    if (parsed.header && parsed.header.op_code) {
      const validOpCodes = [
        'transfer', 'governance_proposal', 'governance_vote',
        'migration_lock', 'migration_mint', 'migration_burn',
        'token_mint', 'token_transfer', 'delegation'
      ];
      if (!validOpCodes.includes(parsed.header.op_code)) {
        this.errors.push(`Invalid op_code: ${parsed.header.op_code}`);
      }
    }

    // Check 5: Mixed operation types (governance and financial should be separate)
    if (parsed.header && parsed.header.op_code) {
      const isGovernance = parsed.header.op_code.startsWith('governance_');
      const hasAmount = parsed.data && 'amount' in parsed.data;
      
      if (isGovernance && hasAmount && parsed.header.op_code !== 'governance_proposal') {
        this.warnings.push('Mixing governance and transfer operations is discouraged');
      }
    }

    // Check 6: Numeric bounds
    if (parsed.data && parsed.data.amount) {
      const amount = BigInt(parsed.data.amount);
      if (amount < 0n) {
        this.errors.push('Amount cannot be negative');
      }
    }

    // Check 7: Timestamp in reasonable range
    if (parsed.footer && parsed.footer.timestamp) {
      const now = Math.floor(Date.now() / 1000);
      const timestamp = parsed.footer.timestamp;
      if (Math.abs(timestamp - now) > 86400 * 365) {
        this.warnings.push('Timestamp is more than 1 year from current time');
      }
    }

    // Report results
    if (this.errors.length === 0 && this.warnings.length === 0) {
      console.log(`✅ No issues found\n`);
    } else {
      if (this.errors.length > 0) {
        console.log(`\n❌ Errors (${this.errors.length}):`);
        this.errors.forEach((err, i) => {
          console.log(`  ${i + 1}. ${err}`);
        });
      }
      if (this.warnings.length > 0) {
        console.log(`\n⚠️  Warnings (${this.warnings.length}):`);
        this.warnings.forEach((warn, i) => {
          console.log(`  ${i + 1}. ${warn}`);
        });
      }
      console.log();
    }
  }

  /**
   * Lint directory recursively
   * @param {string} dirPath - Directory path
   */
  lintDirectory(dirPath) {
    console.log(`\n📁 Linting directory: ${dirPath}\n`);
    
    const files = this._findAlnFiles(dirPath);
    
    if (files.length === 0) {
      console.log('No .aln files found\n');
      return;
    }

    let totalErrors = 0;
    let totalWarnings = 0;

    files.forEach(file => {
      this.lintFile(file);
      totalErrors += this.errors.length;
      totalWarnings += this.warnings.length;
    });

    console.log(`\n📊 Summary:`);
    console.log(`  Files checked: ${files.length}`);
    console.log(`  Total errors: ${totalErrors}`);
    console.log(`  Total warnings: ${totalWarnings}\n`);
  }

  /**
   * Find all .aln files in directory
   * @param {string} dirPath - Directory path
   * @returns {Array<string>} Array of file paths
   */
  _findAlnFiles(dirPath) {
    const files = [];
    
    const entries = fs.readdirSync(dirPath, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(dirPath, entry.name);
      
      if (entry.isDirectory()) {
        files.push(...this._findAlnFiles(fullPath));
      } else if (entry.isFile() && entry.name.endsWith('.aln')) {
        files.push(fullPath);
      }
    }
    
    return files;
  }
}

// CLI entry point
if (require.main === module) {
  const linter = new AlnLinter();
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log('ALN Linter\n');
    console.log('Usage:');
    console.log('  node aln_linter.js <file.aln>         Lint single file');
    console.log('  node aln_linter.js <directory>        Lint all .aln files in directory\n');
    process.exit(0);
  }

  const target = args[0];
  const stat = fs.statSync(target);

  if (stat.isDirectory()) {
    linter.lintDirectory(target);
  } else {
    linter.lintFile(target);
  }
}

module.exports = AlnLinter;
