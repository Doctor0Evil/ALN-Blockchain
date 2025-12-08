use serde::{Serialize, Deserialize};
use cosmwasm_std::Uint128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyVector {
    pub auet: Uint128,
    pub csp: Uint128,
    pub erp: Uint128,
}

pub fn map_to_energy(amount: u128, risk_score: &f64, _categories: &Vec<String>) -> EnergyVector {
    // Deterministic conservative mapping: energy = amount * (1 - risk_score)
    let factor = 1.0 - risk_score;
    let au = ((amount as f64) * factor) as u128;
    EnergyVector { auet: Uint128::new(au), csp: Uint128::new((au/2).max(0)), erp: Uint128::new(0) }
}
