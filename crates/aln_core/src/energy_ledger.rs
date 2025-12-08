use serde::{Deserialize, Serialize};
use cosmwasm_std::Uint128;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EnergyVector {
    pub auet: Uint128,
    pub csp: Uint128,
    pub erp: Uint128,
}

#[derive(Debug, thiserror::Error)]
pub enum EnergyError {
    #[error("insufficient-energy")]
    Underflow,
    #[error("forbidden: caller not system contract")]
    Forbidden,
}

pub trait EnergyLedger {
    fn credit(&mut self, owner: &[u8], delta: EnergyVector) -> Result<()>;
    fn debit(&mut self, owner: &[u8], delta: EnergyVector, caller_contract_id: u32) -> Result<(), EnergyError>;
    fn balance_of(&self, owner: &[u8]) -> EnergyVector;
}

// Simple in-memory ledger used for unit tests and local prototypes
pub struct InMemoryEnergyLedger {
    pub state: std::collections::HashMap<Vec<u8>, EnergyVector>,
    pub system_mask: u32,
}

impl InMemoryEnergyLedger {
    pub fn new(system_mask: u32) -> Self {
        Self { state: std::collections::HashMap::new(), system_mask }
    }
}

impl EnergyLedger for InMemoryEnergyLedger {
    fn credit(&mut self, owner: &[u8], delta: EnergyVector) -> Result<()> {
        let key = owner.to_vec();
        let entry = self.state.entry(key).or_insert_with(Default::default);
        entry.auet = Uint128::new(entry.auet.u128() + delta.auet.u128());
        entry.csp = Uint128::new(entry.csp.u128() + delta.csp.u128());
        entry.erp = Uint128::new(entry.erp.u128() + delta.erp.u128());
        Ok(())
    }
    fn debit(&mut self, owner: &[u8], delta: EnergyVector, caller_contract_id: u32) -> Result<(), EnergyError> {
        // permission check via system mask
        if (caller_contract_id & self.system_mask) == 0 {
            return Err(EnergyError::Forbidden);
        }
        let key = owner.to_vec();
        let entry = self.state.entry(key).or_insert_with(Default::default);
        // underflow checks
        if entry.auet.u128() < delta.auet.u128() || entry.csp.u128() < delta.csp.u128() || entry.erp.u128() < delta.erp.u128() {
            return Err(EnergyError::Underflow);
        }
        entry.auet = Uint128::new(entry.auet.u128() - delta.auet.u128());
        entry.csp = Uint128::new(entry.csp.u128() - delta.csp.u128());
        entry.erp = Uint128::new(entry.erp.u128() - delta.erp.u128());
        Ok(())
    }
    fn balance_of(&self, owner: &[u8]) -> EnergyVector {
        self.state.get(owner).cloned().unwrap_or_default()
    }
}
