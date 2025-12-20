# UB Security (Replay Protection) — Design

Purpose
- Provide replay protection for sealed refactor events and other ledger ingress points.
- Prevent repeated consumption or double-counting of the same external-origin proof.

Core components
- `ub_replay_keys` table (Postgres) — stores unique replay keys with timestamps and optional expiry.
- `ub_security` module (`aln-energy-ledger::ub_security`) — defines `ReplayKey`, `ReplayDecision`, and the `ReplayProtectionStore` trait.
- Postgres-backed store (`PostgresReplayProtectionStore`) — implemented using `sqlx` behind the `ub_security` feature flag.
- `sealed_refactor` module — uses `ReplayVerifier` to enforce one-time-use semantics for incoming bridge/ingress proofs.

DB Schema
- Table: `ub_replay_keys`
  - `id` BIGSERIAL PRIMARY KEY
  - `replay_key` BYTEA NOT NULL UNIQUE
  - `first_seen_at` TIMESTAMPTZ NOT NULL DEFAULT now()
  - `last_seen_at` TIMESTAMPTZ NOT NULL DEFAULT now()
  - `expires_at` TIMESTAMPTZ NULL
  - `source` TEXT
  - `chain_id` BIGINT NULL
  - `nonce` BIGINT NULL

Migrations are in `infra/migrations/ub_security/20251220_init_ub_security.up.sql` and `.down.sql`.

DID linkage and identity
- Public DID (`ALN_PRIMARY_DID`) in `infra/identity/did-config.json` acts as a non-secret identity label in CI and service logs.
- DO NOT store private keys or credentials in the repo; CI should read credentials from secure stores and only use the DID as a public tag.

Operational notes
- The `PostgresReplayProtectionStore` constructor accepts a `sqlx::Pool<Postgres>` and should be created by top-level services that manage DB connectivity.
- In tests and during early dev, a mock in-memory `ReplayProtectionStore` (HashMap) is used so unit tests do not require a running DB.

See code in `aln/aln-energy-ledger/src/ub_security` and `aln/aln-energy-ledger/src/sealed_refactor` for the current implementation and TODOs.
