use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crate::energy_mapping::EnergyVector;
use super::SanitizationDecision;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UBSReport {
    pub denom: String,
    pub decision: SanitizationDecision,
    pub energy: EnergyVector,
    pub severity: String,
}

pub fn build_report(origin_chain: &str, token_addr: &str, _static: &crate::static_analysis::StaticReport, _econ: &crate::econ_metadata::EconMetadata, _dyn: &crate::dynamic_behavior::DynamicBehavior, e: &EnergyVector) -> UBSReportResult {
    let report = UBSReport {
        denom: format!("{}::{}", origin_chain, token_addr),
        decision: SanitizationDecision::Approved,
        energy: e.clone(),
        severity: "low".to_string(),
    };
    let json = serde_json::to_string_pretty(&report).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let digest = hasher.finalize();
    let hex = format!("0x{}", hex::encode(digest));
    UBSReportResult { report, json, hash_hex: hex }
}

pub struct UBSReportResult {
    pub report: UBSReport,
    pub json: String,
    pub hash_hex: String,
}

pub fn write_report_to_disk(dir: &str, denom: &str, res: &UBSReportResult) -> std::io::Result<String> {
    std::fs::create_dir_all(dir)?;
    let filename = format!("{}/ubs_report_{}.json", dir.trim_end_matches('/'), denom.replace('/', "_"));
    std::fs::write(&filename, &res.json)?;
    Ok(filename)
}
