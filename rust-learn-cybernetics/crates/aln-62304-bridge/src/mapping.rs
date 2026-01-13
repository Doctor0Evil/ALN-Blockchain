use aln_templates::AlnClauseId;
use bioscale_evidence::EvidenceTag;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardReference {
    pub standard: String,
    pub clause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStandardMapping {
    pub tag: EvidenceTag,
    pub aln_clause: AlnClauseId,
    pub standards: Vec<StandardReference>,
}
