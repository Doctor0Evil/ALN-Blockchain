use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, CosmosMsg};
use cw_storage_plus::{Map, Item};
mod core;
pub use core::bridge_architecture::{OriginLockEvent, EnergyVector, UBS, SanitizationResult, SanitizationDecision, BridgeError};
mod handler_claim_with_origin;
pub use handler_claim_with_origin::claim_with_origin;
use serde::{Deserialize, Serialize};
pub use core::refactor_state::{record_refactor, is_processed as refactor_is_processed};
use cw20::Cw20ExecuteMsg;
use sha2::{Sha256, Digest};
use hex;
use aln_ubs::DefaultUBS;
use ubs_oracle::QueryMsg as OracleQueryMsg;
use serde_json::json;

use aln_registry::{QueryMsg as RegQueryMsg, RegisteredAsset};

const CONTRACT_NAME: &str = "aln-bridge-auet";
const CONTRACT_VERSION: &str = "0.2.0";

static CLAIMED: Map<(&Addr, &str, &str), bool> = Map::new("claimed");
pub const ENERGY_LEDGER: Map<&Addr, EnergyVector> = Map::new("energy_ledger");
pub const SYSTEM_WHITELIST: Map<&Addr, bool> = Map::new("system_whitelist");
pub const AUET_CONTRACT: Item<Addr> = Item::new("auet_contract");
pub const CSP_CONTRACT: Item<Addr> = Item::new("csp_contract");
pub const REGISTRY_CONTRACT: Item<Addr> = Item::new("registry_contract");
pub const UBS_ORACLE_CONTRACT: Item<Option<Addr>> = Item::new("ubs_oracle_contract");
pub const REFACTOR_AUDIT: Map<(&str, &str, u64), String> = Map::new("refactor_audit");
pub const GOVERNANCE: Item<Addr> = Item::new("governance_addr");
pub const TOXIC_SINK: Item<Option<Addr>> = Item::new("toxic_sink");
pub const ANOMALY_THRESHOLD_AMOUNT: Item<Option<Uint128>> = Item::new("anomaly_threshold_amount");
pub const TOTAL_ENERGY: Item<Uint128> = Item::new("total_energy");
pub const TOXIC_ENERGY: Item<Uint128> = Item::new("toxic_energy");
pub const TOXIC_CAP_PERCENT: Item<Option<u8>> = Item::new("toxic_cap_percent");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub auet_contract: String,
    pub csp_contract: Option<String>,
    pub registry_contract: String,
    pub governance_addr: String,
    pub toxic_sink: Option<String>,
    pub anomaly_threshold_amount: Option<Uint128>,
    pub toxic_cap_percent: Option<u8>,
    pub system_whitelist: Option<Vec<String>>,
    pub ubs_oracle_contract: Option<String>,
}

// SnapshotEntry is the legacy RPC-derived shape for snapshots. We will migrate to `OriginLockEvent`.
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
        // Optional origin tx metadata for replay protection and forensic mapping
        origin_tx_hash: Option<String>,
        origin_nonce: Option<u64>,
        ubs_report_hash: Option<String>,
    },
    ClaimWithOrigin { asset_id: String, origin_event: crate::core::bridge_architecture::OriginLockEvent, merkle_proof: Vec<ProofStep>, ubs_report_hash: Option<String>, amount_auet: Uint128, amount_csp: Option<Uint128> },
    /// System contract consumes a user's energy (debits ledger). ACL enforced.
    SystemConsume { owner: String, delta: EnergyVector },
    AddSystemWhitelist { addr: String },
    RemoveSystemWhitelist { addr: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    IsClaimed { address: String, asset_id: String, snapshot_hash: String },
    EnergyBalance { address: String },
    RefactorAudit { origin_chain: String, tx_hash: String, nonce: u64 },
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
    if let Some(s) = msg.toxic_sink {
        let saddr = deps.api.addr_validate(&s)?;
        TOXIC_SINK.save(deps.storage, &Some(saddr))?;
    } else {
        TOXIC_SINK.save(deps.storage, &None)?;
    }
    ANOMALY_THRESHOLD_AMOUNT.save(deps.storage, &msg.anomaly_threshold_amount)?;
    TOTAL_ENERGY.save(deps.storage, &Uint128::new(0u128))?;
    TOXIC_ENERGY.save(deps.storage, &Uint128::new(0u128))?;
    TOXIC_CAP_PERCENT.save(deps.storage, &msg.toxic_cap_percent)?;
    // Initialize system whitelist
    if let Some(list) = msg.system_whitelist {
        for addr in list {
            let a = deps.api.addr_validate(&addr)?;
            SYSTEM_WHITELIST.save(deps.storage, &a, &true)?;
        }
    }
    // set ubs oracle addr
    if let Some(ob) = msg.ubs_oracle_contract {
        let a = deps.api.addr_validate(&ob)?;
        UBS_ORACLE_CONTRACT.save(deps.storage, &Some(a))?;
    } else { UBS_ORACLE_CONTRACT.save(deps.storage, &None)?; }
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
        ExecuteMsg::Claim { asset_id, snapshot, snapshot_hash, merkle_proof, amount_auet, amount_csp, origin_tx_hash, origin_nonce, ubs_report_hash } => {
            claim(deps, env, info, asset_id, snapshot, snapshot_hash, merkle_proof, amount_auet, amount_csp, origin_tx_hash, origin_nonce, ubs_report_hash)
        }
        ExecuteMsg::ClaimWithOrigin { asset_id, origin_event, merkle_proof, ubs_report_hash, amount_auet, amount_csp } => {
            crate::handler_claim_with_origin::claim_with_origin(deps, env, info, asset_id, origin_event, merkle_proof, ubs_report_hash, amount_auet, amount_csp)
        }
        ExecuteMsg::SystemConsume { owner, delta } => {
            // Only whitelisted system contracts can call this action
            let caller = info.sender.clone();
            let owner_addr = deps.api.addr_validate(&owner)?;
            debit_energy(deps, &owner_addr, delta, &caller)?;
            Ok(Response::new().add_attribute("action", "system_consume").add_attribute("owner", owner))
        }
        ExecuteMsg::AddSystemWhitelist { addr } => {
            let caller = info.sender.clone();
            let gov = GOVERNANCE.load(deps.storage)?;
            if caller != gov { return Err(cosmwasm_std::StdError::generic_err("only governance can set system whitelist")); }
            let a = deps.api.addr_validate(&addr)?;
            SYSTEM_WHITELIST.save(deps.storage, &a, &true)?;
            Ok(Response::new().add_attribute("action", "add_system_whitelist").add_attribute("addr", addr))
        }
        ExecuteMsg::RemoveSystemWhitelist { addr } => {
            let caller = info.sender.clone();
            let gov = GOVERNANCE.load(deps.storage)?;
            if caller != gov { return Err(cosmwasm_std::StdError::generic_err("only governance can set system whitelist")); }
            let a = deps.api.addr_validate(&addr)?;
            SYSTEM_WHITELIST.save(deps.storage, &a, &false)?;
            Ok(Response::new().add_attribute("action", "remove_system_whitelist").add_attribute("addr", addr))
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
    origin_tx_hash: Option<String>,
    origin_nonce: Option<u64>,
    ubs_report_hash: Option<String>,
) -> StdResult<Response> {
    let recipient = info.sender.clone();
    let key = (&recipient, asset_id.as_str(), snapshot_hash.as_str());
    if CLAIMED.may_load(deps.storage, key)?.unwrap_or(false) {
        return Err(cosmwasm_std::StdError::generic_err("already claimed"));
    }

    // If origin tx metadata provided, check refactor registry to avoid replays across chains
    if origin_tx_hash.is_some() && origin_nonce.is_some() {
        let txh = origin_tx_hash.as_ref().unwrap();
        let n = origin_nonce.unwrap();
        if refactor_is_processed(deps.as_ref(), snapshot.chain_id.as_str(), snapshot.denom.as_str(), txh.as_str(), n)? {
            return Err(cosmwasm_std::StdError::generic_err("origin event already processed"));
        }
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

    // check sanitized_approved and presence of a ubs_report_hash
    if !asset.sanitized_approved { return Err(cosmwasm_std::StdError::generic_err("asset not sanitized")); }
    let reg_ubs_hash = asset.ubs_report_hash.clone();
    if reg_ubs_hash.is_none() { return Err(cosmwasm_std::StdError::generic_err("ubs report hash missing on registered asset")); }
    // If claim included an explicit ubs hash, verify it matches the registry
    if let Some(claim_hash) = ubs_report_hash.clone() {
        if Some(claim_hash) != reg_ubs_hash.clone() {
            return Err(cosmwasm_std::StdError::generic_err("ubs report hash mismatch"));
        }
    }

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

    // record refactor into append-only refactor registry if origin metadata provided
    if origin_tx_hash.is_some() && origin_nonce.is_some() {
        // processed_at using block time (seconds)
        let ts = env.block.time.seconds();
        record_refactor(deps, snapshot.chain_id.as_str(), snapshot.denom.as_str(), origin_tx_hash.as_ref().unwrap(), origin_nonce.unwrap(), ts)?;
    }

    // Check toxic cap and update totals
    let mut total_energy = TOTAL_ENERGY.load(deps.storage)?;
    let mut toxic_energy = TOXIC_ENERGY.load(deps.storage)?;
    let toxic_cap = TOXIC_CAP_PERCENT.may_load(deps.storage)?;
    let scaling_is_malicious = asset.scaling_profile_id.contains("malicious");
    if scaling_is_malicious {
        // proposed toxic addition (auet + csp if present)
        let add = amount_auet.u128() + amount_csp.map(|c| c.u128()).unwrap_or(0);
        let new_to = toxic_energy.u128() + add;
        let new_total = total_energy.u128() + add;
        if let Some(pct) = toxic_cap {
            let pct_val = pct as u128;
            // if new_total == 0, allow (initial); else check <= pct
            if new_total > 0 {
                let cur_pct = (new_to * 100u128) / new_total;
                if cur_pct > pct_val { return Err(cosmwasm_std::StdError::generic_err("toxic cap exceeded")); }
            }
        }
        // update counters
        TOXIC_ENERGY.save(deps.storage, &Uint128::new(new_to))?;
        TOTAL_ENERGY.save(deps.storage, &Uint128::new(new_total))?;
    } else {
        let add = amount_auet.u128() + amount_csp.map(|c| c.u128()).unwrap_or(0);
        let new_total = total_energy.u128() + add;
        TOTAL_ENERGY.save(deps.storage, &Uint128::new(new_total))?;
    }

    // Transfer AU.ET and CSP if present
    let auet_addr = AUET_CONTRACT.load(deps.storage)?;
    // If the asset is marked malicious, route to the toxic sink if configured
    let sink = TOXIC_SINK.may_load(deps.storage)?;
    let target = if asset.scaling_profile_id.contains("malicious") {
        // if sink configured, route there, else fail to protect users
        if let Some(sink_addr) = sink { sink_addr.to_string() } else { return Err(cosmwasm_std::StdError::generic_err("toxic asset requires sink")); }
    } else {
        recipient.to_string()
    };
    // anomaly detection for large amounts
    if let Some(th) = ANOMALY_THRESHOLD_AMOUNT.may_load(deps.storage)? {
        if amount_auet > th {
            // mark anomaly and route to sink if configured, or fail
            let sink = TOXIC_SINK.may_load(deps.storage)?;
            if let Some(sink_addr) = sink { let transfer_auet = Cw20ExecuteMsg::Transfer { recipient: sink_addr.to_string(), amount: amount_auet }; let wasm_msg: CosmosMsg = WasmMsg::Execute { contract_addr: auet_addr.to_string(), msg: to_binary(&transfer_auet)?, funds: vec![] }.into(); let mut res = Response::new().add_message(wasm_msg).add_attribute("action", "claim_anomaly").add_attribute("snapshot_hash", snapshot_hash); return Ok(res); } else { return Err(cosmwasm_std::StdError::generic_err("anomaly threshold exceeded and no sink configured")); }
        }
    }
    // ----- UBS sanitization + Sealed refactor ledger credit path -----
    // Call UBS to sanitize origin token and compute energy vector
            // Query UBS oracle for a finalized sanitized result if oracle configured
            let ubs_oracle_addr_opt = UBS_ORACLE_CONTRACT.may_load(deps.storage)?;
            let mut sres: SanitizationResult;
            if let Some(ubs_addr) = ubs_oracle_addr_opt {
                // Build replay key using origin chain, tx_hash and nonce in a stable way
                let replay_key_raw = (snapshot.chain_id.clone() + ":" + origin_tx_hash.as_ref().unwrap_or(&"".to_string()) + ":" + &origin_nonce.unwrap_or(0).to_string());
                let replay_bin = Binary::from(replay_key_raw.clone().into_bytes());
                // Query the on-chain UBS oracle (aggregated report) and map to a SanitizationResult
                let qres: Option<ubs_oracle::AggregatedReport> = deps.querier.query_wasm_smart(ubs_addr.clone(), &OracleQueryMsg::GetReport { replay_key: replay_bin })?;
                if let Some(agg) = qres {
                    // Map aggregated report to a sanitization result
                    let decision = match agg.ubs_class {
                        0 => aln_ubs::SanitizationDecision::Approved,
                        1 => aln_ubs::SanitizationDecision::Downgraded,
                        _ => aln_ubs::SanitizationDecision::Rejected,
                    };
                    let risk_score = (agg.threat_bps as f64) / 10000.0;
                    let amount_total = amount_auet.u128() + amount_csp.map(|c| c.u128()).unwrap_or(0);
                    let energy_vec = aln_ubs::energy_mapping::map_to_energy(amount_total, &risk_score, &vec![]);
                    sres = aln_ubs::SanitizationResult { decision: decision, energy: energy_vec, report_hash: format!("oracle_agg:{}:{}", agg.ubs_class, agg.threat_bps) };
                } else {
                    return Err(cosmwasm_std::StdError::generic_err("ubs oracle report not available"));
                }
            } else {
                // fallback to local DefaultUBS (not recommended in prod)
                let ubs = DefaultUBS {};
                sres = ubs.sanitize(snapshot.chain_id.as_str(), snapshot.denom.as_str(), &[]).map_err(|e| cosmwasm_std::StdError::generic_err(format!("ubs sanitize failed: {:?}", e)))?;
            }
            // Map aln_ubs energy vector to contract EnergyVector
            let ev = EnergyVector { auet: sres.energy.auet, csp: sres.energy.csp, erp: sres.energy.erp };
            // store audit (report hash) if origin metadata present
            if origin_tx_hash.is_some() && origin_nonce.is_some() {
                let txh = origin_tx_hash.as_ref().unwrap();
                REFACTOR_AUDIT.save(deps.storage, (snapshot.chain_id.as_str(), txh.as_str(), origin_nonce.unwrap()), &sres.report_hash)?;
            }
            // If rejected, record the attempt and do not mint
            if sres.decision == aln_ubs::SanitizationDecision::Rejected {
                if origin_tx_hash.is_some() && origin_nonce.is_some() {
                    record_refactor(deps, snapshot.chain_id.as_str(), snapshot.denom.as_str(), origin_tx_hash.as_ref().unwrap(), origin_nonce.unwrap(), env.block.time.seconds())?;
                }
                let json = serde_json::json!({"action":"claim_rejected","origin_chain":snapshot.chain_id.as_str(),"tx":origin_tx_hash.as_ref().unwrap_or(&"".to_string()),"report_hash": sres.report_hash});
                return Ok(Response::new().add_attribute("action","claim_rejected").add_attribute("refactor_audit", json.to_string()));
            }
            // Credit ledger with energy vector
            credit_energy(deps, &recipient, ev.clone()).map_err(|e| cosmwasm_std::StdError::generic_err(format!("ledger credit failed: {:?}", e)))?;
            // save refactor record and update totals
            if origin_tx_hash.is_some() && origin_nonce.is_some() { record_refactor(deps, snapshot.chain_id.as_str(), snapshot.denom.as_str(), origin_tx_hash.as_ref().unwrap(), origin_nonce.unwrap(), env.block.time.seconds())?; }
            TOTAL_ENERGY.save(deps.storage, &Uint128::new(TOTAL_ENERGY.load(deps.storage)?.u128() + ev.auet.u128() + ev.csp.u128()))?;
            if asset.scaling_profile_id.contains("malicious") { TOXIC_ENERGY.save(deps.storage, &Uint128::new(TOXIC_ENERGY.load(deps.storage)?.u128() + ev.auet.u128() + ev.csp.u128()))?; }
            let json = serde_json::json!({"action":"claim_refactored","origin_chain":snapshot.chain_id.as_str(),"tx":origin_tx_hash.as_ref().unwrap_or(&"".to_string()),"report_hash": sres.report_hash});
            let mut res = Response::new().add_attribute("action", "claim").add_attribute("snapshot_hash", snapshot_hash).add_attribute("claim_refactored", "true").add_attribute("refactor_audit", json.to_string());
            return Ok(res);
        }
    }

    // No immediate cw20 transfers to user - balances are recorded in the ledger

    // Response already returned in the claim refactor path
    Ok(Response::new())
}

/// Credit energy ledger for user
fn credit_energy(deps: DepsMut, owner: &Addr, delta: EnergyVector) -> StdResult<()> {
    let existing = ENERGY_LEDGER.may_load(deps.storage, owner)?.unwrap_or(EnergyVector { auet: Uint128::zero(), csp: Uint128::zero(), erp: Uint128::zero() });
    let newv = EnergyVector { auet: Uint128::new(existing.auet.u128() + delta.auet.u128()), csp: Uint128::new(existing.csp.u128() + delta.csp.u128()), erp: Uint128::new(existing.erp.u128() + delta.erp.u128()) };
    ENERGY_LEDGER.save(deps.storage, owner, &newv)?;
    Ok(())
}

/// Debit energy ledger for owner if caller is system-allowed
fn debit_energy(deps: DepsMut, owner: &Addr, delta: EnergyVector, caller: &Addr) -> StdResult<()> {
    // Check system whitelist
    if !SYSTEM_WHITELIST.may_load(deps.storage, caller)?.unwrap_or(false) {
        return Err(cosmwasm_std::StdError::generic_err("caller not system-allowed"));
    }
    let existing = ENERGY_LEDGER.may_load(deps.storage, owner)?.unwrap_or(EnergyVector { auet: Uint128::zero(), csp: Uint128::zero(), erp: Uint128::zero() });
    if existing.auet.u128() < delta.auet.u128() || existing.csp.u128() < delta.csp.u128() || existing.erp.u128() < delta.erp.u128() {
        return Err(cosmwasm_std::StdError::generic_err("insufficient energy"));
    }
    let newv = EnergyVector { auet: Uint128::new(existing.auet.u128() - delta.auet.u128()), csp: Uint128::new(existing.csp.u128() - delta.csp.u128()), erp: Uint128::new(existing.erp.u128() - delta.erp.u128()) };
    ENERGY_LEDGER.save(deps.storage, owner, &newv)?;
    Ok(())
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
        QueryMsg::EnergyBalance { address } => {
            let addr = deps.api.addr_validate(&address)?;
            let v = ENERGY_LEDGER.may_load(deps.storage, &addr)?.unwrap_or(EnergyVector { auet: Uint128::zero(), csp: Uint128::zero(), erp: Uint128::zero() });
            Ok(to_binary(&v)?)
        }
        QueryMsg::RefactorAudit { origin_chain, tx_hash, nonce } => {
            let val = REFACTOR_AUDIT.may_load(deps.storage, (origin_chain.as_str(), tx_hash.as_str(), nonce))?;
            Ok(to_binary(&val)?)
        }
    }
}
