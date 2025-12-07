use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, CosmosMsg};
use cw_storage_plus::{Map, Item};
use serde::{Deserialize, Serialize};
use cw20::Cw20ExecuteMsg;
use sha2::{Sha256, Digest};
use hex;

use aln_registry::{QueryMsg as RegQueryMsg, RegisteredAsset};

const CONTRACT_NAME: &str = "aln-bridge-auet";
const CONTRACT_VERSION: &str = "0.2.0";

static CLAIMED: Map<(&Addr, &str, &str), bool> = Map::new("claimed");
pub const AUET_CONTRACT: Item<Addr> = Item::new("auet_contract");
pub const CSP_CONTRACT: Item<Addr> = Item::new("csp_contract");
pub const REGISTRY_CONTRACT: Item<Addr> = Item::new("registry_contract");
pub const GOVERNANCE: Item<Addr> = Item::new("governance_addr");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub auet_contract: String,
    pub csp_contract: Option<String>,
    pub registry_contract: String,
    pub governance_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SnapshotEntry {
    pub chain_id: String,
    pub height: u64,
    pub denom: String,
    pub address: String,
    pub balance: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProofStep {
    pub sibling: Binary,
    pub is_left: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Claim {
        asset_id: String,
        snapshot: SnapshotEntry,
        snapshot_hash: String,
        merkle_proof: Vec<ProofStep>,
        amount_auet: Uint128,
        amount_csp: Option<Uint128>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    IsClaimed { address: String, asset_id: String, snapshot_hash: String },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let au = deps.api.addr_validate(&msg.auet_contract)?;
    AUET_CONTRACT.save(deps.storage, &au)?;
    if let Some(csp) = msg.csp_contract {
        let csaddr = deps.api.addr_validate(&csp)?;
        CSP_CONTRACT.save(deps.storage, &csaddr)?;
    }
    let reg = deps.api.addr_validate(&msg.registry_contract)?;
    REGISTRY_CONTRACT.save(deps.storage, &reg)?;
    let gov = deps.api.addr_validate(&msg.governance_addr)?;
    GOVERNANCE.save(deps.storage, &gov)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Claim { asset_id, snapshot, snapshot_hash, merkle_proof, amount_auet, amount_csp } => {
            claim(deps, env, info, asset_id, snapshot, snapshot_hash, merkle_proof, amount_auet, amount_csp)
        }
    }
}

fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset_id: String,
    snapshot: SnapshotEntry,
    snapshot_hash: String,
    merkle_proof: Vec<ProofStep>,
    amount_auet: Uint128,
    amount_csp: Option<Uint128>,
) -> StdResult<Response> {
    let recipient = info.sender.clone();
    let key = (&recipient, asset_id.as_str(), snapshot_hash.as_str());
    if CLAIMED.may_load(deps.storage, key)?.unwrap_or(false) {
        return Err(cosmwasm_std::StdError::generic_err("already claimed"));
    }

    // recompute H_i
    let mut hasher = Sha256::new();
    hasher.update(snapshot.chain_id.as_bytes());
    hasher.update(&snapshot.height.to_be_bytes());
    hasher.update(snapshot.denom.as_bytes());
    hasher.update(snapshot.address.as_bytes());
    let b: u128 = snapshot.balance.parse().map_err(|_| cosmwasm_std::StdError::generic_err("invalid balance in snapshot"))?;
    hasher.update(&b.to_be_bytes());
    let digest = hasher.finalize();
    let computed_h = format!("0x{}", hex::encode(digest));
    if computed_h != snapshot_hash {
        return Err(cosmwasm_std::StdError::generic_err("snapshot hash mismatch"));
    }

    // fetch asset from registry
    let reg_addr = REGISTRY_CONTRACT.load(deps.storage)?;
    let asset: RegisteredAsset = deps.querier.query_wasm_smart(reg_addr, &RegQueryMsg::GetAsset { id: asset_id.clone() })?;

    // check sanitized_approved
    if !asset.sanitized_approved { return Err(cosmwasm_std::StdError::generic_err("asset not sanitized")); }

    // check activation_height
    if env.block.height < asset.activation_height.into() { return Err(cosmwasm_std::StdError::generic_err("asset claim not activated yet")); }

    // verify merkle proof using merkle root from asset
    let root = asset.merkle_root.clone();
    // convert computed_h (which is hex str) to bytes
    let mut leaf_bytes = [0u8; 32];
    let bytes = hex::decode(computed_h.trim_start_matches("0x")).map_err(|_| cosmwasm_std::StdError::generic_err("invalid snapshot_hash hex"))?;
    if bytes.len() != 32 { return Err(cosmwasm_std::StdError::generic_err("invalid snapshot_hash length")); }
    leaf_bytes.copy_from_slice(&bytes);

    // build proof vector (byte arrays)
    let mut proof_steps: Vec<( [u8;32], bool )> = vec![];
    for p in merkle_proof.iter() {
        let pbytes = p.sibling.clone().0;
        let mut arr = [0u8;32];
        if pbytes.len() != 32 { return Err(cosmwasm_std::StdError::generic_err("invalid proof sibling length")); }
        arr.copy_from_slice(&pbytes);
        proof_steps.push((arr, p.is_left));
    }

    if !verify_merkle_proof(&leaf_bytes, &proof_steps, root.trim_start_matches("0x")) {
        return Err(cosmwasm_std::StdError::generic_err("invalid merkle proof"));
    }

    // mark as claimed
    CLAIMED.save(deps.storage, key, &true)?;

    // Transfer AU.ET and CSP if present
    let auet_addr = AUET_CONTRACT.load(deps.storage)?;
    let transfer_auet = Cw20ExecuteMsg::Transfer { recipient: recipient.to_string(), amount: amount_auet };
    let wasm_msg: CosmosMsg = WasmMsg::Execute { contract_addr: auet_addr.to_string(), msg: to_binary(&transfer_auet)?, funds: vec![] }.into();

    let mut res = Response::new().add_message(wasm_msg).add_attribute("action", "claim").add_attribute("snapshot_hash", snapshot_hash);

    if let Some(csp_amt) = amount_csp {
        if CSP_CONTRACT.may_load(deps.storage)?.is_some() {
            let csp_addr = CSP_CONTRACT.load(deps.storage)?;
            let transfer_csp = Cw20ExecuteMsg::Transfer { recipient: recipient.to_string(), amount: csp_amt };
            let wasm_msg2: CosmosMsg = WasmMsg::Execute { contract_addr: csp_addr.to_string(), msg: to_binary(&transfer_csp)?, funds: vec![] }.into();
            res = res.add_message(wasm_msg2);
        }
    }

    Ok(res)
}

fn verify_merkle_proof(leaf: &[u8;32], proof: &Vec<([u8;32], bool)>, root_hex: &str) -> bool {
    let mut cur = *leaf;
    for (sib, is_left) in proof.iter() {
        let mut h = Sha256::new();
        if *is_left {
            h.update(sib);
            h.update(&cur);
        } else {
            h.update(&cur);
            h.update(sib);
        }
        let res = h.finalize();
        cur.copy_from_slice(&res);
    }
    let root_bytes = match hex::decode(root_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    if root_bytes.len() != 32 { return false; }
    let mut root_arr = [0u8;32];
    root_arr.copy_from_slice(&root_bytes);
    cur == root_arr
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsClaimed { address, asset_id, snapshot_hash } => {
            let addr = deps.api.addr_validate(&address)?;
            let key = (&addr, asset_id.as_str(), snapshot_hash.as_str());
            let val = CLAIMED.may_load(deps.storage, key)?.unwrap_or(false);
            Ok(to_binary(&val)?)
        }
    }
}
