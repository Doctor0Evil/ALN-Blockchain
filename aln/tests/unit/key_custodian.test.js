const fs = require('fs/promises');
const os = require('os');
const path = require('path');
const { KeyCustodian } = require('../../security/key_custodian');

describe('KeyCustodian', () => {
  let tempDir;
  let custodian;

  beforeEach(async () => {
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'aln-custodian-'));
    custodian = new KeyCustodian(tempDir);
    await custodian.initialize();
  });

  afterEach(async () => {
    await fs.rm(tempDir, { recursive: true, force: true });
  });

  test('creates and retrieves encrypted key material', async () => {
    const keyA = await custodian.ensureKey('validator', 'strong-passphrase');
    expect(keyA).toBeInstanceOf(Buffer);
    expect(keyA.length).toBe(32);

    const keyB = await custodian.ensureKey('validator', 'strong-passphrase');
    expect(Buffer.compare(keyA, keyB)).toBe(0);
  });

  test('rejects decryption with wrong passphrase', async () => {
    await custodian.ensureKey('validator', 'correct-passphrase');
    await expect(custodian.ensureKey('validator', 'bad-passphrase')).rejects.toThrow();
  });
});
