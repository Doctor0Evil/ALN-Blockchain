/**
 * ALN Transaction Builder
 * 
 * Non-custodial transaction builder using chainlexeme format.
 * Private keys never leave the browser.
 */

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
  return {
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
      to: 'aln1governance000000000000000000000000000',
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
  return {
    header: {
      op_code: 'governance_vote',
      from: voter,
      to: 'aln1governance000000000000000000000000000',
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
}

/**
 * Sign transaction (mock implementation)
 * In production, use proper ed25519 signing with Web Crypto API
 * 
 * @param {Object} chainlexeme - Unsigned chainlexeme
 * @param {string} privateKey - Private key (never sent to server)
 * @returns {Object} Signed chainlexeme
 */
function signTx(chainlexeme, privateKey) {
  // Mock signature generation
  // In production: use crypto.subtle.sign with ed25519
  
  const message = JSON.stringify({
    header: chainlexeme.header,
    data: chainlexeme.data
  });
  
  // Generate mock signature
  const mockSignature = `ed25519:0x${hashMock(message + privateKey)}`;
  
  chainlexeme.footer.signature = mockSignature;
  
  return chainlexeme;
}

/**
 * Mock hash function (replace with actual crypto)
 */
function hashMock(data) {
  let hash = 0;
  for (let i = 0; i < data.length; i++) {
    hash = ((hash << 5) - hash) + data.charCodeAt(i);
    hash = hash & hash;
  }
  return Math.abs(hash).toString(16).padStart(64, '0').substring(0, 64);
}

/**
 * Generate deterministic keypair from seed (mock)
 * In production: use proper ed25519 key derivation
 */
function generateKeypairFromSeed(seed) {
  const privateKey = hashMock(seed);
  const publicKey = hashMock(privateKey);
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
function verifyTxSignature(chainlexeme) {
  if (!chainlexeme.footer.signature) {
    return { valid: false, error: 'No signature' };
  }
  
  if (!chainlexeme.footer.signature.startsWith('ed25519:')) {
    return { valid: false, error: 'Invalid signature format' };
  }
  
  // In production: verify ed25519 signature
  return { valid: true };
}

// Export for use in browser or Node.js
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    buildTransferTx,
    buildGovernanceProposalTx,
    buildGovernanceVoteTx,
    signTx,
    generateKeypairFromSeed,
    verifyTxSignature
  };
}
