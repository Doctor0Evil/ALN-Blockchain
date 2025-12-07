use clap::{Parser, Subcommand};
use anyhow::Result;
use sqlx::PgPool;
use chrono::Utc;

#[derive(Parser)]
#[command(name = "aln_indexer")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    RetentionCompact { window_days: i64 },
    FollowChain { reorg_window: Option<i64> },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::RetentionCompact { window_days } => {
            println!("Running retention compact for {} days", window_days);
            // connect to DB
            let pool_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/aln_indexer_test".to_string());
            let pool = PgPool::connect(&pool_url).await.expect("db connect");
            // Insert indexer_runs entry
            let did = crate::did_identity::load_did(None).unwrap_or_else(|_| "did:unknown:local".to_string());
            let started = Utc::now();
            let run_id: i64 = sqlx::query_scalar!("INSERT INTO indexer_runs(did, started_at, status, git_commit) VALUES ($1, $2, $3, $4) RETURNING run_id", did, started, "running", "unknown").fetch_one(&pool).await.unwrap_or(0);
            let res = crate::retention::retention_compact(&pool, window_days).await;
            let finished = Utc::now();
            match res {
                Ok(_) => { sqlx::query!("UPDATE indexer_runs SET finished_at = $1, status = $2 WHERE run_id = $3", finished, "ok", run_id).execute(&pool).await.ok(); }
                Err(e) => { sqlx::query!("UPDATE indexer_runs SET finished_at = $1, status = $2 WHERE run_id = $3", finished, "error", run_id).execute(&pool).await.ok(); eprintln!("retention failed: {:?}", e); }
            }
        }
        Commands::FollowChain { reorg_window } => {
            println!("Follow-chain with reorg window: {:?}", reorg_window);
        }
    }
    Ok(())
}
