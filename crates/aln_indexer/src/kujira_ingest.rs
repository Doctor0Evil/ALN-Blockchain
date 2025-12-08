use anyhow::{Result, Context};
use crate::db_postgres::{Db, BlockHeader};
use serde_json::Value;
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
                // Check tx payloads for bridge events
                for tx in txs.iter() {
                    if tx.contains("RefactoredAsset") || tx.contains("claim_refactored") || tx.contains("action\":\"claim\"") {
                        m.write().await.aln_bridge_events_total.inc();
                    }
                    if tx.contains("\"toxic\":true") {
                    if tx.contains("claim_refactored") {
                        m.write().await.sealed_refactor_total.inc();
                    }
                    if tx.contains("claim_rejected") || tx.contains("refactor_rejected") {
                        m.write().await.sealed_refactor_rejected_total.inc();
                    }
                        // optimistic parsing: increment toxic gauge by 1 (use real amount parsing in production)
                        let cur = m.write().await.aln_energy_toxic_total.get();
                        m.write().await.aln_energy_toxic_total.set(cur + 1);
                    }
                    if tx.contains("\"toxic\":false") {
                        let cur = m.write().await.aln_energy_clean_total.get();
                        m.write().await.aln_energy_clean_total.set(cur + 1);
                    }
                }
            }

            // parse tx payloads to index token classes and mints/burns
            for txraw in txs.iter() {
                // try parse JSON representation
                if let Ok(val) = serde_json::from_str::<Value>(txraw) {
                    // recursively search for register_asset payload
                    fn find_register_asset(v: &Value) -> Option<&Value> {
                        match v {
                            Value::Object(map) => {
                                if map.contains_key("register_asset") { return Some(&map["register_asset"]); }
                                for (_k, vv) in map.iter() {
                                    if let Some(found) = find_register_asset(vv) { return Some(found); }
                                }
                                None
                            }
                            Value::Array(arr) => {
                                for item in arr.iter() { if let Some(found) = find_register_asset(item) { return Some(found); } }
                                None
                            }
                            _ => None
                        }
                    }

                    if let Some(reg) = find_register_asset(&val) {
                        // `reg` is an object that contains `asset: { ... }` structure
                        if let Some(asset) = reg.get("asset") {
                            let id = asset.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                            let source_denom = asset.get("source_denom").and_then(|v| v.as_str()).unwrap_or("");
                            let name = source_denom; // fallback, contract may not set a name field
                            let symbol = source_denom;
                            let params_json = asset.to_string();
                            let creator = "registry";
                            let is_transferable = true;
                            // insert token class
                            let _ = db_impl.insert_token_class(id, name, symbol, &params_json, creator, is_transferable).await;
                        }
                    }

                    // detect mint or burn messages referencing class_id
                    fn find_mint_burn(v: &Value) -> Option<(String, String, String)> {
                        match v {
                            Value::Object(map) => {
                                if map.contains_key("mint") {
                                    // mint may have class_id and amount under mint
                                    let m = &map["mint"];
                                    let class_id = m.get("class_id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                                    let amt = m.get("amount").and_then(|s| s.as_str()).unwrap_or("0").to_string();
                                    return Some(("mint".to_string(), class_id, amt));
                                }
                                if map.contains_key("burn") {
                                    let m = &map["burn"];
                                    let class_id = m.get("class_id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                                    let amt = m.get("amount").and_then(|s| s.as_str()).unwrap_or("0").to_string();
                                    return Some(("burn".to_string(), class_id, amt));
                                }
                                for (_k, vv) in map.iter() { if let Some(found) = find_mint_burn(vv) { return Some(found); } }
                                None
                            }
                            Value::Array(arr) => {
                                for item in arr.iter() { if let Some(found) = find_mint_burn(item) { return Some(found); } }
                                None
                            }
                            _ => None
                        }
                    }

                    if let Some((action, class_id, amt)) = find_mint_burn(&val) {
                        if class_id != "" {
                            if action == "mint" {
                                let _ = db_impl.record_class_mint(&class_id, &amt, b.height).await;
                                if let Some(m) = &metrics {
                                    if let Ok(n) = amt.parse::<u64>() {
                                        m.write().await.class_mint_total.with_label_values(&[&class_id]).inc_by(n);
                                    } else {
                                        m.write().await.class_mint_total.with_label_values(&[&class_id]).inc();
                                    }
                                }
                            } else if action == "burn" {
                                let _ = db_impl.record_class_burn(&class_id, &amt, b.height).await;
                                if let Some(m) = &metrics {
                                    if let Ok(n) = amt.parse::<u64>() {
                                        m.write().await.class_burn_total.with_label_values(&[&class_id]).inc_by(n);
                                    } else {
                                        m.write().await.class_burn_total.with_label_values(&[&class_id]).inc();
                                    }
                                }
                            }
                        }
                    }

                    // detect burn messages (we reused find_mint_burn for both)
                    // detect toxic flag in asset payload (if present) and mark class
                    fn find_toxic(v: &Value) -> Option<(String,bool)> {
                        match v {
                            Value::Object(map) => {
                                if map.contains_key("toxic") && map.contains_key("class_id") {
                                    let class_id = map.get("class_id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                                    let t = map.get("toxic").and_then(|s| s.as_bool()).unwrap_or(false);
                                    return Some((class_id, t));
                                }
                                for (_k, vv) in map.iter() { if let Some(found) = find_toxic(vv) { return Some(found); } }
                                None
                            }
                            Value::Array(arr) => {
                                for item in arr.iter() { if let Some(found) = find_toxic(item) { return Some(found); } }
                                None
                            }
                            _ => None
                        }
                    }

                    if let Some((class_id, t)) = find_toxic(&val) {
                        if class_id != "" {
                            let _ = db_impl.set_class_toxic(&class_id, t).await;
                            if let Some(m) = &metrics {
                                m.write().await.class_toxic_gauge.with_label_values(&[&class_id]).set(if t { 1 } else { 0 });
                            }
                        }
                    }
                } else {
                    // fallback: heuristic string-search detection
                    if txraw.contains("register_asset") {
                        // best-effort id extraction
                        if let Some(start) = txraw.find("\"id\":") {
                            let s = &txraw[start+6..];
                            if let Some(end) = s.find('"') {
                                let id = s[0..end].trim_matches('"');
                                let _ = db_impl.insert_token_class(id, id, id, "{}", "registry", true).await;
                            }
                        }
                    }
                    if txraw.contains("\"mint\"") && txraw.contains("class_id") {
                        // naive parse: find class_id and amount tokens
                        if let Some(start) = txraw.find("class_id") {
                            let s = &txraw[start..];
                            if let Some(cid_start) = s.find(':') { let s2 = &s[cid_start+1..]; if let Some(q) = s2.find('"') { let cid = &s2[q+1..]; if let Some(q2) = cid.find('"') { let class_id = &cid[..q2]; let _ = db_impl.record_class_mint(class_id, "0", b.height).await; } } }
                        }
                    }
                }
            }
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
