-- Initial migrations for aln_indexer

CREATE TABLE IF NOT EXISTS blocks (
  height BIGINT PRIMARY KEY,
  hash TEXT NOT NULL,
  parent_hash TEXT NOT NULL,
  indexed_at TIMESTAMP DEFAULT now(),
  is_canonical BOOLEAN DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS denom (
  id SERIAL PRIMARY KEY,
  raw_denom TEXT NOT NULL UNIQUE,
  ibc_hash TEXT,
  path TEXT,
  base_denom TEXT,
  logo_uri TEXT,
  coingecko_id TEXT,
  risk_score NUMERIC(5,2),
  is_orphan BOOLEAN DEFAULT FALSE,
  last_seen_height BIGINT
);

CREATE TABLE IF NOT EXISTS account (
  id BIGSERIAL PRIMARY KEY,
  address TEXT NOT NULL UNIQUE,
  created_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS balance_snapshot (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  account_id BIGINT NOT NULL REFERENCES account(id),
  denom_id BIGINT NOT NULL REFERENCES denom(id),
  amount TEXT NOT NULL,
  is_orphan BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS balance_rollup (
  id BIGSERIAL PRIMARY KEY,
  period_start BIGINT NOT NULL,
  account_id BIGINT NOT NULL REFERENCES account(id),
  denom_id BIGINT NOT NULL REFERENCES denom(id),
  amount TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS indexer_runs (
  run_id BIGSERIAL PRIMARY KEY,
  did TEXT NOT NULL,
  started_at TIMESTAMP NOT NULL DEFAULT now(),
  finished_at TIMESTAMP,
  status TEXT NOT NULL,
  git_commit TEXT
);
