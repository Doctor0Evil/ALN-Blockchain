CREATE OR REPLACE FUNCTION compact_indexer_retention(
    p_chain_id BIGINT,
    p_safe_height BIGINT
) RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO block_archive (
        chain_id,
        height,
        hash,
        parent_hash,
        timestamp,
        raw_json,
        archived_at
    )
    SELECT
        b.chain_id,
        b.height,
        b.hash,
        b.parent_hash,
        b.timestamp,
        b.raw_json,
        NOW()
    FROM block b
    WHERE b.chain_id = p_chain_id
      AND b.height <= p_safe_height
      AND b.is_canonical = FALSE
      AND NOT EXISTS (
          SELECT 1
          FROM block_archive ba
          WHERE ba.chain_id = b.chain_id
            AND ba.hash = b.hash
      );

    INSERT INTO tx_archive (
        chain_id,
        block_height,
        tx_hash,
        idx_in_block,
        raw_json,
        archived_at
    )
    SELECT
        t.chain_id,
        b.height,
        t.tx_hash,
        t.idx_in_block,
        t.raw_json,
        NOW()
    FROM tx t
    JOIN block b ON t.block_id = b.id
    WHERE t.chain_id = p_chain_id
      AND b.height <= p_safe_height
      AND t.is_canonical = FALSE
      AND NOT EXISTS (
          SELECT 1
          FROM tx_archive ta
          WHERE ta.chain_id = t.chain_id
            AND ta.tx_hash = t.tx_hash
      );

    DELETE FROM tx
    USING block
    WHERE tx.block_id = block.id
      AND block.chain_id = p_chain_id
      AND block.height <= p_safe_height
      AND tx.is_canonical = FALSE;

    DELETE FROM block
    WHERE chain_id = p_chain_id
      AND height <= p_safe_height
      AND is_canonical = FALSE;

    UPDATE indexer_state
    SET last_compacted_height = GREATEST(last_compacted_height, p_safe_height),
        updated_at = NOW()
    WHERE chain_id = p_chain_id;
END;
$$;

CREATE OR REPLACE FUNCTION indexer_archive_hash(p_chain_id BIGINT)
RETURNS TEXT
LANGUAGE plpgsql
AS $$
DECLARE
    v_concat BYTEA := ''::BYTEA;
    v_hash   TEXT;
BEGIN
    SELECT string_agg(hash::TEXT, '' ORDER BY height)
    INTO v_hash
    FROM block_archive
    WHERE chain_id = p_chain_id;

    IF v_hash IS NULL THEN
        RETURN NULL;
    END IF;

    SELECT encode(digest(v_hash, 'sha256'), 'hex') INTO v_hash;
    RETURN v_hash;
END;
$$;
