-- Add unique index to avoid duplicate rollups

CREATE UNIQUE INDEX IF NOT EXISTS idx_balance_rollup_unique ON balance_rollup(period_start, account_id, denom_id);
