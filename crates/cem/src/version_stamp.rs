use sha2::{Digest, Sha256};

pub fn cem_version_stamp() -> String {
    let payload = b"CEM_CALIBRATION_v1.0|INDEXER_v1.0|KUJIRA_v1.0";
    let mut hasher = Sha256::new();
    hasher.update(payload);
    format!("{:x}", hasher.finalize())
}
