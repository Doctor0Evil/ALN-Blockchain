use serde::{Serialize, Deserialize};
use cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnergyBalance { pub class_id: String, pub balance: Uint128 }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HudSchema {
    pub balances: Vec<EnergyBalance>,
    pub active_commitments: Vec<String>,
    pub risk_indicators: Vec<(String, String)>,
    pub pending_conversions: Vec<(String, Uint128)>,
    pub anomalies: Vec<String>,
}
