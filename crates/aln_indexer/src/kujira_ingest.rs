use anyhow::{Result, Context};
use crate::db_postgres::{Db, BlockHeader};
use reqwest::Client;
use std::time::Duration;

/// Ingests Kujira chain: fetch status, fetch blocks up to lag, insert into DB using provided Db implementation
pub async fn ingest_kujira_chain<D: Db + Sync + Send + 'static>(pool: &sqlx::PgPool, db_impl: &D, chain_id: i64, rpc_endpoint: &str, lag_blocks: i64, metrics: Option<std::sync::Arc<tokio::sync::RwLock<crate::metrics::Metrics>>>) -> Result<()> {
    let client = Client::new();
    loop {
        let status = client.get(format!("{}/status", rpc_endpoint)).send().await?.json::<serde_json::Value>().await?;
        let latest_height = status["result"]["sync_info"]["latest_block_height"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
        let target_height = latest_height - lag_blocks;
        if target_height <= 0 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        }
        let head = db_impl.get_head(chain_id).await?;
        let mut next_height = match head { Some(h) => h.height + 1, None => 1 };
        while next_height <= target_height {
            let resp = client.get(format!("{}/block?height={}", rpc_endpoint, next_height)).send().await?.json::<serde_json::Value>().await?;
            let result = &resp["result"];
            let block_id = result["block_id"].clone();
            let header = &result["block"]["header"];
            let hash = block_id["hash"].as_str().unwrap_or("").to_string();
            let parent_hash = header["last_block_id"]["hash"].as_str().unwrap_or("").to_string();
            let height = header["height"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
            let txs = result["block"]["data"]["txs"].as_array().map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect::<Vec<_>>()).unwrap_or_else(Vec::new);
            let raw_block = result.to_string();
            let b = BlockHeader { chain_id, height, hash: hash.clone(), parent_hash: parent_hash.clone() };
            db_impl.insert_block_and_txs(b, &raw_block, &txs).await?;
            db_impl.update_indexer_state_head(chain_id, height, &hash).await?;

            // call reorg handler if necessary - using chain_id=1 for default
            let replayed = crate::reorg::handle_reorg(pool, chain_id, height).await?;
            if replayed > 0 {
                if let Some(m) = &metrics {
                    m.write().await.reorg_events_total.inc();
                    m.write().await.replayed_blocks_total.inc_by(replayed as u64);
                }
            }

            // update metrics for block
            if let Some(m) = &metrics {
                m.write().await.indexed_blocks_total.inc();
                m.write().await.indexer_head_height.set(height as i64);
            }

            next_height += 1;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
        if replayed > 0 {
            if let Some(m) = &metrics {
                m.write().await.reorg_events_total.inc();
                m.write().await.replayed_blocks_total.inc_by(replayed as u64);
            }
        }
        if let Some(m) = &metrics {
            m.write().await.indexed_blocks_total.inc();
            m.write().await.indexer_head_height.set(height as i64);
        }

/// Backfill blocks from start_height up to stop_height (inclusive). If stop_height==0, it will read the latest known height.
pub async fn backfill_kujira_chain<D: Db + Sync + Send + 'static>(db_impl: &D, chain_id: i64, rpc_endpoint: &str, start_height: i64, stop_height: i64) -> Result<()> {
    let client = Client::new();

    let mut target = stop_height;
    if stop_height == 0 {
        let status = client.get(format!("{}/status", rpc_endpoint)).send().await?.json::<serde_json::Value>().await?;
        target = status["result"]["sync_info"]["latest_block_height"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
    }

    let mut h = start_height;
    while h <= target {
        let resp = client.get(format!("{}/block?height={}", rpc_endpoint, h)).send().await?.json::<serde_json::Value>().await?;
        let result = &resp["result"];
        let block_id = result["block_id"].clone();
        let header = &result["block"]["header"];
        let hash = block_id["hash"].as_str().unwrap_or("").to_string();
        let parent_hash = header["last_block_id"]["hash"].as_str().unwrap_or("").to_string();
        let height = header["height"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
        let txs = result["block"]["data"]["txs"].as_array().map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect::<Vec<_>>()).unwrap_or_else(Vec::new);
        let raw_block = result.to_string();
        let b = BlockHeader { chain_id, height, hash: hash.clone(), parent_hash: parent_hash.clone() };
        db_impl.insert_block_and_txs(b, &raw_block, &txs).await?;
        db_impl.update_indexer_state_head(chain_id, height, &hash).await?;
        h += 1;
    }
    Ok(())
}
