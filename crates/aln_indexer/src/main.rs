use clap::{Parser, Subcommand};
use anyhow::Result;
use sqlx::PgPool;
use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use crate::follow_chain::follow_chain;
use crate::metrics::Metrics;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::db_postgres::PostgresDb;
use crate::kujira_ingest::ingest_kujira_chain;
use warp::Filter;
use tracing_subscriber;
use sqlx::migrate::Migrator;
static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Parser)]
#[command(name = "aln_indexer")]
struct Cli {
    /// Optional metrics server address to enable Prometheus scraping (e.g., 127.0.0.1:9888)
    #[arg(long)]
    pub metrics_addr: Option<String>,
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    RetentionCompact { window_days: i64 },
    Ingest { mode: Option<String>, start_height: Option<i64>, stop_height: Option<i64>, lag_blocks: Option<i64> },
    ReplayFrom { chain_id: i64, from_height: i64, rpc_endpoint: Option<String> },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with JSON output
    tracing_subscriber::fmt().json().flatten_event(true).with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let cli = Cli::parse();
    match cli.cmd {
        Commands::RetentionCompact { window_days } => {
            println!("Running retention compact for {} days", window_days);
            // connect to DB
            let pool_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/aln_indexer_test".to_string());
            let pool = PgPool::connect(&pool_url).await.expect("db connect");
            // Run migrations and Insert indexer_runs entry
            MIGRATOR.run(&pool).await.expect("migrations failed");
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
        Commands::Ingest { mode, start_height, stop_height, lag_blocks } => {
            let mode = mode.unwrap_or_else(|| "follow".to_string());
            let start_height = start_height.unwrap_or_else(|| std::env::var("START_HEIGHT").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1));
            let stop_height = stop_height.unwrap_or(0);
            let lag_blocks = lag_blocks.unwrap_or_else(|| std::env::var("LAG_BLOCKS").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(10));
            println!("Ingest mode {} start {} stop {} lag {}", mode, start_height, stop_height, lag_blocks);
            let pool_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/aln_indexer_test".to_string());
            let pool = PgPool::connect(&pool_url).await.expect("db connect");
            // default start height and reorg window
            let start_height = std::env::var("START_HEIGHT").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
            let reorg_window = reorg_window.unwrap_or(10);
            MIGRATOR.run(&pool).await.expect("migrations failed");
            // Use Postgres DB-backed ingest
            let db_impl = PostgresDb::new(pool.clone());
            // Start metrics HTTP server for Prometheus scraping (opt-in via CLI flag or env var)
            let metrics = Metrics::new();
            let metrics_clone = metrics.clone();
            let metrics_filter = warp::path("metrics").and_then(move || {
                let m = metrics_clone.clone();
                async move {
                    let s = m.gather().await;
                    Ok::<_, std::convert::Infallible>(warp::reply::with_header(s, "content-type", "text/plain; version=0.0.4"))
                }
            });
            // CLI flag `metrics_addr` takes precedence over environment variable
            let metrics_addr = cli.metrics_addr.clone().or_else(|| std::env::var("METRICS_ADDR").ok());
            if let Some(addr) = metrics_addr {
                let addr_clone = addr.clone();
                tokio::spawn(async move {
                    warp::serve(metrics_filter).run(addr_clone.parse().unwrap()).await;
                });
            }
            // spawn a compaction loop running every minute (in production tune this)
            let pool2 = pool.clone();
            let metrics2 = metrics.clone();
            tokio::spawn(async move {
                loop {
                    let safe_height: i64 = std::env::var("COMPACT_SAFE_HEIGHT_LAG").ok().and_then(|s| s.parse().ok()).unwrap_or(20);
                    // compute safe height as latest head - lag
                    if let Ok(maxh) = sqlx::query_scalar!("SELECT COALESCE(MAX(height), 0) FROM blocks").fetch_one(&pool2).await {
                        let safe = if maxh > safe_height { maxh - safe_height } else { 0 };
                        if safe > 0 {
                            // call stored procedure
                            let _ = sqlx::query!("SELECT compact_indexer_retention($1, $2)", 1_i64, safe).execute(&pool2).await;
                            // update metrics
                            metrics2.write().await.last_compacted_height.set(safe);
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            });
            // run ingestion (blocking), choose mode
            let rpc = std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:26657".to_string());
            // run ingestion (blocking), choose mode
            if mode == "backfill" {
                crate::kujira_ingest::backfill_kujira_chain(&db_impl, 1, &rpc, start_height, stop_height).await.expect("backfill failed");
            } else {
                follow_chain(&pool, &rpc, start_height, lag_blocks as i64, Some(metrics.clone())).await.expect("follow_chain failed");
                crate::kujira_ingest::ingest_kujira_chain(&pool, &db_impl, 1, &rpc, lag_blocks as i64, Some(metrics.clone())).await.expect("ingest_kujira_chain failed");
            }
        }
        Commands::ReplayFrom { chain_id, from_height, rpc_endpoint } => {
            let pool_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/aln_indexer_test".to_string());
            let pool = PgPool::connect(&pool_url).await.expect("db connect");
            MIGRATOR.run(&pool).await.expect("migrations failed");
            let db_impl = PostgresDb::new(pool.clone());
            let rpc = rpc_endpoint.unwrap_or_else(|| std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:26657".to_string()));
            let reprocessed = crate::replay_reindex::replay_from_height(&db_impl, chain_id, from_height, &rpc).await.expect("replay failed");
            println!("Reprocessed {} blocks", reprocessed);
        }
    }
    Ok(())
}
