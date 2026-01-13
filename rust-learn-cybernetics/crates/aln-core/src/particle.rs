use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ALNComplianceParticle {
    pub id: Uuid,
    pub clause_id: String,
    pub evidence_envelope_id: Uuid,
}
