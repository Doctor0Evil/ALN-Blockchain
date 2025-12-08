pub mod identity_origin;
pub mod static_analysis;
pub mod econ_metadata;
pub mod dynamic_behavior;
pub mod energy_mapping;
pub mod report;

use serde::{Serialize, Deserialize};
use crate::energy_mapping::EnergyVector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanitizationDecision { Approved, Downgraded, Rejected }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationResult {
    pub decision: SanitizationDecision,
    pub energy: EnergyVector,
    pub report_hash: String,
}

pub trait UBS {
    fn sanitize(&self, origin_chain_id: &str, token_addr: &str, contract_wasm: &[u8]) -> anyhow::Result<SanitizationResult>;
}

pub struct DefaultUBS;
impl UBS for DefaultUBS {
    fn sanitize(&self, origin_chain_id: &str, token_addr: &str, contract_wasm: &[u8]) -> anyhow::Result<SanitizationResult> {
        // Compose deterministic pipeline
        let id_ok = identity_origin::check_identity(origin_chain_id, token_addr);
        let static_report = static_analysis::analyze_contract(contract_wasm);
        let econ = econ_metadata::analyze_denom(token_addr);
        let dyn_beh = dynamic_behavior::assess_dynamic(contract_wasm);
        let energy = energy_mapping::map_to_energy(100u128, &econ.risk_score, &static_report.categories);
        let report = report::build_report(origin_chain_id, token_addr, &static_report, &econ, &dyn_beh, &energy);
        Ok(SanitizationResult { decision: report.decision.clone(), energy: energy.clone(), report_hash: report.hash_hex })
    }
}
