use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticReport {
    pub categories: Vec<String>,
}

pub fn analyze_contract(_wasm: &[u8]) -> StaticReport {
    // Simple deterministic heuristic: in a real implementation, parse wasm / AST
    StaticReport { categories: vec!["no_mint_detected".to_string()] }
}
