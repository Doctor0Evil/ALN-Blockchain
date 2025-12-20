use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use aln_energy_ledger::sealed_refactor::ReplayVerifier;
use aln_energy_ledger::ub_security::{ReplayDecision, ReplayKey, ReplayProtectionStore};

#[derive(Clone)]
struct MockStore {
    inner: Arc<Mutex<HashMap<Vec<u8>, ReplayKey>>>,
}

impl MockStore {
    fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[async_trait::async_trait]
impl ReplayProtectionStore for MockStore {
    async fn check_and_insert(&self, key: &ReplayKey) -> Result<ReplayDecision, Box<dyn std::error::Error + Send + Sync>> {
        let mut m = self.inner.lock().unwrap();
        if let Some(existing) = m.get(&key.replay_key) {
            // check expiry
            if let Some(exp) = existing.expires_at {
                if exp <= SystemTime::now() {
                    return Ok(ReplayDecision::Expired);
                }
            }
            return Ok(ReplayDecision::Duplicate);
        }
        // insert
        m.insert(key.replay_key.clone(), key.clone());
        Ok(ReplayDecision::Fresh)
    }
}

#[tokio::test]
async fn first_time_key_is_fresh() {
    let store = MockStore::new();
    let verifier = ReplayVerifier::new(store.clone());

    let key = ReplayKey {
        replay_key: b"k1".to_vec(),
        first_seen_at: SystemTime::now(),
        last_seen_at: SystemTime::now(),
        expires_at: None,
        source: Some("test".to_string()),
        chain_id: None,
        nonce: None,
    };

    let res = verifier.check_and_insert(&key).await.unwrap();
    assert_eq!(res, ReplayDecision::Fresh);
}

#[tokio::test]
async fn reused_key_is_duplicate() {
    let store = MockStore::new();
    let verifier = ReplayVerifier::new(store.clone());

    let key = ReplayKey {
        replay_key: b"k2".to_vec(),
        first_seen_at: SystemTime::now(),
        last_seen_at: SystemTime::now(),
        expires_at: None,
        source: Some("test".to_string()),
        chain_id: None,
        nonce: None,
    };

    let _ = verifier.check_and_insert(&key).await.unwrap();
    let res = verifier.check_and_insert(&key).await.unwrap();
    assert_eq!(res, ReplayDecision::Duplicate);
}

#[tokio::test]
async fn expired_key_is_expired() {
    let store = MockStore::new();
    let verifier = ReplayVerifier::new(store.clone());

    let expired = ReplayKey {
        replay_key: b"k3".to_vec(),
        first_seen_at: SystemTime::now() - Duration::from_secs(3600),
        last_seen_at: SystemTime::now() - Duration::from_secs(3600),
        expires_at: Some(SystemTime::now() - Duration::from_secs(10)),
        source: Some("test".to_string()),
        chain_id: None,
        nonce: None,
    };

    // Insert directly into store to simulate prior insertion
    store.inner.lock().unwrap().insert(expired.replay_key.clone(), expired.clone());

    let res = verifier.check_and_insert(&expired).await.unwrap();
    assert_eq!(res, ReplayDecision::Expired);
}
