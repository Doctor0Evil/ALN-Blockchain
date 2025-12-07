#!/usr/bin/env bash
set -euo pipefail

DB_URL=${DATABASE_URL:-"postgres://postgres:password@localhost:5432/aln_indexer_test"}
RPC_URL=${RPC_URL:-"http://127.0.0.1:26657"}
START_HEIGHT=${START_HEIGHT:-1}

export DATABASE_URL="$DB_URL"

# Apply migrations locally
./scripts/migrate_db.sh

# Build and run indexer follow-chain
cd crates/aln_indexer
cargo run --release -- follow-chain

# (Optionally run retention compaction)
# cargo run --release -- retention-compact --window-days 7

