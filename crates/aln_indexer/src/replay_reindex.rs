use crate::db_postgres::{Db, BlockHeader};
use anyhow::Result;
use reqwest;
use serde_json::Value;
use tracing::instrument;

#[instrument(skip(db, rpc_endpoint))]
pub async fn replay_from_height<D: Db + Sync>(db: &D, chain_id: i64, from_height: i64, rpc_endpoint: &str) -> Result<i64> {
    // mark the range as non-canonical in the DB
    db.mark_range_non_canonical(chain_id, from_height).await?;

    let client = reqwest::Client::new();
    let status: Value = client.get(format!("{}/status", rpc_endpoint)).send().await?.json().await?;
    let latest: i64 = status["result"]["sync_info"]["latest_block_height"].as_str().unwrap_or("0").parse().unwrap_or(0);

    let mut h = from_height;
    while h <= latest {
        let resp: Value = client.get(format!("{}/block?height={}", rpc_endpoint, h)).send().await?.json().await?;
        let result = &resp["result"];
        let hash = result["block_id"]["hash"].as_str().unwrap_or_default().to_string();
        let parent_hash = result["block"]["header"]["last_block_id"]["hash"].as_str().unwrap_or_default().to_string();
        let height: i64 = result["block"]["header"]["height"].as_str().unwrap_or("0").parse().unwrap_or(0);
        let txs: Vec<String> = result["block"]["data"]["txs"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
        let raw_block = result.to_string();

        let header = BlockHeader { chain_id, height, hash: hash.clone(), parent_hash };
        db.insert_block_and_txs(header, &raw_block, &txs).await?;
        db.update_indexer_state_head(chain_id, height, &hash).await?;
        h += 1;
    }
    Ok((latest - from_height + 1).max(0))
}
