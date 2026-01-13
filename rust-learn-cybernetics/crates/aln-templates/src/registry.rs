use crate::{AlnClauseId, ValidClause};
use bioscale_evidence::EvidenceTag;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseBinding {
    pub clause: ValidClause,
    pub required_tags: Vec<EvidenceTag>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClauseRegistry {
    pub bindings: Vec<ClauseBinding>,
}

impl ClauseRegistry {
    pub fn register(&mut self, binding: ClauseBinding) {
        self.bindings.push(binding);
    }
}
