#[cfg(test)]
mod db_tests {
    use aln_indexer::db_postgres::{PostgresDb, Db, BlockHeader};
    use anyhow::Result as AnyResult;
    use sqlx::PgPool;
    use sqlx::migrate::Migrator;
    static MIGRATOR: Migrator = sqlx::migrate!();

    async fn setup_pool() -> Result<PgPool, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await?;
        MIGRATOR.run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn test_insert_block_and_reorg() -> AnyResult<()> {
        let pool = setup_pool().await?;
        let db = PostgresDb::new(pool.clone());

        // insert blocks 1..5
        for h in 1..=5 {
            let header = BlockHeader { chain_id: 1, height: h, hash: format!("h{}", h), parent_hash: if h==1 { "".into() } else { format!("h{}", h-1) } };
            db.insert_block_and_txs(header.clone(), "{}", &Vec::new()).await?;
        }
        // head should be height 5
        let head = db.get_head(1).await?;
        assert!(head.is_some());
        assert_eq!(head.unwrap().height, 5);

        // insert reorg: new block at height 3 with different parent
        let h3 = BlockHeader { chain_id:1, height:3, hash:"new_h3".into(), parent_hash:"x".into() };
        db.insert_block_and_txs(h3.clone(), "{}", &Vec::new()).await?;
        // calling mark_block_canonical(false) on old chain above common ancestor
        db.mark_block_canonical(1, "h4", false).await?;
        db.mark_tx_canonical_by_block(1, "h4", false).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_token_class_and_mint_burn_flow() -> AnyResult<()> {
        let pool = setup_pool().await?;
        let db = PostgresDb::new(pool.clone());
        // insert token class
        db.insert_token_class("aln-test", "Test Token", "TST", "{}", "creator", true).await?;
        // record mint
        db.record_class_mint("aln-test", "100", 10).await?;
        // record burn
        db.record_class_burn("aln-test", "25", 11).await?;
        // mark toxic
        db.set_class_toxic("aln-test", true).await?;
        // validate rows
        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM token_class WHERE class_id = 'aln-test'").fetch_one(&pool).await?;
        assert_eq!(count, 1);
        let minted: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM class_mint WHERE class_id = 'aln-test'").fetch_one(&pool).await?;
        assert_eq!(minted, 2); // we inserted both mint and burn rows in class_mint table as history
        let toxic: bool = sqlx::query_scalar!("SELECT toxic FROM class_stats WHERE class_id = 'aln-test'").fetch_one(&pool).await?;
        assert!(toxic);
        Ok(())
    }
}
