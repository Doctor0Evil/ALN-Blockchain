#[cfg(test)]
mod tests {
    use super::super::*;
    use anyhow::Result;
    
    use crate::schema::*;
    use sqlx::migrate::Migrator;
    static MIGRATOR: Migrator = sqlx::migrate!();

    async fn setup_db() -> Result<sqlx::PgPool> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
        let pool = sqlx::PgPool::connect(&url).await?;
        MIGRATOR.run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn retention_runs() -> Result<()> {
        let pool = match setup_db().await {
            Ok(p) => p,
            Err(e) => { eprintln!("skipping retention test: {}", e); return Ok(()); }
        };

        // Insert sample blocks and snapshots
        sqlx::query!("INSERT INTO account(address) VALUES ('acct1') ON CONFLICT DO NOTHING").execute(&pool).await?;
        let account_id: i64 = sqlx::query_scalar!("SELECT id FROM account WHERE address = 'acct1'").fetch_one(&pool).await?;
        sqlx::query!("INSERT INTO denom(raw_denom) VALUES ('token1') ON CONFLICT DO NOTHING").execute(&pool).await?;
        let denom_id: i64 = sqlx::query_scalar!("SELECT id FROM denom WHERE raw_denom = 'token1'").fetch_one(&pool).await?;

        // insert blocks 100..106
        for h in 100..106 {
            sqlx::query!("INSERT INTO blocks(height, hash, parent_hash, is_canonical) VALUES ($1,$2,$3,true) ON CONFLICT (height) DO NOTHING", h as i64, format!("hash{}", h), format!("hash{}", h-1)).execute(&pool).await?;
            sqlx::query!("INSERT INTO balance_snapshot(block_height, account_id, denom_id, amount) VALUES ($1,$2,$3,$4)", h as i64, account_id, denom_id, "100.0").execute(&pool).await?;
        }

        // Run retention for small window (assuming BLOCKS_PER_DAY constant -> 2880)
        // Since cutoff = max - window*2880 -> For small max 105 and window 0 -> cutoff 105
        crate::retention::retention_compact(&pool, 0).await?;

        // Check that balance_snapshot rows <= cutoff were deleted
        let remaining: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM balance_snapshot").fetch_one(&pool).await?;
        assert_eq!(remaining, 0);

        // Check rollup entries exist
        let rr: (i64,) = sqlx::query_as!( (i64,), "SELECT COUNT(*) FROM balance_rollup" ).fetch_one(&pool).await?;
        assert!(rr.0 > 0);
        Ok(())
    }

    #[tokio::test]
    async fn reorg_runs() -> Result<()> {
        let pool = match setup_db().await { Ok(p) => p, Err(e) => { eprintln!("skipping reorg test: {}", e); return Ok(()); } };
        // Insert a canonical chain
        for h in 1..6 {
            sqlx::query!("INSERT INTO blocks(height, hash, parent_hash, is_canonical) VALUES ($1,$2,$3,true) ON CONFLICT (height) DO NOTHING", h as i64, format!("h{}", h), if h==1 {"".to_string()} else {format!("h{}", h-1)}).execute(&pool).await?;
            sqlx::query!("INSERT INTO account(address) VALUES ('acct{}') ON CONFLICT DO NOTHING", h).execute(&pool).await?;
            let account_id: i64 = sqlx::query_scalar!("SELECT id FROM account WHERE address = $1", format!("acct{}", 1)).fetch_one(&pool).await?;
            let denom_id: i64 = sqlx::query_scalar!("SELECT id FROM denom WHERE raw_denom = 'token1'").fetch_one(&pool).await?;
            sqlx::query!("INSERT INTO balance_snapshot(block_height, account_id, denom_id, amount) VALUES ($1,$2,$3,$4)", h as i64, account_id, denom_id, "10").execute(&pool).await?;
        }

        // Simulate reorg: create a new block at height 3 with an unexpected parent (hash mismatch) and set it canonical
        sqlx::query!("INSERT INTO blocks(height, hash, parent_hash, is_canonical) VALUES ($1,$2,$3,true) ON CONFLICT (height) DO UPDATE SET hash=$2,parent_hash=$3,is_canonical=true", 3_i64, "new_h3", "x1").execute(&pool).await?;
        // Call handle_reorg
        crate::reorg::handle_reorg(&pool, 1, 3).await?;
        // Blocks 3..5 should be marked non-canonical
        let non_canonical_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM blocks WHERE is_canonical = false").fetch_one(&pool).await?;
        assert!(non_canonical_count >= 3);
        // Snapshots for those heights should be marked orphan
        let orphan_snapshots: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM balance_snapshot WHERE is_orphan = true").fetch_one(&pool).await?;
        assert!(orphan_snapshots >= 3);
        Ok(())
    }
}
