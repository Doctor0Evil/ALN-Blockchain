CREATE TABLE IF NOT EXISTS chain (
    id               BIGSERIAL PRIMARY KEY,
    name             TEXT NOT NULL UNIQUE,
    network_id       TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS block (
    id               BIGSERIAL PRIMARY KEY,
    chain_id         BIGINT NOT NULL REFERENCES chain(id),
    height           BIGINT NOT NULL,
    hash             TEXT NOT NULL,
    parent_hash      TEXT NOT NULL,
    timestamp        TIMESTAMPTZ NOT NULL,
    is_canonical     BOOLEAN NOT NULL DEFAULT TRUE,
    raw_json         JSONB NOT NULL,
    CONSTRAINT uq_block_chain_height UNIQUE (chain_id, height),
    CONSTRAINT uq_block_chain_hash UNIQUE (chain_id, hash)
);

CREATE INDEX IF NOT EXISTS idx_block_chain_parent
    ON block (chain_id, parent_hash);

CREATE TABLE IF NOT EXISTS tx (
    id               BIGSERIAL PRIMARY KEY,
    chain_id         BIGINT NOT NULL REFERENCES chain(id),
    block_id         BIGINT NOT NULL REFERENCES block(id),
    tx_hash          TEXT NOT NULL,
    idx_in_block     INT NOT NULL,
    raw_json         JSONB NOT NULL,
    is_canonical     BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT uq_tx_chain_hash UNIQUE (chain_id, tx_hash)
);

CREATE INDEX IF NOT EXISTS idx_tx_block
    ON tx (block_id);

CREATE TABLE IF NOT EXISTS indexer_state (
    id               BIGSERIAL PRIMARY KEY,
    chain_id         BIGINT NOT NULL REFERENCES chain(id),
    last_canonical_height BIGINT NOT NULL DEFAULT 0,
    last_canonical_hash   TEXT   NOT NULL DEFAULT '',
    safe_height          BIGINT NOT NULL DEFAULT 0,
    finalized_height     BIGINT NOT NULL DEFAULT 0,
    last_compacted_height BIGINT NOT NULL DEFAULT 0,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_indexer_state_chain UNIQUE (chain_id)
);

CREATE TABLE IF NOT EXISTS block_archive (
    id               BIGSERIAL PRIMARY KEY,
    chain_id         BIGINT NOT NULL,
    height           BIGINT NOT NULL,
    hash             TEXT NOT NULL,
    parent_hash      TEXT NOT NULL,
    timestamp        TIMESTAMPTZ NOT NULL,
    raw_json         JSONB NOT NULL,
    archived_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tx_archive (
    id               BIGSERIAL PRIMARY KEY,
    chain_id         BIGINT NOT NULL,
    block_height     BIGINT NOT NULL,
    tx_hash          TEXT NOT NULL,
    idx_in_block     INT NOT NULL,
    raw_json         JSONB NOT NULL,
    archived_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
