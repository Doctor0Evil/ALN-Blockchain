# ALN20 Non-Mintable Design: Kujira Orphan Tokens & AU.ET / CSP Mapping

## Purpose
Provide a reproducible, auditable, and legally‑sound process to: identify undocumented/orphaned Kujira denominated assets; snapshot and prove the origin of on‑chain holdings; rescale those holdings to ALN‑20 assets (AU.ET and CSP) with deterministic math and strict non‑mintable containment; and deploy Cosmos-WASM or EVM contracts that enforce immutability and provenance.

---

## Key Concepts
- Orphan tokens: IBC/factory/gravity tokens not present in the registry, or whose base denom is not in a curated allowlist.
- Snapshot tuple S_i = (chain_id, height, denom, address, balance).
- Provenance hash H_i = SHA256(chain_id || height_be_u64 || denom || address_bytes || balance_be_u256).
- Merkle root over {H_i}, stored in metadata for the ALN-20 series.
- Scaling factor c_E and c_S to compress human-readable balances into tiny ALN units.

---

## Step-by-step outline
1) Identification & classification
- Call Kujira denom-trace endpoint to collect {path, base_denom}.
- Recompute ibc/{SHA256(path + "/" + base_denom)} and match on-chain denoms.
- Mark as Category B (candidate for remapping) only if base_denom is not in the authoritative allowlist.

2) Snapshot & provenance
- Choose snapshot height h.
- For each candidate denom and address where an operator has balances, record integer minimal balance B_i.
- Construct S_i and compute H_i as described above.
- Build a Merkle tree to create the snapshot root R.
- Publish raw CSV + root + H_i values for auditing.

3) Deterministic scaling math
- Input: B_i (minimal source units), d_src (source decimals), d_aln (ALN decimals chosen), c_E, c_S.
- Normalize: A_src = B_i / 10^d_src
- AU.ET (human): A_E = A_src * c_E
- CSP (human): A_S = A_src * c_S
- Minimal ALN units: B_E = floor(A_E * 10^d_aln), B_S = floor(A_S * 10^d_aln)

Proof: fixed inputs (chain_id, h, di, a, B_i, d_src, d_aln, c_E, c_S) produce unique outputs (B_E,B_S). Publishing all of these allows anyone to recompute allocations.

Examples
- Unknown 1,010,000 with d_src=6 (so B = 1,010,000.000000):
  - A_src = 1,010,000
  - c_E = 10^-12 => A_E = 1.01e-6 => B_E(min units, d_aln=6) = floor(1.01e-6*1e6) = 1.
  - c_S = 5e-13 => A_S = 5.05e-7 => B_S = floor(5.05e-7*1e6) = 0.

More examples
- Unknown 500,000 (d_src=6):
  * A_src = 500,000
  * c_E = 1e-12 => A_E = 5e-7 => B_E = floor(5e-7*1e6) = floor(0.5) = 0
  * c_S = 5e-13 => A_S = 2.5e-7 => B_S = floor(2.5e-7*1e6) = 0

- Unknown 84,114.977103 (d_src=6):
  * A_src = 84,114.977103
  * c_E = 1e-12 => A_E = 8.41149771e-8 => B_E = floor(8.41149771e-8*1e6) = floor(0.0841149771) = 0
  * c_S = 5e-13 => A_S = 4.205748855e-8 => B_S = 0

These show how aggressive compression yields small or zero ALN minimal units for deprecated, abandoned, or suspicious supplies; the compression constants can be tuned by governance if different scarcity or thresholds are desired.

4) Contract policy & containment
- AU.ET, CSP ALN-20 must be non-mintable post-deploy (mint only in constructor/instantiate).
- Store immutable metadata: source_chain_id, source_denom, snapshot_height, snapshot_root, scaling_config_id.
- No external mint paths; burning allowed.
- Bridge contract holds allocation and has `claimed(snapshot_hash) -> bool` to avoid replays.

5) Bridge & claim
- On Kujira: lock/burn tokens (if safe). Emit event with snapshot tuple and H_i.
- On ALN: the bridge consumes the proof of lock/burn and mints the precomputed amount into ALN-20 tokens per allocation (one-time).
- Alternatively: require the operator to deposit the precomputed AU.ET/CSP to the bridge contract (fund pool), and then claims are transfers from that pool after verifying H_i.

6) Legal & compliance
- Only convert operator-held balances (never sweep others’ holdings).
- Use cut-off date h and whitelist rules for base_denoms.
- Publish the criteria and snapshots for auditors.

---

## Hashing spec (binary format) for H_i
- chain_id: bytes, ASCII.
- height (u64 big-endian): 8 bytes.
- denom: bytes, ASCII.
- address bytes: 20 bytes (Bech32 decoded). For addresses shorter or with other encoding, use canonical form.
- balance (u256 big-endian): 32 bytes.
- Serialize and then `H_i = SHA256(serialized_bytes)`.

Merkle root over H_i values (pairwise SHA256(H_left || H_right)).

---

## ALN-20 metadata
- name, symbol, decimals
- total_supply (immutable)
- source_chain_id, source_denom, snapshot_height
- snapshot_root (merkle root)
- scaling_config: { d_src, d_aln, c_E, c_S }
- provenance_registry_tx (URL/chain+tx) - optional

---

## Non-inflation guarantee
- No external mint function.
- totalSupply is sum of initial allocations.
- Burn allowed; burns decrease totalSupply.

---

## Audit checklist
- All H_i recompute from raw CSV inputs.
- sum(B_E) == contract.totalSupply AU.ET and sum(B_S) == contract.totalSupply CSP.
- No mint functions exist in final deployed encoding (either not included or disabled by `finalized` boolean and no admin role).
- Build & test `orphan_ibc.json`, `snapshots_orphan.csv`, `snapshot_root.json`, and verify with tools.

---

## Next actions & TODOs
- Implement orphan scanner (Rust) using Kujira LCD.
- Implement snapshot-hash (Rust) and merkle root generator.
- Implement scaling engine & allocation exporter.
- Add CosmWasm AU.ET & CSP contracts.
- Add bridge contract & deployment scripts.

---

Appendix: sample CLI commands (Kujira)
- List denom traces (paginated):
  `GET https://kujira-api.polkachu.com/ibc/apps/transfer/v1/denom_traces?pagination.limit=1000`
- Query a specific denom trace:
  `GET https://kujira-api.polkachu.com/ibc/apps/transfer/v1/denom_traces/{HASH}`


References
- IBC ADR-001: coin-source tracing
- Kujira API / Finder
- CosmWasm cw20 base & multi-test
- OpenZeppelin ERC20 patterns
