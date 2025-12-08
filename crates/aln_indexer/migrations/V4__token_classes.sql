-- Add token class tracking

CREATE TABLE IF NOT EXISTS token_class (
    id BIGSERIAL PRIMARY KEY,
    class_id TEXT NOT NULL UNIQUE,
    name TEXT,
    symbol TEXT,
    params JSONB,
    creator TEXT,
    is_transferable BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS class_mint (
    id BIGSERIAL PRIMARY KEY,
    class_id TEXT NOT NULL,
    amount TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS class_stats (
    class_id TEXT PRIMARY KEY,
    total_minted TEXT NOT NULL DEFAULT '0',
    total_burned TEXT NOT NULL DEFAULT '0',
    toxic BOOLEAN DEFAULT FALSE
);
