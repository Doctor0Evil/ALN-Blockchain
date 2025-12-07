# ALN Indexer

This crate contains the indexer CLI for ALN. It supports retention compaction, block follow-chain ingestion, and reorg handling.

## Running

Set up a Postgres DB with the URL set in `DATABASE_URL` env variable, e.g.:

```
export DATABASE_URL=postgres://postgres:password@localhost:5432/aln_indexer_test
```

Run migrations (the CLI will automatically run migrations for retention and follow-chain commands):

```
cargo run -p aln_indexer -- retention-compact --window-days 7
```

Follow chain against a running RPC node:

```
export RPC_URL=http://127.0.0.1:26657
export START_HEIGHT=1
cargo run -p aln_indexer -- follow-chain
```

## Tests

CI runs `indexer-tests` which creates a Postgres service, runs migrations, loads fixtures, and executes the integration tests. Locally you can run the integration tests as:

```
export DATABASE_URL=postgres://postgres:password@localhost:5432/aln_indexer_test
psql -U postgres -d aln_indexer_test -f crates/aln_indexer/migrations/V1__init.sql
psql -U postgres -d aln_indexer_test -f crates/aln_indexer/migrations/V2__rollup_unique_index.sql
psql -U postgres -d aln_indexer_test -f crates/aln_indexer/fixtures/fixture.sql
cargo test -p aln_indexer -- --nocapture
```

The tests exercise retention compaction, reorg handling, and basic pagination logic.
