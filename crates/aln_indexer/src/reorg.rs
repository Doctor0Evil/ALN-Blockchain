use anyhow::{Result, Context};
use sqlx::PgPool;
use chrono::Utc;

/// Checks the canonical chain for parent_hash mismatches and marks orphaned blocks and snapshots
pub async fn handle_reorg(pool: &PgPool, chain_id: i64, new_block_height: i64) -> Result<i64> {
    let span = tracing::span!(tracing::Level::INFO, "handle_reorg", chain_id = chain_id, height = new_block_height);
    let _enter = span.enter();
    println!("handle_reorg called for height {}", new_block_height);

    // Load canonical blocks ordered by height
    let rows = sqlx::query!("SELECT height, hash, parent_hash FROM blocks WHERE is_canonical = true ORDER BY height ASC")
        .fetch_all(pool)
        .await
        .context("fetch canonical blocks")?;

    if rows.is_empty() {
        println!("No canonical blocks present; nothing to do.");
        return Ok(());
    }

    // Walk chain and detect first mismatch
    let mut last_hash: Option<String> = None;
    let mut first_bad_height: Option<i64> = None;

    for r in rows.iter() {
        if let Some(ref last) = last_hash {
            if r.parent_hash != *last {
                first_bad_height = Some(r.height);
                break;
            }
        }
        last_hash = Some(r.hash.clone());
    }

    if let Some(first_bad) = first_bad_height {
        println!("Reorg detected; marking canonical blocks >= {} as orphan", first_bad);
        let mut tx = pool.begin().await.context("begin reorg tx")?;

        sqlx::query!("UPDATE blocks SET is_canonical = false WHERE height >= $1 AND is_canonical = true", first_bad)
            .execute(&mut tx)
            .await
            .context("mark blocks non-canonical")?;

        sqlx::query!("UPDATE balance_snapshot SET is_orphan = true WHERE block_height >= $1", first_bad)
            .execute(&mut tx)
            .await
            .context("mark snapshots orphan")?;

        // Optionally: mark denoms as orphan if only seen in orphaned blocks (left as future work)

        tx.commit().await.context("commit reorg tx")?;
        println!("Reorg handling complete at height {}", first_bad);
        // After marking orphaned rows, replay from common ancestor (first_bad - 1)
        let db = crate::db_postgres::PostgresDb::new(pool.clone());
        let replay_start = if first_bad > 0 { first_bad - 1 } else { 0 };
        let replayed = db.replay_from(chain_id, replay_start).await?;
        tracing::info!(replayed_blocks = replayed, "replayed blocks after reorg");
        return Ok(replayed);
    } else {
        println!("No reorg detected.");
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    #[tokio::test]
    async fn test_handle_reorg() {
        // stub test
    }
}
