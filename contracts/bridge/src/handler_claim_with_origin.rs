use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult, Uint128, Response};
use crate::core::bridge_architecture::OriginLockEvent;
use crate::{SnapshotEntry, ProofStep};
use crate::{record_refactor, refactor_is_processed};
use sha2::{Sha256, Digest};
use hex;

pub fn claim_with_origin(_deps: DepsMut, _env: Env, _info: MessageInfo, asset_id: String, origin_event: OriginLockEvent, merkle_proof: Vec<ProofStep>, ubs_report_hash: Option<String>, amount_auet: Uint128, amount_csp: Option<Uint128>) -> StdResult<Response> {
    // Convert OriginLockEvent -> SnapshotEntry-like record for H_i computation
    let snapshot = SnapshotEntry { chain_id: origin_event.origin_chain_id.clone(), height: origin_event.height.unwrap_or(0), denom: origin_event.denom.clone(), address: origin_event.origin_address.clone(), balance: origin_event.amount.clone() };
    // Reuse existing claim logic via calculation of snapshot_hash
    let mut hasher = Sha256::new();
    hasher.update(snapshot.chain_id.as_bytes());
    hasher.update(&snapshot.height.to_be_bytes());
    hasher.update(snapshot.denom.as_bytes());
    hasher.update(snapshot.address.as_bytes());
    let b: u128 = snapshot.balance.parse().map_err(|_| cosmwasm_std::StdError::generic_err("invalid balance in snapshot"))?;
    hasher.update(&b.to_be_bytes());
    let digest = hasher.finalize();
    let snapshot_hash = format!("0x{}", hex::encode(digest));
    // Call existing claim function on contract
    crate::claim(_deps, _env, _info, asset_id, snapshot, snapshot_hash, merkle_proof, amount_auet, amount_csp, Some(origin_event.tx_hash.clone()), Some(origin_event.nonce), ubs_report_hash)
}
