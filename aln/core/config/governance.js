/**
 * ALN Governance Constants Module
 *
 * Centralizes governance-related addresses and policy identifiers so
 * contracts, CLIs, and services never hard-code destinations.
 */

const GOVERNANCE_ADDRESSES = Object.freeze({
  /**
   * Primary governance council / proposal registry account.
   * Placeholder until mainnet genesis assigns final bech32 address.
   */
  COUNCIL: 'aln1governance000000000000000000000000000',

  /**
   * Address reserved for policy + compliance registry submissions.
   */
  POLICY_REGISTRY: 'aln1policy000000000000000000000000000',

  /**
   * Address reserved for migragraph + bridge governance hooks.
   */
  MIGRAGRAPH_ROUTER: 'aln1migrationgov00000000000000000000'
});

const GOVERNANCE_POLICY_TAGS = Object.freeze({
  CHATAI_DAO: 'dao.chatai',
  AUGMENTED_POLICY: 'policy.augmented.capability',
  MIGRAGRAPH_BRIDGE: 'policy.migration.bridge',
  LAW_ENF_ASSIST: 'policy.law_enf_assist'
});

/**
 * Resolve a governance address by key.
 * @param {keyof typeof GOVERNANCE_ADDRESSES} key
 * @returns {string | null}
 */
function getGovernanceAddress(key) {
  return GOVERNANCE_ADDRESSES[key] || null;
}

module.exports = {
  GOVERNANCE_ADDRESSES,
  GOVERNANCE_POLICY_TAGS,
  getGovernanceAddress
};
