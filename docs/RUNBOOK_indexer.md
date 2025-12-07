# ALN Indexer Runbook

This runbook describes indexer operations and tasks for `aln_indexer` crate.

Indexing flow:
1. Tail Kujira node via RPC / gRPC or stream blocks.
2. For each new block:
   - Upsert block into `blocks` table with height/hash/parent_hash.
   - If parent_hash mismatch, detect a reorg and call `reorg` flow to reconcile.
   - Extract balance changes and insert into `balance_snapshot`.
   - Resolve denoms (IBC denom trace) and store in `denom` table.
3. Periodically run `retention compact` for old windows to rollup and remove detailed snapshots.

Operational scripts:
- Run indexer follow-mode (tip tracking):
  ```bash
  cargo run -p aln_indexer -- follow-chain --reorg-window 200
  ```

- Run retention compaction manually:
  ```bash
  cargo run -p aln_indexer -- retention-compact --window-days 60
  ```

Monitoring:
- Monitor indexer run durations and `indexer_runs` table for failures.
- Monitor disk usage of Postgres and balance snapshot sizes.

Database migration example:
- Use SQL files in `crates/aln_indexer/migrations`.
- Run migrations before starting indexer: `sqlx migrate run` (if using `sqlx`).

Reorg guidance:
- The `reorg_window` is the number of tip blocks that can be rewritten; keep conservative (e.g., 200).
- For large reorgs, operator intervention may be required.

Provenance and DID:
- `did_identity` provides run-level DID provenance saved to `indexer_runs` table.
- Indexer runs should notarize run outputs with `did_provenance` tooling if artifacts are produced.
