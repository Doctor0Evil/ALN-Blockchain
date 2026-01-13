use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostBudget {
    pub max_energy_joules: f32,
    pub max_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSpecs {
    pub subject_id: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDescriptor {
    pub id: Uuid,
    pub name: String,
    pub required_energy_joules: f32,
    pub worst_case_latency_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpgradeState {
    Proposed,
    Approved,
    Rejected,
}
