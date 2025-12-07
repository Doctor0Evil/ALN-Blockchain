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

Observability (logging & metrics):
- Structured JSON logs are enabled by default via `tracing_subscriber::fmt().json()` and are controlled with the `RUST_LOG` environment variable. Example to enable info logs:
  ```bash
  export RUST_LOG=info
  ```
  For PowerShell on Windows:
  ```powershell
  $env:RUST_LOG = 'info'
  ```
- Prometheus metrics endpoint is exposed by starting the indexer with `METRICS_ADDR` env var or by providing `--metrics-addr` if supported. Example:
  ```bash
  METRICS_ADDR=127.0.0.1:9888 cargo run -p aln_indexer -- Ingest --mode follow
  ```
/metrics will include `aln_blocks_ingested_total`, `aln_reorg_events_total`, `aln_replay_blocks_total`, and more; add Prometheus scrape config accordingly.

Database migration example:
- Use SQL files in `crates/aln_indexer/migrations`.
- Run migrations before starting indexer: `sqlx migrate run` (if using `sqlx`).

Reorg guidance:
- The `reorg_window` is the number of tip blocks that can be rewritten; keep conservative (e.g., 200).
- For large reorgs, operator intervention may be required.

Replay / reindex runbook:
- To mark a range as non-canonical and replay from a height using RPC, run:
  ```bash
  # simple replay from height 100 using the default RPC_URL or pass --rpc-endpoint
  cargo run -p aln_indexer -- ReplayFrom --chain-id 1 --from-height 100 --rpc-endpoint http://127.0.0.1:26657
  ```
- Pre-conditions: Ensure RPC endpoint is reachable and the node provides historical blocks required for the replay. This operation marks blocks `>= from-height` as non-canonical and replays canonical blocks from the RPC, rebuilding snapshots and rollups.
- After replay: Verify DB invariants
  - Count canonical blocks: `SELECT COUNT(*) FROM blocks WHERE is_canonical = true;`
  - Ensure `indexer_state` last canonical height matches the expected tip: `SELECT last_canonical_height, last_canonical_hash FROM indexer_state;`

Provenance and DID:
- `did_identity` provides run-level DID provenance saved to `indexer_runs` table.
- Indexer runs should notarize run outputs with `did_provenance` tooling if artifacts are produced.
