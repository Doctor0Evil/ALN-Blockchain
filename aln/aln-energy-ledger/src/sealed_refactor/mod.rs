use super::ub_security::{ReplayDecision, ReplayKey, ReplayProtectionStore};

/// A tiny sealed module pattern so only internal types may implement certain traits in future.
mod sealed {
    pub trait Sealed {}
}

/// Public trait that is implemented only for types we control (sealed pattern placeholder).
pub trait ReplayVerifiable: sealed::Sealed {}

/// Error type for verifier operations.
#[derive(Debug)]
pub enum Error {
    StoreError(String),
}

/// Result of verification actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDecisionExt {
    Fresh,
    Duplicate,
    Expired,
}

/// ReplayVerifier holds a store and provides check_and_insert semantics.
pub struct ReplayVerifier<S: ReplayProtectionStore> {
    store: S,
}

impl<S: ReplayProtectionStore> ReplayVerifier<S> {
    /// Create a new verifier from any ReplayProtectionStore
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Check and insert a replay key. Returns a ReplayDecision (Fresh, Duplicate, Expired)
    ///
    /// Intended flow (high level):
    /// 1. Query the store for existence and expiry of the key.
    /// 2. If not present and not expired, insert and return Fresh.
    /// 3. If present, return Duplicate.
    /// 4. If expired, return Expired.
    pub async fn check_and_insert(&self, key: &ReplayKey) -> Result<ReplayDecision, Error> {
        // Implementation will delegate to the store; for now forward the store result or convert errors.
        match self.store.check_and_insert(key).await {
            Ok(d) => Ok(d),
            Err(e) => Err(Error::StoreError(format!("{}", e))),
        }
    }
}
