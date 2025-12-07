-- Fixture loader for aln_indexer Integration Tests
BEGIN;

INSERT INTO account (address)
SELECT 'acct' || i
FROM generate_series(1,10) AS s(i)
ON CONFLICT (address) DO NOTHING;

INSERT INTO denom (raw_denom)
SELECT 'token' || i
FROM generate_series(1,3) AS s(i)
ON CONFLICT (raw_denom) DO NOTHING;

-- Create blocks 1..500
INSERT INTO blocks(height, hash, parent_hash, is_canonical)
SELECT i, 'h' || i, CASE WHEN i=1 THEN '' ELSE 'h' || (i-1) END, true
FROM generate_series(1,500) AS s(i)
ON CONFLICT (height) DO NOTHING;

-- Insert snapshots for each block for a subset of accounts & denoms
INSERT INTO balance_snapshot(block_height, account_id, denom_id, amount)
SELECT b.height, a.id, d.id, (i % 100 + 1)::text
FROM blocks b
JOIN account a ON a.id <= 5
JOIN denom d ON d.id <= 2
CROSS JOIN generate_series(1,5) AS s(i)
WHERE b.height BETWEEN 1 AND 500;

COMMIT;
