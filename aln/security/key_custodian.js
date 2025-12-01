/**
 * KeyCustodian
 *
 * Provides encrypted-at-rest management for validator and wallet keys.
 * Can proxy to hardware security modules or secure enclaves in production.
 */

const fs = require('fs/promises');
const path = require('path');
const crypto = require('crypto');

class KeyCustodian {
  constructor(rootDir, options = {}) {
    this.rootDir = rootDir;
    this.kdfIterations = options.kdfIterations || 120000;
    this.algorithm = 'aes-256-gcm';
  }

  async initialize() {
    await fs.mkdir(this.rootDir, { recursive: true });
  }

  // TODO(policy): extend signature to accept usageScope (governance|wallet|threat_feed) and log each access.
  async ensureKey(label, passphrase) {
    if (!passphrase) {
      throw new Error('Custodian passphrase required');
    }

    const filePath = this._keyPath(label);
    const exists = await this._fileExists(filePath);
    if (!exists) {
      const privateKey = crypto.randomBytes(32);
      await this._writeEncrypted(filePath, privateKey, passphrase);
      return privateKey;
    }
    try {
      return await this._readEncrypted(filePath, passphrase);
    } catch (err) {
      throw new Error(`KeyCustodian(${label}) decrypt failed: ${err.message}`);
    }
  }

  async storeKey(label, privateKeyBytes, passphrase) {
    if (!Buffer.isBuffer(privateKeyBytes) || privateKeyBytes.length !== 32) {
      throw new Error('Private key must be 32-byte Buffer');
    }
    try {
      await this._writeEncrypted(this._keyPath(label), privateKeyBytes, passphrase);
    } catch (err) {
      throw new Error(`KeyCustodian(${label}) store failed: ${err.message}`);
    }
  }

  async signDigest(label, digest, passphrase, signer) {
    const keyBytes = await this.ensureKey(label, passphrase);
    if (typeof signer !== 'function') {
      throw new Error('Signer function required');
    }
    return signer(keyBytes, digest);
  }

  async _writeEncrypted(filePath, data, passphrase) {
    const salt = crypto.randomBytes(16);
    const key = await this._deriveKey(passphrase, salt);
    const iv = crypto.randomBytes(12);
    const cipher = crypto.createCipheriv(this.algorithm, key, iv);
    const ciphertext = Buffer.concat([cipher.update(data), cipher.final()]);
    const authTag = cipher.getAuthTag();

    const payload = {
      iv: iv.toString('hex'),
      salt: salt.toString('hex'),
      authTag: authTag.toString('hex'),
      ciphertext: ciphertext.toString('hex'),
      created_at: Date.now()
    };
    await fs.writeFile(filePath, JSON.stringify(payload, null, 2), 'utf-8');
  }

  async _readEncrypted(filePath, passphrase) {
    const raw = await fs.readFile(filePath, 'utf-8');
    const payload = JSON.parse(raw);
    const iv = Buffer.from(payload.iv, 'hex');
    const salt = Buffer.from(payload.salt, 'hex');
    const authTag = Buffer.from(payload.authTag, 'hex');
    const ciphertext = Buffer.from(payload.ciphertext, 'hex');

    const key = await this._deriveKey(passphrase, salt);
    const decipher = crypto.createDecipheriv(this.algorithm, key, iv);
    decipher.setAuthTag(authTag);
    const plaintext = Buffer.concat([decipher.update(ciphertext), decipher.final()]);
    return plaintext;
  }

  async _fileExists(filePath) {
    try {
      await fs.access(filePath);
      return true;
    } catch (err) {
      return false;
    }
  }

  _keyPath(label) {
    const safeLabel = label.replace(/[^a-zA-Z0-9_-]/g, '_');
    return path.join(this.rootDir, `${safeLabel}.sealed`);
  }

  async _deriveKey(passphrase, salt) {
    return new Promise((resolve, reject) => {
      crypto.scrypt(passphrase, salt, 32, { N: 2 ** 15, r: 8, p: 1 }, (err, derivedKey) => {
        if (err) return reject(err);
        resolve(derivedKey);
      });
    });
  }
}

module.exports = { KeyCustodian };
