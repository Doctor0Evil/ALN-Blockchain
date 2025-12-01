/**
 * ALN Transaction Builder
 * 
 * Non-custodial transaction builder using chainlexeme format.
 * Private keys never leave the browser.
 */

const crypto = require('crypto');
const { sync: ed25519 } = require('@noble/ed25519');
const { GOVERNANCE_ADDRESSES } = require('../core/config/governance');

/**
 * Build transfer transaction chainlexeme
 * @param {string} fromAddr - Sender address
 * @param {string} toAddr - Recipient address
 * @param {string} amount - Amount to transfer
 * @param {number} nonce - Transaction nonce
 * @param {number} fee - Transaction fee (gas price)
 * @returns {Object} Unsigned chainlexeme
 */
function buildTransferTx(fromAddr, toAddr, amount, nonce, fee = 100) {
  const tx = {
    header: {
      op_code: 'transfer',
      from: fromAddr,
      to: toAddr,
      nonce: nonce
    },
    data: {
      asset: 'ALN',
      amount: amount.toString(),
      constraints: []
    },
    footer: {
      signature: null, // To be filled by signTx
      timestamp: Math.floor(Date.now() / 1000),
      gas_limit: 21000,
      gas_price: fee
    }
  };
  return tx;
}

/**
 * Build governance proposal transaction
 * @param {string} proposer - Proposer address
 * @param {Object} proposalData - Proposal details
 * @returns {Object} Unsigned chainlexeme
 */
function buildGovernanceProposalTx(proposer, proposalData) {
  return {
    header: {
      op_code: 'governance_proposal',
      from: proposer,
      to: GOVERNANCE_ADDRESSES.COUNCIL,
      nonce: proposalData.nonce
    },
    data: {
      proposal_id: proposalData.proposal_id,
      title: proposalData.title,
      description: proposalData.description,
      category: proposalData.category,
      execution_route: proposalData.execution_route,
      quorum: proposalData.quorum || 0.4,
      threshold: proposalData.threshold || 0.66,
      duration_blocks: proposalData.duration_blocks || 10000,
      constraints: proposalData.constraints || []
    },
    footer: {
      signature: null,
      timestamp: Math.floor(Date.now() / 1000),
      gas_limit: 500000,
      gas_price: 200
    }
  };
}

/**
 * Build governance vote transaction
 * @param {string} voter - Voter address
 * @param {string} proposalId - Proposal ID
 * @param {string} support - 'for' | 'against' | 'abstain'
 * @param {number} nonce - Transaction nonce
 * @returns {Object} Unsigned chainlexeme
 */
function buildGovernanceVoteTx(voter, proposalId, support, nonce) {
  const tx = {
    header: {
      op_code: 'governance_vote',
      from: voter,
      to: GOVERNANCE_ADDRESSES.COUNCIL,
      nonce: nonce
    },
    data: {
      proposal_id: proposalId,
      support: support
    },
    footer: {
      signature: null,
      timestamp: Math.floor(Date.now() / 1000),
      gas_limit: 100000,
      gas_price: 150
    }
  };
  return tx;
}

/**
 * Attach chat-native metadata to any unsigned transaction
 * @param {Object} tx - chainlexeme transaction object
 * @param {Object} meta - { chat_context_id, transcript_hash, jurisdiction_tags }
 * @returns {Object} augmented transaction
 */
function withChatMetadata(tx, meta = {}) {
  if (!tx || !tx.header) return tx;
  if (meta.chat_context_id) tx.header.chat_context_id = meta.chat_context_id;
  if (meta.transcript_hash) tx.header.transcript_hash = meta.transcript_hash;
  if (meta.jurisdiction_tags && Array.isArray(meta.jurisdiction_tags)) {
    tx.header.jurisdiction_tags = meta.jurisdiction_tags;
  }
  return tx;
}

/**
 * Compute signing digest (SHA-256 of header+data)
 */
function computeSigningDigest(chainlexeme) {
  return crypto.createHash('sha256').update(JSON.stringify({
    header: chainlexeme.header,
    data: chainlexeme.data
  })).digest();
}

/**
 * Sign transaction using Ed25519
 *
 * @param {Object} chainlexeme - Unsigned chainlexeme
 * @param {string} privateKeyHex - 64-char hex encoded private key
 * @returns {Object} Signed chainlexeme
 */
function signTx(chainlexeme, privateKeyHex) {
  if (!privateKeyHex) {
    throw new Error('Private key required for signing');
  }

  const privateKey = Buffer.from(privateKeyHex, 'hex');
  if (privateKey.length !== 32) {
    throw new Error('Invalid private key length; expected 32 bytes');
  }

  const digest = computeSigningDigest(chainlexeme);
  const signature = ed25519.sign(digest, privateKey);
  chainlexeme.footer.signature = `ed25519:0x${Buffer.from(signature).toString('hex')}`;
  return chainlexeme;
}

/**
 * Generate deterministic keypair from seed (mock)
 * In production: use proper ed25519 key derivation
 */
function generateKeypairFromSeed(seed) {
  const seedBuffer = crypto.createHash('sha512').update(seed).digest();
  const privateKeyBytes = seedBuffer.subarray(0, 32);
  const publicKeyBytes = ed25519.getPublicKey(privateKeyBytes);

  const privateKey = Buffer.from(privateKeyBytes).toString('hex');
  const publicKey = Buffer.from(publicKeyBytes).toString('hex');
  const address = `aln1${publicKey.substring(0, 40)}`;
  
  return {
    privateKey,
    publicKey,
    address
  };
}

/**
 * Verify transaction signature (client-side verification)
 */
function verifyTxSignature(chainlexeme, publicKeyHex) {
  if (!chainlexeme.footer.signature) {
    return { valid: false, error: 'No signature' };
  }

  if (!publicKeyHex) {
    return { valid: false, error: 'Public key required for verification' };
  }

  if (!chainlexeme.footer.signature.startsWith('ed25519:0x')) {
    return { valid: false, error: 'Invalid signature format' };
  }

  const signatureHex = chainlexeme.footer.signature.replace('ed25519:0x', '');
  const signature = Buffer.from(signatureHex, 'hex');
  const publicKey = Buffer.from(publicKeyHex, 'hex');
  const digest = computeSigningDigest(chainlexeme);
  const valid = ed25519.verify(signature, digest, publicKey);

  return valid ? { valid: true } : { valid: false, error: 'Signature verification failed' };
}

// Export for use in browser or Node.js
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    buildTransferTx,
    buildGovernanceProposalTx,
    buildGovernanceVoteTx,
    withChatMetadata,
    signTx,
    generateKeypairFromSeed,
    verifyTxSignature,
    computeSigningDigest
  };
}
