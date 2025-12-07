use anyhow::{Result, Context};
use sqlx::PgPool;
use reqwest::Client;
use crate::reorg::handle_reorg;
use std::sync::Arc;
use crate::metrics::Metrics;

#[tracing::instrument(skip(pool, metrics), fields(chain_rpc = rpc_url, start_height = start_height))]
pub async fn follow_chain(pool: &PgPool, rpc_url: &str, start_height: i64, reorg_window: i64, metrics: Option<Arc<tokio::sync::RwLock<Metrics>>>) -> Result<()> {
    let client = Client::new();
    // Fetch status to find latest height
    let status_url = format!("{}/status", rpc_url.trim_end_matches('/'));
    let status: serde_json::Value = client.get(&status_url).send().await?.json().await?;
    let latest_height = status["result"]["sync_info"]["latest_block_height"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
    println!("Chain latest height {}",(latest_height));

    let mut height = start_height;
    if height == 0 {
        height = 1;
    }

    while height <= latest_height {
        let block_url = format!("{}/block?height={}", rpc_url.trim_end_matches('/'), height);
        let resp: serde_json::Value = client.get(&block_url).send().await?.json().await?;

        // extract block header
        let header = &resp["result"]["block"]["header"];
        let hash = header["app_hash"].as_str().unwrap_or("").to_string();
        let parent_hash = header["last_block_id"]["hash"].as_str().unwrap_or("").to_string();

        // Insert or update block record
        let _ = sqlx::query!(
            "INSERT INTO blocks (height, hash, parent_hash, indexed_at, is_canonical) VALUES ($1,$2,$3,now(),true) ON CONFLICT (height) DO UPDATE SET hash = EXCLUDED.hash, parent_hash = EXCLUDED.parent_hash, is_canonical = true",
            height,
            hash,
            parent_hash
        )
        .execute(pool)
        .await
        .context("insert block")?;

        // call reorg handler if necessary - using chain_id=1 for default
        let replayed = crate::reorg::handle_reorg(pool, 1, height).await?;
        if replayed > 0 {
            if let Some(m) = &metrics {
                m.write().await.reorg_events_total.inc();
                m.write().await.replayed_blocks_total.inc_by(replayed as u64);
            }
        }
        // update metrics
        if let Some(m) = &metrics {
            let mut mlock = m.write().await;
            mlock.indexed_blocks_total.inc();
            mlock.indexer_head_height.set(height as i64);
        }

        height += 1;
    }

    Ok(())
}
