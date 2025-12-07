use serde::{Serialize, Deserialize};

// Simple sqlx-friendly structs for migrations and use in code.

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockRow {
    pub height: i64,
    pub hash: String,
    pub parent_hash: String,
    pub indexed_at: Option<String>,
    pub is_canonical: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DenomRow {
    pub id: i64,
    pub raw_denom: String,
    pub ibc_hash: Option<String>,
    pub path: Option<String>,
    pub base_denom: Option<String>,
    pub logo_uri: Option<String>,
    pub coingecko_id: Option<String>,
    pub risk_score: Option<f64>,
    pub is_orphan: bool,
    pub last_seen_height: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BalanceSnapshotRow {
    pub id: i64,
    pub block_height: i64,
    pub account_id: i64,
    pub denom_id: i64,
    pub amount: String,
    pub is_orphan: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BalanceRollupRow {
    pub id: i64,
    pub period_start: i64,
    pub account_id: i64,
    pub denom_id: i64,
    pub amount: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexerRunRow {
    pub run_id: i64,
    pub did: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub git_commit: Option<String>,
}

// Migrations directory path: crates/aln_indexer/migrations
