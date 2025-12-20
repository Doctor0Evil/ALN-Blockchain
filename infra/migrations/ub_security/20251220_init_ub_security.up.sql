-- Create ub_replay_keys table
CREATE TABLE IF NOT EXISTS ub_replay_keys (
    id BIGSERIAL PRIMARY KEY,
    replay_key BYTEA NOT NULL UNIQUE,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NULL,
    source TEXT,
    chain_id BIGINT NULL,
    nonce BIGINT NULL
);

CREATE INDEX IF NOT EXISTS idx_ub_replay_keys_replay_key ON ub_replay_keys (replay_key);
