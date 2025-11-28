/**
 * Reputation and credit primitives
 * Non-transferable, DID-linked, auditable
 */

class ReputationSystem {
  constructor(stateStore) {
    this.stateStore = stateStore;
    this.goodDeeds = new Map();
    this.reputations = new Map();
    this.appeals = new Map();
  }

  /**
   * Issue good deed record
   */
  async issueGoodDeedRecord(user, deedType, authority, jurisdiction) {
    // TODO: Validate authority credential
    const recordId = `gdr_${Date.now()}_${user.substring(0, 8)}`;
    const record = {
      id: recordId,
      user,
      deedType,
      verifyingAuthority: authority,
      timestamp: Date.now(),
      jurisdiction,
      evidenceRefs: []
    };

    this.goodDeeds.set(recordId, record);

    // Update reputation
    await this.updateReputation(user, 'positive');

    // TODO: Emit audit event
    return recordId;
  }

  /**
   * Revoke record
   */
  async revokeRecord(recordId, reason, authority) {
    const record = this.goodDeeds.get(recordId);
    if (!record) throw new Error('Record not found');

    // TODO: Validate authority
    record.revoked = true;
    record.revokeReason = reason;

    await this.updateReputation(record.user, 'negative');
    // TODO: Emit audit event
  }

  /**
   * Appeal a record
   */
  async appealRecord(recordId, user, appealReason) {
    const appealId = `appeal_${Date.now()}_${recordId}`;
    this.appeals.set(appealId, {
      id: appealId,
      recordId,
      user,
      appealReason,
      status: 'PENDING',
      timestamp: Date.now()
    });
    return appealId;
  }

  /**
   * Resolve appeal
   */
  async resolveAppeal(appealId, decision, authority) {
    const appeal = this.appeals.get(appealId);
    if (!appeal) throw new Error('Appeal not found');

    appeal.status = decision === 'approve' ? 'APPROVED' : 'DENIED';
    appeal.resolver = authority;

    if (decision === 'approve') {
      // Adjust reputation
      await this.updateReputation(appeal.user, 'positive');
    }
    // TODO: Emit transparency event
  }

  /**
   * Compute reputation score
   */
  computeReputation(user) {
    const rep = this.reputations.get(user) || {
      positiveEventsCount: 0,
      negativeEventsCount: 0,
      lawEnfAssistEvents: 0
    };

    // Formula: weighted sum with floor at 10
    const score = Math.max(10, Math.min(100, rep.positiveEventsCount * 10 - rep.negativeEventsCount * 20));
    return score;
  }

  /**
   * Update reputation helper
   */
  async updateReputation(user, eventType) {
    const rep = this.reputations.get(user) || {
      user,
      positiveEventsCount: 0,
      negativeEventsCount: 0,
      lawEnfAssistEvents: 0,
      lastReviewBlock: 0,
      computedScore: 0
    };

    if (eventType === 'positive') rep.positiveEventsCount++;
    if (eventType === 'negative') rep.negativeEventsCount++;
    rep.computedScore = this.computeReputation(user);

    this.reputations.set(user, rep);
  }
}

module.exports = { ReputationSystem };
