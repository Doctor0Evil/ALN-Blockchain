-- Add tx and indexer_state tables for aln_indexer

CREATE TABLE IF NOT EXISTS tx (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  tx_hash TEXT NOT NULL,
  idx_in_block INT NOT NULL,
  raw_json JSONB NOT NULL,
  is_canonical BOOLEAN DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_tx_block ON tx (block_height);

CREATE TABLE IF NOT EXISTS indexer_state (
  id BIGSERIAL PRIMARY KEY,
  last_canonical_height BIGINT DEFAULT 0,
  last_canonical_hash TEXT DEFAULT '',
  safe_height BIGINT DEFAULT 0,
  finalized_height BIGINT DEFAULT 0,
  last_compacted_height BIGINT DEFAULT 0,
  updated_at TIMESTAMPTZ DEFAULT NOW()
);
