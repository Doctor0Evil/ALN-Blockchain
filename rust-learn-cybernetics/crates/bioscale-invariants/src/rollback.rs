use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RollbackInvariantError {
    #[error("evidence not preserved across rollback")]
    EvidenceLost,
}

pub fn rollback_always_preserves_evidence(before: &[u8], after: &[u8]) -> Result<(), RollbackInvariantError> {
    if before == after {
        Ok(())
    } else {
        Err(RollbackInvariantError::EvidenceLost)
    }
}
