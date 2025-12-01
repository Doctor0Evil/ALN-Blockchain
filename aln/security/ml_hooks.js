/**
 * ML-based threat detection hooks
 * Integrates with malware.aln policy
 */

const { ThreatFeedIngestor } = require('./threat_feed');

const FALLBACK_SIGNATURES = [
  { pattern: /selfdestruct\(/i, domain: 'drainer', severity: 'high' },
  { pattern: /delegatecall/i, domain: 'supply_chain', severity: 'high' },
  { pattern: /tx\.origin/i, domain: 'phishing', severity: 'medium' }
];

class MLThreatHooks {
  constructor(threatFeedIngestor = null) {
    this.signatures = [];
    this.lastUpdate = 0;
    this.threatFeedIngestor = threatFeedIngestor || new ThreatFeedIngestor();
  }

  /**
   * Load malware signatures from registry or feed
   */
  async loadSignatures() {
    try {
      if (this.threatFeedIngestor) {
        const feedSignatures = await this.threatFeedIngestor.fetchSignatures();
        this.signatures = feedSignatures.filter(Boolean);
        this.lastUpdate = Date.now();
        return this.signatures;
      }
    } catch (err) {
      console.warn('[MLThreatHooks] Threat feed fetch failed, using fallback:', err.message);
    }

    this.signatures = FALLBACK_SIGNATURES;
    this.lastUpdate = Date.now();
    return this.signatures;
  }

  /**
   * Score transaction or contract payload
   * @param {string|Object} payload - Code or transaction data
   * @returns {Object} ThreatScore
   */
  scorePayload(payload) {
    let score = 0;
    let domain = 'none';
    let evidence = [];
    const text = typeof payload === 'string' ? payload : JSON.stringify(payload);

    for (const sig of this.signatures) {
      if (sig.pattern.test(text)) {
        score += sig.severity === 'high' ? 40 : 20;
        domain = sig.domain;
        evidence.push({ pattern: sig.pattern.toString(), severity: sig.severity });
      }
    }

    return {
      score: Math.min(score, 100),
      domain,
      confidence: score > 0 ? 0.8 : 0.0,
      evidence
    };
  }

  /**
   * Update policies periodically
   */
  async updatePolicies() {
    await this.loadSignatures();
  }
}

module.exports = { MLThreatHooks };
