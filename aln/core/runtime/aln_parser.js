/**
 * ALN Parser and Validator
 * 
 * Parses ALN documents (chainlexemes) and validates their structure
 * according to /aln/core/spec/aln-syntax.aln specification.
 */

class ParsedAlnDoc {
  constructor() {
    this.header = {};
    this.data = {};
    this.footer = {};
    this.rawSections = {};
    this.isValid = false;
    this.errors = [];
  }
}

class ValidationReport {
  constructor() {
    this.isValid = true;
    this.errors = [];
    this.warnings = [];
  }

  addError(message, line = null) {
    this.errors.push({ message, line });
    this.isValid = false;
  }

  addWarning(message, line = null) {
    this.warnings.push({ message, line });
  }
}

/**
 * Parse ALN document text into structured format
 * @param {string} text - Raw ALN document
 * @returns {ParsedAlnDoc} Parsed document structure
 */
function parseAlnDocument(text) {
  const doc = new ParsedAlnDoc();
  
  if (!text || typeof text !== 'string') {
    doc.errors.push('Invalid input: text must be non-empty string');
    return doc;
  }

  const lines = text.split('\n');
  let currentSection = null;
  let lineNumber = 0;

  for (const line of lines) {
    lineNumber++;
    const trimmed = line.trim();

    // Skip empty lines and comments
    if (!trimmed || trimmed.startsWith('#')) {
      continue;
    }

    // Check for section headers
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
      currentSection = trimmed.slice(1, -1).toLowerCase();
      doc.rawSections[currentSection] = [];
      continue;
    }

    // Parse key-value pairs
    const colonIndex = trimmed.indexOf(':');
    if (colonIndex > 0 && currentSection) {
      const key = trimmed.slice(0, colonIndex).trim();
      const value = trimmed.slice(colonIndex + 1).trim();

      // Store in appropriate section
      if (currentSection === 'header') {
        doc.header[key] = parseValue(value);
      } else if (currentSection === 'data') {
        doc.data[key] = parseValue(value);
      } else if (currentSection === 'footer') {
        doc.footer[key] = parseValue(value);
      }

      doc.rawSections[currentSection].push({ key, value, line: lineNumber });
    }
  }

  return doc;
}

/**
 * Parse value string into appropriate JavaScript type
 * @param {string} value - String value to parse
 * @returns {*} Parsed value (string, number, boolean, array)
 */
function parseValue(value) {
  // Remove quotes
  if ((value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))) {
    return value.slice(1, -1);
  }

  // Parse boolean
  if (value === 'true') return true;
  if (value === 'false') return false;

  // Parse number
  if (/^\d+$/.test(value)) {
    return parseInt(value, 10);
  }
  if (/^\d+\.\d+$/.test(value)) {
    return parseFloat(value);
  }

  // Parse array
  if (value.startsWith('[') && value.endsWith(']')) {
    const items = value.slice(1, -1).split(',').map(item => item.trim());
    return items.map(item => parseValue(item));
  }

  // Default: return as string
  return value;
}

/**
 * Validate parsed chainlexeme according to ALN specification
 * @param {ParsedAlnDoc} doc - Parsed ALN document
 * @returns {ValidationReport} Validation results
 */
function validateChainlexemes(doc) {
  const report = new ValidationReport();

  // Rule 1: Check required sections
  if (!doc.header || Object.keys(doc.header).length === 0) {
    report.addError('Missing required [header] section');
  }
  if (!doc.data || Object.keys(doc.data).length === 0) {
    report.addError('Missing required [data] section');
  }
  if (!doc.footer || Object.keys(doc.footer).length === 0) {
    report.addError('Missing required [footer] section');
  }

  // Rule 2: Validate header fields
  const requiredHeaderFields = ['op_code', 'from', 'to', 'nonce'];
  for (const field of requiredHeaderFields) {
    if (!(field in doc.header)) {
      report.addError(`Missing required header field: ${field}`);
    }
  }

  // Rule 3: Validate op_code
  const validOpCodes = [
    'transfer', 'governance_proposal', 'governance_vote',
    'migration_lock', 'migration_mint', 'migration_burn',
    'token_mint', 'token_transfer', 'delegation'
  ];
  if (doc.header.op_code && !validOpCodes.includes(doc.header.op_code)) {
    report.addError(`Invalid op_code: ${doc.header.op_code}`);
  }

  // Rule 4: Validate address format (aln1...)
  if (doc.header.from && !doc.header.from.startsWith('aln1')) {
    report.addError(`Invalid from address format: must start with 'aln1'`);
  }
  if (doc.header.to && !doc.header.to.startsWith('aln1')) {
    report.addError(`Invalid to address format: must start with 'aln1'`);
  }

  // Rule 5: Validate nonce
  if (doc.header.nonce !== undefined) {
    if (typeof doc.header.nonce !== 'number' || doc.header.nonce < 0) {
      report.addError('Nonce must be non-negative integer');
    }
  }

  // Rule 6: Validate footer fields
  const requiredFooterFields = ['signature', 'timestamp'];
  for (const field of requiredFooterFields) {
    if (!(field in doc.footer)) {
      report.addError(`Missing required footer field: ${field}`);
    }
  }

  // Rule 7: Validate signature format
  if (doc.footer.signature && !doc.footer.signature.startsWith('ed25519:')) {
    report.addWarning('Signature should use ed25519: prefix');
  }

  // Rule 8: Validate timestamp
  if (doc.footer.timestamp !== undefined) {
    if (typeof doc.footer.timestamp !== 'number' || doc.footer.timestamp <= 0) {
      report.addError('Timestamp must be positive integer (Unix timestamp)');
    }
  }

  // Rule 9: Validate gas fields
  if (doc.footer.gas_limit !== undefined && doc.footer.gas_limit < 21000) {
    report.addWarning('Gas limit below minimum (21000)');
  }

  // Rule 10: Op-code specific validation
  if (doc.header.op_code === 'transfer') {
    if (!doc.data.amount) {
      report.addError('Transfer requires amount in data section');
    }
  }

  if (doc.header.op_code === 'governance_proposal') {
    const requiredFields = ['proposal_id', 'title', 'category'];
    for (const field of requiredFields) {
      if (!doc.data[field]) {
        report.addError(`Governance proposal requires ${field} in data section`);
      }
    }
  }

  return report;
}

/**
 * Serialize ALN document back to text format
 * @param {ParsedAlnDoc} doc - Parsed document
 * @returns {string} ALN text format
 */
function serializeAlnDocument(doc) {
  let output = '[header]\n';
  for (const [key, value] of Object.entries(doc.header)) {
    output += `${key}: ${formatValue(value)}\n`;
  }

  output += '\n[data]\n';
  for (const [key, value] of Object.entries(doc.data)) {
    output += `${key}: ${formatValue(value)}\n`;
  }

  output += '\n[footer]\n';
  for (const [key, value] of Object.entries(doc.footer)) {
    output += `${key}: ${formatValue(value)}\n`;
  }

  return output;
}

/**
 * Format value for ALN serialization
 * @param {*} value - Value to format
 * @returns {string} Formatted string
 */
function formatValue(value) {
  if (Array.isArray(value)) {
    return '[' + value.map(v => formatValue(v)).join(', ') + ']';
  }
  if (typeof value === 'string' && (value.includes(' ') || value.includes(':'))) {
    return `"${value}"`;
  }
  return String(value);
}

module.exports = {
  parseAlnDocument,
  validateChainlexemes,
  serializeAlnDocument,
  ParsedAlnDoc,
  ValidationReport
};
