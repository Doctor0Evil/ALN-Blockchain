use cosmwasm_std::{StdResult, DepsMut, Deps, Env, MessageInfo};
use cw_storage_plus::Map;
use serde::{Serialize, Deserialize};

static REFACTORS: Map<(&str, &str, &str, u64), bool> = Map::new("refactors");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RefactorRecord {
    pub origin_chain: String,
    pub tx_hash: String,
    pub nonce: u64,
    pub processed_at: u64,
}

pub fn record_refactor(deps: DepsMut, origin_chain: &str, token_addr: &str, tx_hash: &str, nonce: u64, processed_at: u64) -> StdResult<()> {
    REFACTORS.save(deps.storage, (origin_chain, token_addr, tx_hash, nonce), &true)?;
    Ok(())
}

pub fn is_processed(deps: Deps, origin_chain: &str, token_addr: &str, tx_hash: &str, nonce: u64) -> StdResult<bool> {
    Ok(REFACTORS.may_load(deps.storage, (origin_chain, token_addr, tx_hash, nonce))?.unwrap_or(false))
}
