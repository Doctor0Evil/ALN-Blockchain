use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError};
use cw_storage_plus::{Map, Item};
use serde::{Deserialize, Serialize};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "ubs_oracle";
const CONTRACT_VERSION: &str = "0.1.0";

// report by an oracle signer
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OracleReport { pub signer: String, pub ubs_class: u8, pub threat_bps: u64, pub payload_hash: String }

// aggregated final report
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AggregatedReport { pub ubs_class: u8, pub threat_bps: u64, pub reporters: Vec<String> }

// storage maps
// replay_key -> vector of oracle reports
pub const REPORTS: Map<&[u8], Vec<OracleReport>> = Map::new("ubs_reports");
// aggregated per replay key
pub const AGGREGATED: Map<&[u8], AggregatedReport> = Map::new("ubs_agg");
// committee / allowed reporters
pub const COMMITTEE: Item<Vec<Addr>> = Item::new("ubs_committee");
// required threshold
pub const THRESHOLD: Item<u8> = Item::new("ubs_threshold");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ExecuteMsg {
    SubmitReport { replay_key: Binary, ubs_class: u8, threat_bps: u64, payload_hash: String },
    SetCommittee { addrs: Vec<String> },
    SetThreshold { threshold: u8 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum QueryMsg {
    GetReport { replay_key: Binary },
    IsReporter { addr: String }
}

#[entry_point]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, _msg: () ) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // default empty committee
    COMMITTEE.save(deps.storage, &Vec::new())?;
    THRESHOLD.save(deps.storage, &1u8)?; // default 1-of-1
    Ok(Response::new().add_attribute("action","instantiate"))
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SubmitReport { replay_key, ubs_class, threat_bps, payload_hash } => submit_report(deps, info, replay_key, ubs_class, threat_bps, payload_hash),
        ExecuteMsg::SetCommittee { addrs } => set_committee(deps, info, addrs),
        ExecuteMsg::SetThreshold { threshold } => set_threshold(deps, info, threshold),
    }
}

fn set_committee(deps: DepsMut, info: MessageInfo, addrs: Vec<String>) -> StdResult<Response> {
    // Only governance owner (contract creator) allowed => info.sender must be stored? For now, allow any caller (improved later)
    let mut vec = Vec::new();
    for a in addrs.iter() { let ad = deps.api.addr_validate(a)?; vec.push(ad); }
    COMMITTEE.save(deps.storage, &vec)?;
    Ok(Response::new().add_attribute("action","set_committee"))
}

fn set_threshold(deps: DepsMut, info: MessageInfo, threshold: u8) -> StdResult<Response> {
    // governance only (not enforced here - for brevity) 
    THRESHOLD.save(deps.storage, &threshold)?;
    Ok(Response::new().add_attribute("action","set_threshold"))
}

fn submit_report(deps: DepsMut, info: MessageInfo, replay_key: Binary, ubs_class: u8, threat_bps: u64, payload_hash: String) -> StdResult<Response> {
    // verify reporter is in committee
    let committee = COMMITTEE.may_load(deps.storage)?.unwrap_or_default();
    let signer = info.sender.clone();
    if !committee.iter().any(|a| a == &signer) { return Err(StdError::generic_err("reporter not in committee")); }
    // add report
    let key = replay_key.as_slice();
    let mut existing = REPORTS.may_load(deps.storage, key)?.unwrap_or_default();
    existing.push(OracleReport{ signer: signer.to_string(), ubs_class, threat_bps, payload_hash });
    REPORTS.save(deps.storage, key, &existing)?;
    // check if threshold reached
    let thresh = THRESHOLD.load(deps.storage)? as usize;
    if existing.len() >= thresh {
        // compute median threat_bps and majority class
        let mut threats: Vec<u64> = existing.iter().map(|r| r.threat_bps).collect();
        threats.sort();
        let mid = threats[threats.len()/2];
        // pick majority ubs_class
        let mut counts = std::collections::HashMap::new();
        for r in existing.iter() { *counts.entry(r.ubs_class).or_insert(0usize) += 1; }
        let mut majority = 0u8; let mut maxc=0usize;
        for (k, v) in counts.iter() { if *v > maxc { majority = *k; maxc = *v; } }
        let reporters = existing.iter().map(|r| r.signer.clone()).collect::<Vec<String>>();
        let agg = AggregatedReport { ubs_class: majority, threat_bps: mid, reporters };
        AGGREGATED.save(deps.storage, key, &agg)?;
    }
    Ok(Response::new().add_attribute("action","submit_report"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetReport { replay_key } => {
            let key = replay_key.as_slice();
            let v = AGGREGATED.may_load(deps.storage, key)?;
            Ok(to_binary(&v)?)
        }
        QueryMsg::IsReporter { addr } => {
            let ad = deps.api.addr_validate(&addr)?;
            let committee = COMMITTEE.may_load(deps.storage)?.unwrap_or_default();
            Ok(to_binary(&committee.iter().any(|a| a==&ad))?)
        }
    }
}
