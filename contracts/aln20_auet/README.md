# ALN AU.ET (ALN20)

Fungible ALN-20 token representing Augmented-User Energy Tokens.

- Based on CW20 (CosmWasm) pattern.
- Non-mintable after instantiate: totalSupply assigned at instantiate time only.
- Immutable metadata: source_chain, source_denom, snapshot_height, snapshot_root, scaling config.
- Behavior: transfer, send, allowance; burn allowed.

Deployments should use `artifacts/aln_init_auet.json` produced by the scaling tool.

Unit tests (cw-multi-test) should verify: total supply == sum allocated, no mint path exists, metadata present.
