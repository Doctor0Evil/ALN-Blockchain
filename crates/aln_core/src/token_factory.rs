use serde::{Serialize, Deserialize};
use anyhow::Result;
use sha2::{Digest, Sha256};
use crate::identity::map_did_to_role;
use crate::identity::RoleProfile;
use crate::energy_ledger::{EnergyLedger, EnergyVector};
use hex;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TokenClassType { Aln20Energy, Other }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenClassParams {
    pub name: String,
    pub symbol: String,
    pub class_type: TokenClassType,
    pub template_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenClassRecord {
    pub class_id: String,
    pub params: TokenClassParams,
    pub creator: String,
    pub is_transferable: bool,
}

pub struct TokenFactory;

#[derive(thiserror::Error, Debug)]
pub enum FactoryError { #[error("fee_required")] FeeRequired }

impl TokenFactory {
    pub fn create_token_class(params: &TokenClassParams, creator: &str, aln_fee_paid: u128) -> Result<TokenClassRecord, FactoryError> {
        // enforce fee
        let required_fee = match params.class_type {
            TokenClassType::Aln20Energy => 10u128,
            _ => 1u128,
        };
        if aln_fee_paid < required_fee { return Err(FactoryError::FeeRequired); }
        // check role profile of creator: must be `Operator` or `Builder` to create classes
        let role = map_did_to_role(creator);
        match role {
            RoleProfile::Operator | RoleProfile::Builder => {}
            _ => return Err(FactoryError::FeeRequired), // reuse error for unauthorized (simplification)
        }
        // UBS check, template validation would happen off-chain; assume passed
        let mut hasher = Sha256::new();
        hasher.update(params.name.as_bytes());
        hasher.update(params.symbol.as_bytes());
        if let Some(t) = &params.template_id { hasher.update(t.as_bytes()); }
        let digest = hasher.finalize();
        let id = format!("tc_0x{}", hex::encode(digest));
        // persist or return record
        let is_transferable = match params.class_type { TokenClassType::Aln20Energy => false, _ => true };
        Ok(TokenClassRecord { class_id: id, params: params.clone(), creator: creator.to_string(), is_transferable })
    }
    // Mint as energy into the ledger instead of minting a tradable token
    pub fn mint_to_ledger<L: EnergyLedger>(ledger: &mut L, owner: &[u8], amount: u128) -> anyhow::Result<()> {
        let ev = EnergyVector { auet: cosmwasm_std::Uint128::new(amount), csp: cosmwasm_std::Uint128::new(amount/2), erp: cosmwasm_std::Uint128::new(0) };
        ledger.credit(owner, ev)?;
        Ok(())
    }
}
