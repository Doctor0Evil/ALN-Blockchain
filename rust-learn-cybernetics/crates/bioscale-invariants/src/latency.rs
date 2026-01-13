use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LatencyInvariantError {
    #[error("latency invariant violated: {actual_ms} ms > {limit_ms} ms")]
    Exceeded { actual_ms: u64, limit_ms: u64 },
}

pub fn always_within_latency_ms(actual_ms: u64, limit_ms: u64) -> Result<(), LatencyInvariantError> {
    if actual_ms > limit_ms {
        Err(LatencyInvariantError::Exceeded { actual_ms, limit_ms })
    } else {
        Ok(())
    }
}
