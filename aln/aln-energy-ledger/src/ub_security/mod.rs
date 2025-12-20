use std::time::SystemTime;

/// Basic replay key structure for use in replay protection stores.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayKey {
    pub replay_key: Vec<u8>, // canonical key / hash
    pub first_seen_at: SystemTime,
    pub last_seen_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub source: Option<String>,
    pub chain_id: Option<i64>,
    pub nonce: Option<i64>,
}

/// Decisions returned by a ReplayProtectionStore when checking/inserting keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayDecision {
    Fresh,
    Duplicate,
    Expired,
}

/// Trait defining a replay protection store.
#[async_trait::async_trait]
pub trait ReplayProtectionStore: Send + Sync + 'static {
    /// Check the provided replay key and insert or update meta as needed.
    async fn check_and_insert(&self, key: &ReplayKey) -> Result<ReplayDecision, Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(feature = "ub_security")]
pub mod postgres {
    use super::*;
    use sqlx::Pool;
    use sqlx::Postgres;

    pub struct PostgresReplayProtectionStore {
        pool: Pool<Postgres>,
    }

    impl PostgresReplayProtectionStore {
        pub fn new(pool: Pool<Postgres>) -> Self {
            Self { pool }
        }
    }

    #[async_trait::async_trait]
    impl ReplayProtectionStore for PostgresReplayProtectionStore {
        async fn check_and_insert(&self, _key: &ReplayKey) -> Result<ReplayDecision, Box<dyn std::error::Error + Send + Sync>> {
            // TODO: implement using sqlx queries
            todo!("Postgres replay protection is not implemented yet");
        }
    }
}
