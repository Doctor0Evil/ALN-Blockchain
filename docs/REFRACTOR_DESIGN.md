# ALN Sealed Refactor & Energy Ledger Design

This document documents high-level design and implementation details for the sealed refactor pipeline which replaces the conventional use of burn addresses with a VM-enforced, one-way refactor function and internal, account-bound energy ledger.

## Core principles

- One-way refactor function: `claim`/`claim_with_origin` only allow verified lock/burn proofs to be consumed and mint non-transferable energy (SECs) into an internal ledger.
- No outbound mint-unlock flow exists: there is no function that can convert SECs back to origin chain assets.
- Non-transferable tokens: ALN-20 SEC contract types (`AU.ET`, `CSP`) explicitly forbid user-to-user transfers; only whitelisted system contracts can consume SEC balances.
- UBS gating: UBS analysis (static + dynamic) determines if a bridged token is `Approved`, `Downgraded`, or `Rejected`. The `claim` flow uses UBS to map origin amounts to an internal `EnergyVector`.
- Replay protection: refactor events are keyed by `(origin_chain_id, tx_hash, nonce)` using `record_refactor` and `is_processed` functions.
- Structured audit: UBSanitization reports are stored in `REFACTOR_AUDIT` map (report hash -> string) and also emitted as structured attributes for indexing and observability.
- Ledger permissioning: the `debit` API validates system ACL stored in `SYSTEM_WHITELIST` and prevents unauthorized draws.

## Implementation notes

- `crates/aln_core::energy_ledger` implements a simple Rust `EnergyLedger` trait and `InMemoryEnergyLedger` used for tests. The public trait contains `credit`, `debit`, and `balance_of`.
- `contracts/bridge` contract uses a `Map<&Addr, EnergyVector>` to persist ledger balances on chain and exposes `SystemConsume` to allow whitelisted system contracts to debit balances.
- UBS integration is wired using the `aln_ubs::DefaultUBS` instance; `claim` calls `sanitize()` and accepts or rejects results accordingly.
- `REFACTOR_AUDIT` stores the UBS report hash keyed by `(origin_chain, tx_hash, nonce)`.
- Indexer (`crates/aln_indexer`) watches for bridge events and increments sealed refactor Prometheus metrics `sealed_refactor_total` and `sealed_refactor_rejected_total`.

## Testing & CI

- Contract unit tests are added for bridge to validate:
  - replay protection
  - ledger crediting on successful sealed refactor
  - `SystemConsume` ACL enforcement for whitelisted consumers
  - and `RefactorAudit` is saved and queryable.
- `crates/aln_core` includes unit tests for ledger credit/debit and `TokenFactory::mint_to_ledger` behavior.
- CI collects `aln_bridge` tests under `rust-integration` and indexer-run smoke tests.

## Operational notes

- UBS analysis must be integrated with the audit & registry pipeline; for the POC we call `DefaultUBS::sanitize` with a stub `contract_wasm` (empty slice). In production, the bridge should fetch the origin token contract, perform sandboxed analysis, compute deterministic report hashes, and persist the report in the registry before claims are accepted.
- Be mindful of high-cardinality metrics when exposing per-class labels in Prometheus; choose a label strategy that avoids unbounded cardinality.
- Upgrade-safe paths are enforced by using ACL-controlled `SYSTEM_WHITELIST` and append-only refactor records.

## Ledger invariants & tests

- Credit followed by equal debit restores owner balance when vec_geq holds.
- Debits never underflow: implementations must reject any debit where delta exceeds current balance per component.
- Global conservation: sum of all balances is conserved except when explicit mint or burn operations are invoked.
- These properties are enforced with unit tests and proptest suites in `crates/aln_core/src/energy_ledger_proptest.rs`.

## Future improvements

- Replace the stub `DefaultUBS::sanitize` with a comprehensive deterministic UBS pipeline; produce signed report objects and attestations.
- Integrate a light-client or on-chain verification of origin lock proofs for stronger provenance.
- Add more fuzz & property-based tests for ledger invariants under randomized sequences.
- Add L2/contract-level validations to prevent accidental misconfiguration in whitelists and error cases.

## How to call sealed refactor (contract example)

A claim via the bridge contract is the sealed refactor entrypoint. Example payload (CosmWasm JSON message):

```json
{
  "claim_with_origin": {
    "asset_id": "b1",
    "origin_event": { "origin_chain_id": "k1", "tx_hash": "tx123", "nonce": 1, "denom": "ibc/x", "origin_address": "u1", "amount":"100","height":0 },
    "merkle_proof": [],
    "ubs_report_hash": "h1",
    "amount_auet": "100",
    "amount_csp": null
  }
}
```

If UBS approves (or downgrades) the token, the bridge will credit the ledger with SECs. If UBS rejects, the claim is consumed, but no minting occurs.

## Querying ledger & audit

- `QueryMsg::EnergyBalance { address }` returns the internal ledger energy vector for a user.
- `QueryMsg::RefactorAudit { origin_chain, tx_hash, nonce }` returns the UBS report hash associated with a specific refactor event.


