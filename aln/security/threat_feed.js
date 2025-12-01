/**
 * ThreatFeedIngestor
 *
 * Aggregates external malware/threat intel feeds and normalizes patterns
 * for MLThreatHooks.
 */

const fs = require('fs/promises');
const https = require('https');
const path = require('path');

class ThreatFeedIngestor {
  constructor(options = {}) {
    this.sources = options.sources || [
      { type: 'file', path: path.join(__dirname, 'threat_feed_sample.json') }
    ];
    this.cache = [];
    this.lastSyncMs = 0;
    // TODO(policy): gate sources against ALN compliance routing so only jurisdiction-approved feeds load per tenant.
  }

  async fetchSignatures() {
    const entries = [];
    for (const source of this.sources) {
      if (source.type === 'file') {
        const resolved = path.resolve(source.path);
        const raw = await fs.readFile(resolved, 'utf-8');
        entries.push(...JSON.parse(raw));
      } else if (source.type === 'https') {
        const payload = await this._fetchHttps(source.url, source.headers);
        entries.push(...JSON.parse(payload));
      }
    }
    this.cache = entries;
    this.lastSyncMs = Date.now();
    return entries
      .map((entry) => this._normalizeEntry(entry))
      .filter(Boolean);
  }

  async _fetchHttps(url, headers = {}) {
    return new Promise((resolve, reject) => {
      https.get(url, { headers }, (res) => {
        const chunks = [];
        res.on('data', (chunk) => chunks.push(chunk));
        res.on('end', () => resolve(Buffer.concat(chunks).toString('utf-8')));
      }).on('error', reject);
    });
  }

  _normalizeEntry(entry) {
    try {
      const pattern = entry.flags
        ? new RegExp(entry.pattern, entry.flags)
        : new RegExp(entry.pattern, 'i');
      return {
        pattern,
        domain: entry.domain || 'unknown',
        severity: entry.severity || 'medium',
        source: entry.source || 'external'
      };
    } catch (err) {
      return null;
    }
  }
}

module.exports = { ThreatFeedIngestor };
