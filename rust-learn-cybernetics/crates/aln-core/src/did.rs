use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDid {
    pub did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BostromAddress {
    pub address: String,
}
