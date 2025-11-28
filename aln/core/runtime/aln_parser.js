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
    this.metadata = {
      modules: [],
      types: [],
      raw: null
    };
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

  // Support for module/type syntax (aln_module "name" { ... })
  // We keep a simple state machine for module blocks
  let inModule = false;
  let currentModuleName = null;
  let currentModuleBuffer = [];

  for (const rawLine of lines) {
    lineNumber++;
    const line = rawLine.replace(/\r$/, '');
    const trimmed = line.trim();

    if (!trimmed) continue;

    // Comments
    if (trimmed.startsWith('#') || trimmed.startsWith('//')) continue;

    // Module start
    if (!inModule && /^aln_module\s+"[^"]+"\s*\{\s*$/.test(trimmed)) {
      inModule = true;
      currentModuleName = trimmed.match(/^aln_module\s+"([^"]+)"/)[1];
      currentModuleBuffer = [];
      continue;
    }

    // Module end
    if (inModule && trimmed === '}') {
      doc.rawSections[`module:${currentModuleName}`] = currentModuleBuffer.slice();
      inModule = false;
      currentModuleName = null;
      currentModuleBuffer = [];
      continue;
    }

    if (inModule) {
      currentModuleBuffer.push({ line: lineNumber, content: trimmed });
      continue;
    }

    // Section headers
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
      currentSection = trimmed.slice(1, -1).toLowerCase();
      doc.rawSections[currentSection] = [];
      continue;
    }

    // Key-value inside chainlexeme sections
    const colonIndex = trimmed.indexOf(':');
    if (colonIndex > 0 && currentSection) {
      const key = trimmed.slice(0, colonIndex).trim();
      const value = trimmed.slice(colonIndex + 1).trim();

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

  // Rule 2: Validate header fields (support chat-native metadata)
  const requiredHeaderFields = ['op_code', 'from', 'to', 'nonce'];
  for (const field of requiredHeaderFields) {
    if (!(field in doc.header)) {
      report.addError(`Missing required header field: ${field}`);
    }
  }

  // Optional chat-native fields - warn if governance/migration without them
  const chatFields = ['chat_context_id', 'transcript_hash'];
  const needsChatFields = ['governance_proposal', 'governance_vote', 'migration_lock', 'migration_mint'];
  if (doc.header.op_code && needsChatFields.includes(doc.header.op_code)) {
    for (const cf of chatFields) {
      if (!(cf in doc.header)) {
        report.addWarning(`Recommended header field missing for ${doc.header.op_code}: ${cf}`);
      }
    }
  }
  if (doc.header.jurisdiction_tags && !Array.isArray(doc.header.jurisdiction_tags)) {
    report.addError('jurisdiction_tags must be an array');
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
  if (doc.header.from && typeof doc.header.from === 'string' && !doc.header.from.startsWith('aln1')) {
    report.addError(`Invalid from address format: must start with 'aln1'`);
  }
  if (doc.header.to && typeof doc.header.to === 'string' && !doc.header.to.startsWith('aln1')) {
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

  // Governance vote requires proposal_id and support
  if (doc.header.op_code === 'governance_vote') {
    if (!doc.data.proposal_id) {
      report.addError('Governance vote requires proposal_id in data section');
    }
    if (!doc.data.support) {
      report.addError('Governance vote requires support (for|against|abstain)');
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
