use anyhow::{Result, Context};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub chain_id: i64,
    pub height: i64,
    pub hash: String,
    pub parent_hash: String,
}

pub trait Db {
    fn get_head<'a>(&'a self, chain_id: i64) -> futures::future::BoxFuture<'a, Result<Option<BlockHeader>>>;
    fn get_block_by_hash<'a>(&'a self, chain_id: i64, hash: &'a str) -> futures::future::BoxFuture<'a, Result<Option<BlockHeader>>>;
    fn mark_block_canonical<'a>(&'a self, chain_id: i64, hash: &'a str, canonical: bool) -> futures::future::BoxFuture<'a, Result<()>>;
    fn mark_tx_canonical_by_block<'a>(&'a self, chain_id: i64, block_hash: &'a str, canonical: bool) -> futures::future::BoxFuture<'a, Result<()>>;
    fn insert_block_and_txs<'a>(&'a self, header: BlockHeader, raw_json: &'a str, txs_json: &'a [String]) -> futures::future::BoxFuture<'a, Result<()>>;
    fn update_indexer_state_head<'a>(&'a self, chain_id: i64, height: i64, hash: &'a str) -> futures::future::BoxFuture<'a, Result<()>>;
    fn mark_range_non_canonical<'a>(&'a self, chain_id: i64, from_height: i64) -> futures::future::BoxFuture<'a, Result<()>>;
    fn replay_from<'a>(&'a self, chain_id: i64, start_height: i64) -> futures::future::BoxFuture<'a, Result<i64>>;
    fn insert_token_class<'a>(&'a self, class_id: &'a str, name: &'a str, symbol: &'a str, params_json: &'a str, creator: &'a str, is_transferable: bool) -> futures::future::BoxFuture<'a, Result<()>>;
    fn record_class_mint<'a>(&'a self, class_id: &'a str, amount: &'a str, block_height: i64) -> futures::future::BoxFuture<'a, Result<()>>;
    fn record_class_burn<'a>(&'a self, class_id: &'a str, amount: &'a str, block_height: i64) -> futures::future::BoxFuture<'a, Result<()>>;
    fn set_class_toxic<'a>(&'a self, class_id: &'a str, toxic: bool) -> futures::future::BoxFuture<'a, Result<()>>;
}

pub struct PostgresDb {
    pool: PgPool,
}

impl PostgresDb {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

impl Db for PostgresDb {
    fn get_head<'a>(&'a self, chain_id: i64) -> futures::future::BoxFuture<'a, Result<Option<BlockHeader>>> {
        Box::pin(async move {
            let row = sqlx::query!("SELECT height, hash, parent_hash FROM blocks WHERE is_canonical = true ORDER BY height DESC LIMIT 1")
                .fetch_optional(&self.pool).await.context("fetch head")?;
            if let Some(r) = row {
                Ok(Some(BlockHeader { chain_id, height: r.height.unwrap_or(0), hash: r.hash, parent_hash: r.parent_hash }))
            } else { Ok(None) }
        })
    }

    fn get_block_by_hash<'a>(&'a self, chain_id: i64, hash: &'a str) -> futures::future::BoxFuture<'a, Result<Option<BlockHeader>>> {
        Box::pin(async move {
            let r = sqlx::query!("SELECT height, hash, parent_hash FROM blocks WHERE hash = $1 LIMIT 1", hash)
                .fetch_optional(&self.pool).await.context("get block by hash")?;
            if let Some(row) = r {
                Ok(Some(BlockHeader { chain_id, height: row.height.unwrap_or(0), hash: row.hash, parent_hash: row.parent_hash }))
            } else { Ok(None) }
        })
    }

    fn mark_block_canonical<'a>(&'a self, chain_id: i64, hash: &'a str, canonical: bool) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            sqlx::query!("UPDATE blocks SET is_canonical = $1 WHERE hash = $2", canonical, hash).execute(&self.pool).await.context("mark block canonical")?;
            Ok(())
        })
    }

    fn mark_tx_canonical_by_block<'a>(&'a self, chain_id: i64, block_hash: &'a str, canonical: bool) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            // resolve block height by hash
            let b = sqlx::query_scalar!("SELECT height FROM blocks WHERE hash = $1", block_hash).fetch_optional(&self.pool).await.context("get block height")?;
            if let Some(height) = b {
                sqlx::query!("UPDATE tx SET is_canonical = $1 WHERE block_height = $2", canonical, height).execute(&self.pool).await.context("update tx canonical")?;
            }
            Ok(())
        })
    }

    fn insert_block_and_txs<'a>(&'a self, header: BlockHeader, raw_json: &'a str, txs_json: &'a [String]) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            // insert block if not exists
            sqlx::query!("INSERT INTO blocks (height, hash, parent_hash, indexed_at, is_canonical) VALUES ($1,$2,$3,now(),true) ON CONFLICT (height) DO UPDATE SET hash = EXCLUDED.hash, parent_hash = EXCLUDED.parent_hash, is_canonical = true", header.height, header.hash, header.parent_hash)
                .execute(&self.pool).await.context("insert block")?;

            // insert txs
            for (i, tx) in txs_json.iter().enumerate() {
                sqlx::query!("INSERT INTO tx (block_height, tx_hash, idx_in_block, raw_json, is_canonical) VALUES ($1,$2,$3,$4,true) ON CONFLICT (block_height, tx_hash) DO NOTHING", header.height, format!("tx:{}:{}", header.hash, i as i32), i as i32, tx).execute(&self.pool).await.context("insert tx")?;
            }
            Ok(())
        })
    }

    fn update_indexer_state_head<'a>(&'a self, chain_id: i64, height: i64, hash: &'a str) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            sqlx::query!("INSERT INTO indexer_state (last_canonical_height, last_canonical_hash, updated_at) VALUES ($1,$2,now()) ON CONFLICT DO UPDATE SET last_canonical_height = $1, last_canonical_hash = $2, updated_at = now()", height, hash).execute(&self.pool).await.context("update indexer state")?;
            Ok(())
        })
    }

    // Replays canonical chain from height+1 to latest for this chain, rebuilding snapshots
    fn replay_from<'a>(&'a self, chain_id: i64, start_height: i64) -> futures::future::BoxFuture<'a, Result<i64>> {
        Box::pin(async move {
            // get latest canonical height
            let latest: i64 = sqlx::query_scalar!("SELECT COALESCE(MAX(height), 0) FROM blocks WHERE is_canonical = true").fetch_one(&self.pool).await.context("fetch latest canonical height")?;
            if latest <= start_height {
                return Ok(());
            }

            // fetch canonical blocks >= start_height+1
            let rows = sqlx::query!("SELECT height, hash FROM blocks WHERE is_canonical = true AND height >= $1 ORDER BY height ASC", start_height + 1)
                .fetch_all(&self.pool).await.context("fetch canonical blocks for replay")?;

            // For idempotency, delete snapshots for those heights and recreate deterministic snapshots
            // Get a small list of accounts and denoms for synthetic snapshot generation
            let accounts = sqlx::query!("SELECT id, address FROM account ORDER BY id ASC LIMIT 5").fetch_all(&self.pool).await.context("fetch accounts")?;
            let denoms = sqlx::query!("SELECT id, raw_denom FROM denom ORDER BY id ASC LIMIT 3").fetch_all(&self.pool).await.context("fetch denoms")?;

            for r in rows.iter() {
                let h = r.height.unwrap_or(0);
                let mut tx = self.pool.begin().await.context("begin replay tx")?;
                sqlx::query!("DELETE FROM balance_snapshot WHERE block_height = $1", h).execute(&mut tx).await.context("delete old snapshots")?;
                for a in accounts.iter() {
                    for d in denoms.iter() {
                        let amount = format!("{}.{:03}", h, (a.id % 1000));
                        sqlx::query!("INSERT INTO balance_snapshot(block_height, account_id, denom_id, amount) VALUES ($1,$2,$3,$4)", h, a.id, d.id, amount).execute(&mut tx).await.context("insert synthetic snapshot")?;
                    }
                }
                // recompute rollup for affected bucket
                let period_start = (h / 2880) * 2880;
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
                sqlx::query(rollup_insert).bind(2880_i64).bind(h).execute(&mut tx).await.context("recompute rollup")?;

                tx.commit().await.context("commit replay tx")?;
            }

            Ok(rows.len() as i64)
        })
    }

    fn mark_range_non_canonical<'a>(&'a self, chain_id: i64, from_height: i64) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            // mark blocks >= from_height as non-canonical (keep data)
            sqlx::query!("UPDATE blocks SET is_canonical = false WHERE height >= $1", from_height).execute(&self.pool).await.context("mark blocks non canonical")?;
            // mark tx as non-canonical for the same range
            sqlx::query!("UPDATE tx SET is_canonical = false WHERE block_height >= $1", from_height).execute(&self.pool).await.context("mark tx non canonical")?;
            // update indexer_state to reflect last canonical height = from_height - 1
            let last_height = if from_height > 0 { from_height - 1 } else { 0 };
            let last_hash: Option<String> = sqlx::query_scalar!("SELECT hash FROM blocks WHERE height = $1 LIMIT 1", last_height).fetch_optional(&self.pool).await.context("fetch last canonical hash")?;
            let last_hash_str = last_hash.unwrap_or_else(|| "".to_string());
            sqlx::query!("INSERT INTO indexer_state (last_canonical_height, last_canonical_hash, updated_at) VALUES ($1,$2,now()) ON CONFLICT DO UPDATE SET last_canonical_height = $1, last_canonical_hash = $2, updated_at = now()", last_height, last_hash_str).execute(&self.pool).await.context("update indexer state after mark non canonical")?;
            Ok(())
        })
    }

    fn insert_token_class<'a>(&'a self, class_id: &'a str, name: &'a str, symbol: &'a str, params_json: &'a str, creator: &'a str, is_transferable: bool) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            sqlx::query!("INSERT INTO token_class (class_id, name, symbol, params, creator, is_transferable, created_at) VALUES ($1,$2,$3,$4,$5,$6,now()) ON CONFLICT (class_id) DO UPDATE SET name = EXCLUDED.name, symbol = EXCLUDED.symbol, params = EXCLUDED.params, creator = EXCLUDED.creator, is_transferable = EXCLUDED.is_transferable", class_id, name, symbol, params_json, creator, is_transferable)
                .execute(&self.pool).await.context("insert token class")?;
            // ensure stat row exists
            sqlx::query!("INSERT INTO class_stats (class_id, total_minted, total_burned, toxic) VALUES ($1,'0','0',false) ON CONFLICT (class_id) DO NOTHING", class_id).execute(&self.pool).await.context("ensure class stat row")?;
            Ok(())
        })
    }

    fn record_class_mint<'a>(&'a self, class_id: &'a str, amount: &'a str, block_height: i64) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            sqlx::query!("INSERT INTO class_mint (class_id, amount, block_height, created_at) VALUES ($1,$2,$3,now())", class_id, amount, block_height).execute(&self.pool).await.context("insert class mint")?;
            // update class_stats; amounts stored as text -> numeric
            sqlx::query!("INSERT INTO class_stats (class_id, total_minted, total_burned, toxic) VALUES ($1,$2,'0',false) ON CONFLICT (class_id) DO UPDATE SET total_minted = (COALESCE(class_stats.total_minted::numeric,0) + $2::numeric)::text", class_id, amount).execute(&self.pool).await.context("update class total minted")?;
            Ok(())
        })
    }

    fn record_class_burn<'a>(&'a self, class_id: &'a str, amount: &'a str, block_height: i64) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            sqlx::query!("INSERT INTO class_mint (class_id, amount, block_height, created_at) VALUES ($1,$2,$3,now())", class_id, amount, block_height).execute(&self.pool).await.context("insert class burn row")?;
            sqlx::query!("INSERT INTO class_stats (class_id, total_minted, total_burned, toxic) VALUES ($1,'0',$2,false) ON CONFLICT (class_id) DO UPDATE SET total_burned = (COALESCE(class_stats.total_burned::numeric,0) + $2::numeric)::text", class_id, amount).execute(&self.pool).await.context("update class total burned")?;
            Ok(())
        })
    }

    fn set_class_toxic<'a>(&'a self, class_id: &'a str, toxic: bool) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            sqlx::query!("INSERT INTO class_stats (class_id, total_minted, total_burned, toxic) VALUES ($1,'0','0',$2) ON CONFLICT (class_id) DO UPDATE SET toxic = $2", class_id, toxic).execute(&self.pool).await.context("set class toxic")?;
            Ok(())
        })
    }
}
