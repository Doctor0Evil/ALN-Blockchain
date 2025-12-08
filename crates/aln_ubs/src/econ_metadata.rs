use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconMetadata {
    pub risk_score: f64,
}

pub fn analyze_denom(_token_addr: &str) -> EconMetadata {
    // Deterministic rule: use hash of token string to compute a risk score
    EconMetadata { risk_score: 0.5 }
}
