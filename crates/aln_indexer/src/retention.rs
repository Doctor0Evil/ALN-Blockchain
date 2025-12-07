use anyhow::{Result, Context};
use sqlx::PgPool;
use sqlx::Acquire;
use std::time::Duration;

const BLOCKS_PER_DAY: i64 = 2880; // approx; configurable in future

pub async fn retention_compact(pool: &PgPool, window_days: i64) -> Result<()> {
    // Compute the cutoff height: blocks older or equal to this will be compacted
    let max_height: i64 = sqlx::query_scalar!("SELECT COALESCE(MAX(height), 0) FROM blocks")
        .fetch_one(pool)
        .await
        .context("failed to fetch max block height")?;

    if max_height == 0 {
        println!("No blocks found; nothing to compact");
        return Ok(());
    }

    let cutoff = max_height - (window_days * BLOCKS_PER_DAY);
    if cutoff <= 0 {
        println!("Cutoff <= 0; nothing to compact for window {} days", window_days);
        return Ok(());
    }

    println!("Retention compaction: cutoff height {} (max {})", cutoff, max_height);

    // Use DB transaction
    let mut tx = pool.begin().await.context("begin tx")?;

    // Create rollup aggregates grouped by day bucket (period_start)
    // period_start will be floor(block_height / BLOCKS_PER_DAY) * BLOCKS_PER_DAY
    let rollup_insert = r#"
    WITH rollups AS (
      SELECT ((block_height / $1) * $1) AS period_start, account_id, denom_id, SUM((amount)::numeric) AS sum_amount
      FROM balance_snapshot
      WHERE block_height <= $2
      GROUP BY period_start, account_id, denom_id
    )
    INSERT INTO balance_rollup (period_start, account_id, denom_id, amount)
    SELECT period_start, account_id, denom_id, (sum_amount::text) FROM rollups
    ON CONFLICT (period_start, account_id, denom_id) DO UPDATE
       SET amount = ((COALESCE(balance_rollup.amount::numeric,0) + EXCLUDED.amount::numeric)::text)
    "#;

    // Ensure an index/unique constraint exists; if not, this will still attempt the insert.
    sqlx::query(rollup_insert)
        .bind(BLOCKS_PER_DAY)
        .bind(cutoff)
        .execute(&mut tx)
        .await
        .context("failed to insert rollup aggregates")?;

    // Delete compacted snapshots older than cutoff
    let delete_sql = r#"DELETE FROM balance_snapshot WHERE block_height <= $1"#;
    sqlx::query(delete_sql)
        .bind(cutoff)
        .execute(&mut tx)
        .await
        .context("failed to delete old snapshots")?;

    tx.commit().await.context("commit tx")?;
    println!("Retention compaction completed up to height {}", cutoff);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use tokio;
    
    #[tokio::test]
    async fn test_retention_compact() {
        // This test is a stub; in CI we'll spin up a Postgres DB for integration tests
    }
}
