#[cfg(test)]
mod tests {
    use super::super::*;
    use aln_indexer::db_postgres::PostgresDb;
    use sqlx::PgPool;
    use sqlx::migrate::Migrator;
    use tokio::sync::oneshot;
    use warp::Filter;

    static MIGRATOR: Migrator = sqlx::migrate!();

    async fn setup_pool() -> Result<PgPool, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await?;
        MIGRATOR.run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn test_ingest_kujira_chain_smoke() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        // Build mock server for /status and /block
        let (tx, rx) = oneshot::channel();
        let status = warp::path!("status").map(|| warp::reply::json(&serde_json::json!({
            "result": { "sync_info": { "latest_block_height": "3" } }
        })));
        let block = warp::path!("block").and(warp::query::query()).map(|params: std::collections::HashMap<String, String>| {
            let height = params.get("height").cloned().unwrap_or_else(|| "1".into());
            let result = serde_json::json!({
                "block_id": { "hash": format!("h{}", height) },
                "block": { "header": { "height": height.to_string(), "last_block_id": { "hash": format!("h{}", height.parse::<i64>().unwrap_or(1)-1) } }, "data": { "txs": null } }
            });
            warp::reply::json(&serde_json::json!({ "result": result }))
        });

        let routes = status.or(block);
        let (addr_tx, addr_rx) = oneshot::channel();
        tokio::spawn(async move {
            let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(([127,0,0,1], 0), async {
                rx.await.ok();
            });
            addr_tx.send(addr).ok();
            server.await;
        });
        let addr = addr_rx.await.unwrap();
        let rpc = format!("http://{}", addr);

        let db = PostgresDb::new(pool.clone());
        let ingest = aln_indexer::kujira_ingest::ingest_kujira_chain(&pool, &db, 1, &rpc, 0_i64, None);
        // Run for a short time then cancel
        tokio::select! {
            res = ingest => { res?; }
            _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {}
        }
        // Query DB for blocks inserted
        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM blocks").fetch_one(&pool).await?;
        assert!(count >= 1);
        // shutdown server
        tx.send(()).ok();
        Ok(())
    }

    #[tokio::test]
    async fn test_ingest_register_asset_and_mint() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        // Build mock server for /status and /block
        let (tx, rx) = oneshot::channel();
        let status = warp::path!("status").map(|| warp::reply::json(&serde_json::json!({
            "result": { "sync_info": { "latest_block_height": "2" } }
        })));
        let block = warp::path!("block").and(warp::query::query()).map(|params: std::collections::HashMap<String, String>| {
            let height = params.get("height").cloned().unwrap_or_else(|| "1".into());
            // craft a tx with register_asset and mint
            let tx = serde_json::json!({
                "body": {
                    "messages": [
                        { "wasm": { "execute": { "register_asset": { "asset": { "id": "aln-foo", "source_chain": "kujira", "source_denom": "foo", "snapshot_height": 100, "merkle_root": "abc", "ubs_report_hash": "deadbeef", "scaling_profile_id": "default", "activation_height": 0, "sanitized_approved": true } } } } },
                        { "wasm": { "execute": { "mint": { "class_id": "aln-foo", "amount": "100" } } } },
                        { "wasm": { "execute": { "set_toxic": { "class_id": "aln-foo", "toxic": true } } } }
                    ]
                }
            });
            let result = serde_json::json!({
                "block_id": { "hash": format!("h{}", height) },
                "block": { "header": { "height": height.to_string(), "last_block_id": { "hash": format!("h{}", height.parse::<i64>().unwrap_or(1)-1) } }, "data": { "txs": [tx.to_string()] } }
            });
            warp::reply::json(&serde_json::json!({ "result": result }))
        });

        let routes = status.or(block);
        let (addr_tx, addr_rx) = oneshot::channel();
        tokio::spawn(async move {
            let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(([127,0,0,1], 0), async {
                rx.await.ok();
            });
            addr_tx.send(addr).ok();
            server.await;
        });
        let addr = addr_rx.await.unwrap();
        let rpc = format!("http://{}", addr);

        let db = PostgresDb::new(pool.clone());
        let ingest = aln_indexer::kujira_ingest::ingest_kujira_chain(&pool, &db, 1, &rpc, 0_i64, None);
        // Run for a short time then cancel
        tokio::select! {
            res = ingest => { res?; }
            _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {}
        }
        // Query DB for token_class inserted
        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM token_class WHERE class_id = 'aln-foo'").fetch_one(&pool).await?;
        assert!(count == 1);
        // Query DB for class_mint
        let mint_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM class_mint WHERE class_id = 'aln-foo'").fetch_one(&pool).await?;
        assert!(mint_count >= 1);
        // Query DB for class_toxic
        let toxic: bool = sqlx::query_scalar!("SELECT toxic FROM class_stats WHERE class_id = 'aln-foo'").fetch_one(&pool).await?;
        assert!(toxic);
        // shutdown server
        tx.send(()).ok();
        Ok(())
    }
}
