#[cfg(test)]
mod tests {
    use super::super::*;
    use aln_indexer::db_postgres::PostgresDb;
    use sqlx::PgPool;
    use sqlx::migrate::Migrator;
    use tokio::sync::oneshot;
    use warp::Filter;
    use aln_indexer::replay_reindex::replay_from_height;
    static MIGRATOR: Migrator = sqlx::migrate!();

    async fn setup_pool() -> Result<PgPool, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await?;
        MIGRATOR.run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn test_replay_from_height_smoke() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let db = PostgresDb::new(pool.clone());

        // insert some initial canonical blocks (1..3)
        for h in 1..=3 {
            let header = aln_indexer::db_postgres::BlockHeader { chain_id: 1, height: h, hash: format!("h{}", h), parent_hash: format!("h{}", h-1) };
            db.insert_block_and_txs(header, &format!("{{}}"), &vec![]).await?;
            db.update_indexer_state_head(1, h, &format!("h{}", h)).await?;
        }

        // simulate a reorg where we want to mark >=2 non-canonical and replay from 2
        db.mark_range_non_canonical(1, 2).await?;
        let non_canonical_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM blocks WHERE is_canonical = false").fetch_one(&pool).await?;
        assert!(non_canonical_count >= 2);

        // Now start a mock RPC that serves blocks 2..4
        let (tx, rx) = oneshot::channel();
        let status = warp::path!("status").map(|| warp::reply::json(&serde_json::json!({"result": { "sync_info": { "latest_block_height": "4" } }})));
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

        // run replay
        let reprocessed = replay_from_height(&db, 1, 2, &rpc).await?;
        assert!(reprocessed >= 2);

        // validate DB: blocks >=2 are canonical true after replay
        let canon_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM blocks WHERE is_canonical = true AND height >= 2").fetch_one(&pool).await?;
        assert!(canon_count >= 2);
        tx.send(()).ok();
        Ok(())
    }

    #[tokio::test]
    async fn test_replay_deep_fork_and_idempotency() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let db = PostgresDb::new(pool.clone());

        // Insert canonical blocks 1..20
        for h in 1..=20 {
            let header = aln_indexer::db_postgres::BlockHeader { chain_id: 1, height: h, hash: format!("h{}", h), parent_hash: format!("h{}", h-1) };
            db.insert_block_and_txs(header, &format!("{{}}"), &vec![]).await?;
            db.update_indexer_state_head(1, h, &format!("h{}", h)).await?;
        }

        // Mark >=11 non-canonical to simulate reorg and replay from 11
        db.mark_range_non_canonical(1, 11).await?;

        // Prepare warp server returning a different branch for height >= 11 (h11b..h25b)
        let (tx, rx) = oneshot::channel();
        let status = warp::path!("status").map(|| warp::reply::json(&serde_json::json!({ "result": { "sync_info": { "latest_block_height": "25" } } })));
        let block = warp::path!("block").and(warp::query::query()).map(|params: std::collections::HashMap<String, String>| {
            let height = params.get("height").cloned().unwrap_or_else(|| "1".into());
            let hval = height.parse::<i64>().unwrap_or(1);
            let hash = if hval >= 11 { format!("h{}b", hval) } else { format!("h{}", hval) };
            let result = serde_json::json!({
                "block_id": { "hash": hash },
                "block": { "header": { "height": height.to_string(), "last_block_id": { "hash": format!("h{}", (hval - 1)) } }, "data": { "txs": null } }
            });
            warp::reply::json(&serde_json::json!({ "result": result }))
        });

        let routes = status.or(block);
        let (addr_tx, addr_rx) = oneshot::channel();
        tokio::spawn(async move {
            let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(([127,0,0,1], 0), async { rx.await.ok(); });
            addr_tx.send(addr).ok();
            server.await;
        });
        let addr = addr_rx.await.unwrap();
        let rpc = format!("http://{}", addr);

        // Run replay from 11 which should ingest h11b..h25b
        let reprocessed = replay_from_height(&db, 1, 11, &rpc).await?;
        assert!(reprocessed >= 15);

        // Verify that blocks >=11 are canonical and hash suffix 'b' present
        let rows = sqlx::query!("SELECT height, hash, is_canonical FROM blocks WHERE height >= $1 ORDER BY height ASC", 11_i64).fetch_all(&pool).await?;
        for r in rows.iter() {
            assert!(r.is_canonical.unwrap_or(false));
            assert!(r.hash.ends_with("b"));
        }

        // Call replay again (idempotent) with same RPC (same branch) and ensure no changes/errors
        let reprocessed2 = replay_from_height(&db, 1, 11, &rpc).await?;
        assert_eq!(reprocessed, reprocessed2);

        // Now update RPC to a new branch (h11c..h30c) and replay; hashes should change
        // We accomplish this by starting a new server on a different port
        let (tx2, rx2) = oneshot::channel();
        let status2 = warp::path!("status").map(|| warp::reply::json(&serde_json::json!({ "result": { "sync_info": { "latest_block_height": "30" } } })));
        let block2 = warp::path!("block").and(warp::query::query()).map(|params: std::collections::HashMap<String, String>| {
            let height = params.get("height").cloned().unwrap_or_else(|| "1".into());
            let hval = height.parse::<i64>().unwrap_or(1);
            let hash = if hval >= 11 { format!("h{}c", hval) } else { format!("h{}", hval) };
            let result = serde_json::json!({
                "block_id": { "hash": hash },
                "block": { "header": { "height": height.to_string(), "last_block_id": { "hash": format!("h{}", (hval - 1)) } }, "data": { "txs": null } }
            });
            warp::reply::json(&serde_json::json!({ "result": result }))
        });
        let routes2 = status2.or(block2);
        let (addr_tx2, addr_rx2) = oneshot::channel();
        tokio::spawn(async move {
            let (addr, server) = warp::serve(routes2).bind_with_graceful_shutdown(([127,0,0,1], 0), async { rx2.await.ok(); });
            addr_tx2.send(addr).ok();
            server.await;
        });
        let addr2 = addr_rx2.await.unwrap();
        let rpc2 = format!("http://{}", addr2);

        // Replay onto new branch
        let reprocessed3 = replay_from_height(&db, 1, 11, &rpc2).await?;
        assert!(reprocessed3 >= 20);

        // Verify blocks >=11 changed to new suffix c
        let rows3 = sqlx::query!("SELECT height, hash FROM blocks WHERE height >= $1 ORDER BY height ASC", 11_i64).fetch_all(&pool).await?;
        for r in rows3.iter() {
            assert!(r.hash.ends_with("c"));
        }
        tx.send(()).ok();
        tx2.send(()).ok();
        Ok(())
    }
}
