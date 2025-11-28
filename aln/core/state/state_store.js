/**
 * ALN State Store
 * 
 * Persistent key-value store for blockchain state using LevelDB.
 * Implements deterministic serialization and state root computation.
 */

const crypto = require('crypto');
const { Level } = require('level');
const { parseAlnDocument, validateChainlexemes } = require('../runtime/aln_parser');

class StateStore {
  constructor(dbPath = './data/state') {
    this.db = new Level(dbPath, { valueEncoding: 'json' });
    this.cache = new Map();
  }

  async open() {
    await this.db.open();
  }

  async close() {
    await this.db.close();
  }

  /**
   * Get account by address
   * @param {string} addr - ALN address
   * @returns {Promise<Object|null>} Account object or null
   */
  async getAccount(addr) {
    const key = `acc:${addr}`;
    
    if (this.cache.has(key)) {
      return this.cache.get(key);
    }

    try {
      const account = await this.db.get(key);
      this.cache.set(key, account);
      return account;
    } catch (err) {
      if (err.notFound) {
        return null;
      }
      throw err;
    }
  }

  /**
   * Set account data
   * @param {string} addr - ALN address
   * @param {Object} account - Account object
   */
  async setAccount(addr, account) {
    const key = `acc:${addr}`;
    await this.db.put(key, account);
    this.cache.set(key, account);
  }

  /**
   * Create new account with default values
   * @param {string} addr - ALN address
   * @returns {Object} New account object
   */
  createAccount(addr) {
    return {
      address: addr,
      nonce: 0,
      balance: '0',
      token_balances: {},
      voting_power: '0',
      delegated_to: null,
      code_hash: null,
      storage_root: null
    };
  }

  /**
   * Get balance for specific asset
   * @param {string} addr - ALN address
   * @param {string} asset - Asset identifier (default: 'ALN')
   * @returns {Promise<string>} Balance as string (to avoid precision issues)
   */
  async getBalance(addr, asset = 'ALN') {
    const account = await this.getAccount(addr);
    if (!account) return '0';

    if (asset === 'ALN') {
      return account.balance;
    }
    return account.token_balances[asset] || '0';
  }

  /**
   * Set balance for specific asset
   * @param {string} addr - ALN address
   * @param {string} asset - Asset identifier
   * @param {string} amount - New balance as string
   */
  async setBalance(addr, asset, amount) {
    let account = await this.getAccount(addr);
    if (!account) {
      account = this.createAccount(addr);
    }

    if (asset === 'ALN') {
      account.balance = amount;
    } else {
      account.token_balances[asset] = amount;
    }

    await this.setAccount(addr, account);
  }

  /**
   * Apply chainlexeme transaction to state
   * @param {Object} chainlexeme - Parsed chainlexeme object
   * @returns {Promise<Object>} Result with success status and details
   */
  async applyTransaction(chainlexeme) {
    const { header, data, footer } = chainlexeme;
    const result = { success: false, error: null, state_changes: [] };

    try {
      // Get from account
      let fromAccount = await this.getAccount(header.from);
      if (!fromAccount) {
        fromAccount = this.createAccount(header.from);
      }

      // Verify nonce
      if (fromAccount.nonce !== header.nonce) {
        result.error = `Invalid nonce. Expected ${fromAccount.nonce}, got ${header.nonce}`;
        return result;
      }

      // Capture chat-native metadata (optional)
      const chatMeta = {
        chat_context_id: header.chat_context_id || null,
        transcript_hash: header.transcript_hash || null,
        jurisdiction_tags: Array.isArray(header.jurisdiction_tags) ? header.jurisdiction_tags : null
      };

      if (chatMeta.chat_context_id || chatMeta.transcript_hash) {
        result.state_changes.push({ type: 'chat_metadata', meta: chatMeta });
      }

      // Handle different operation types
      switch (header.op_code) {
        case 'transfer':
          await this._applyTransfer(fromAccount, header, data, result);
          break;
        
        case 'governance_vote':
          await this._applyGovernanceVote(fromAccount, header, data, result);
          break;
        
        case 'migration_mint':
          await this._applyMigrationMint(header, data, result);
          break;
        
        case 'delegation':
          await this._applyDelegation(fromAccount, header, data, result);
          break;
        
        default:
          result.error = `Unsupported op_code: ${header.op_code}`;
          return result;
      }

      // Increment nonce
      fromAccount.nonce++;
      await this.setAccount(header.from, fromAccount);
      result.state_changes.push({ type: 'nonce_increment', address: header.from });

      // Persist lightweight audit record if chat metadata present
      if (chatMeta.chat_context_id || chatMeta.transcript_hash) {
        await this._appendAuditRecord({
          tx_from: header.from,
          tx_to: header.to,
          op_code: header.op_code,
          chat_context_id: chatMeta.chat_context_id,
            transcript_hash: chatMeta.transcript_hash,
          jurisdiction_tags: chatMeta.jurisdiction_tags,
          timestamp: footer.timestamp || Math.floor(Date.now() / 1000)
        });
      }

      result.success = true;
      return result;

    } catch (err) {
      result.error = err.message;
      return result;
    }
  }

  /**
   * Append audit record for chat-native enriched transactions
   */
  async _appendAuditRecord(record) {
    const key = `audit:${record.tx_from}:${record.timestamp}:${record.op_code}`;
    try {
      await this.db.put(key, record);
    } catch (err) {
      // Swallow audit write errors to avoid impacting consensus path
    }
  }

  /**
   * Apply transfer operation
   */
  async _applyTransfer(fromAccount, header, data, result) {
    const asset = data.asset || 'ALN';
    const amount = BigInt(data.amount || 0);

    // Get current balance
    const fromBalance = BigInt(await this.getBalance(header.from, asset));
    
    if (fromBalance < amount) {
      throw new Error(`Insufficient balance. Have ${fromBalance}, need ${amount}`);
    }

    // Get or create to account
    let toAccount = await this.getAccount(header.to);
    if (!toAccount) {
      toAccount = this.createAccount(header.to);
    }

    const toBalance = BigInt(await this.getBalance(header.to, asset));

    // Update balances
    await this.setBalance(header.from, asset, (fromBalance - amount).toString());
    await this.setBalance(header.to, asset, (toBalance + amount).toString());

    result.state_changes.push(
      { type: 'balance_decrease', address: header.from, asset, amount: amount.toString() },
      { type: 'balance_increase', address: header.to, asset, amount: amount.toString() }
    );
  }

  /**
   * Apply governance vote operation
   */
  async _applyGovernanceVote(fromAccount, header, data, result) {
    const proposalKey = `prop:${data.proposal_id}`;
    
    try {
      const proposal = await this.db.get(proposalKey);
      const votingPower = BigInt(fromAccount.voting_power);

      // Update vote tallies
      if (data.support === 'for') {
        proposal.votes_for = (BigInt(proposal.votes_for) + votingPower).toString();
      } else if (data.support === 'against') {
        proposal.votes_against = (BigInt(proposal.votes_against) + votingPower).toString();
      } else {
        proposal.votes_abstain = (BigInt(proposal.votes_abstain) + votingPower).toString();
      }

      // Record voter
      if (!proposal.voters) proposal.voters = [];
      proposal.voters.push({ address: header.from, support: data.support, power: votingPower.toString() });

      await this.db.put(proposalKey, proposal);
      result.state_changes.push({ type: 'vote_cast', proposal_id: data.proposal_id });

    } catch (err) {
      if (err.notFound) {
        throw new Error(`Proposal not found: ${data.proposal_id}`);
      }
      throw err;
    }
  }

  /**
   * Apply migration mint operation
   */
  async _applyMigrationMint(header, data, result) {
    const asset = data.asset;
    const amount = BigInt(data.amount);

    // Get or create destination account
    let toAccount = await this.getAccount(header.to);
    if (!toAccount) {
      toAccount = this.createAccount(header.to);
    }

    const currentBalance = BigInt(await this.getBalance(header.to, asset));
    await this.setBalance(header.to, asset, (currentBalance + amount).toString());

    // Record migration
    const migrationKey = `mig:${data.source_tx_hash}`;
    await this.db.put(migrationKey, {
      migration_id: data.source_tx_hash,
      source_chain: data.source_chain,
      source_tx_hash: data.source_tx_hash,
      dest_address: header.to,
      asset_type: asset,
      amount: amount.toString(),
      proof_hash: data.proof_hash,
      status: 'minted',
      created_at: data.timestamp || Date.now()
    });

    result.state_changes.push(
      { type: 'migration_mint', address: header.to, asset, amount: amount.toString() }
    );
  }

  /**
   * Apply delegation operation
   */
  async _applyDelegation(fromAccount, header, data, result) {
    fromAccount.delegated_to = data.delegate_to;
    await this.setAccount(header.from, fromAccount);
    result.state_changes.push({ type: 'delegation', from: header.from, to: data.delegate_to });
  }

  /**
   * Compute state root hash (Merkle root)
   * @returns {Promise<string>} State root hash
   */
  async computeStateRoot() {
    const hashes = [];
    
    // Iterate all keys and hash values
    for await (const [key, value] of this.db.iterator()) {
      const hash = crypto.createHash('sha256')
        .update(key + JSON.stringify(value))
        .digest('hex');
      hashes.push(hash);
    }

    // Sort hashes for determinism
    hashes.sort();

    // Compute Merkle root
    if (hashes.length === 0) {
      return '0'.repeat(64);
    }

    while (hashes.length > 1) {
      const newHashes = [];
      for (let i = 0; i < hashes.length; i += 2) {
        if (i + 1 < hashes.length) {
          const combined = crypto.createHash('sha256')
            .update(hashes[i] + hashes[i + 1])
            .digest('hex');
          newHashes.push(combined);
        } else {
          newHashes.push(hashes[i]);
        }
      }
      hashes.length = 0;
      hashes.push(...newHashes);
    }

    return hashes[0];
  }

  /**
   * Clear cache (call after block commit)
   */
  clearCache() {
    this.cache.clear();
  }
}

module.exports = StateStore;
