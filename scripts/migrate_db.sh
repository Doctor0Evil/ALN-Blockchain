#!/usr/bin/env bash
set -euo pipefail
DB_URL=${DATABASE_URL:-"postgres://postgres:password@localhost:5432/aln_indexer_test"}
# Apply global schema then indexer migrations
psql "$DB_URL" -f db/schema.sql
psql "$DB_URL" -f db/retention_compaction.sql
psql "$DB_URL" -f crates/aln_indexer/migrations/V1__init.sql
psql "$DB_URL" -f crates/aln_indexer/migrations/V2__rollup_unique_index.sql
psql "$DB_URL" -f crates/aln_indexer/migrations/V3__tx_and_indexer_state.sql

echo "Migrations applied to $DB_URL"
