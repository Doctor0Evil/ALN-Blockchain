# Indexer Overview

This document describes the high-level architecture of the ALN indexer (`crates/aln_indexer`).

- **Ingestion**: tail blocks from Kujira via RPC/gRPC, parse bank module and IBC events.
- **Normalization**: map IBC `ibc/HASH` denoms to `(path, base_denom, ibc_hash)` via `denom_trace`.
- **Storage**: postgresql tables:
  - `blocks`, `denom`, `account`, `balance_snapshot`, `balance_rollup`, `indexer_runs`.
- **Retention & compaction**: rollups older than window to `balance_rollup` and delete raw snapshots.
- **Reorg handling**: detect mismatches in `parent_hash`, find common ancestor, mark orphan rows and reindex.
  - Use `cargo run -p aln_indexer -- ReplayFrom --chain-id 1 --from-height <height> --rpc-endpoint http://127.0.0.1:26657` to mark a range non-canonical and replay from that height using RPC to reindex canonical blocks.
  - Verify `indexer_state` and `blocks` canonical flags after replay.
- **Pagination**: keyset pagination implemented in `pagination.rs` for large data sets.
- **Provenance**: each indexer run records DID/commit to `indexer_runs` and optionally emits artifact provenance.
 - **Observability**: indexer exposes Prometheus metrics via `/metrics` when `METRICS_ADDR` env var is set. Logs are emitted as JSON via `tracing_subscriber` and controlled by `RUST_LOG`.

Operators should consult `RUNBOOK_indexer.md` for operational commands and scheduling.
