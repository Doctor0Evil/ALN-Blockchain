const {
  buildTransferTx,
  signTx,
  verifyTxSignature,
  generateKeypairFromSeed
} = require('../../wallet/tx_builder');

describe('tx_builder signing', () => {
  test('produces verifiable Ed25519 signatures', () => {
    const { privateKey, publicKey } = generateKeypairFromSeed('seed-value');
    const tx = buildTransferTx('aln1sender', 'aln1receiver', '1000', 0, 100);

    const signed = signTx(tx, privateKey);
    expect(signed.footer.signature).toMatch(/^ed25519:0x[0-9a-f]+$/);

    const verification = verifyTxSignature(signed, publicKey);
    expect(verification.valid).toBe(true);
  });
});
