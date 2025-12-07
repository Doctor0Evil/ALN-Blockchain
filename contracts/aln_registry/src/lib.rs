use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_storage_plus::{Map, Item};
use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = "aln-registry";
const CONTRACT_VERSION: &str = "0.1.0";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegisteredAsset {
    pub id: String,
    pub source_chain: String,
    pub source_denom: String,
    pub snapshot_height: u64,
    pub merkle_root: String,
    pub ubs_report_hash: Option<String>,
    pub scaling_profile_id: String,
    pub activation_height: u64,
    pub sanitized_approved: bool,
}

pub const ASSETS: Map<String, RegisteredAsset> = Map::new("reg_assets");
pub const GOVERNANCE: Item<Addr> = Item::new("governance_addr");
pub const ALLOW_MISSING_UBS: Item<bool> = Item::new("allow_missing_ubs");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub governance_addr: String,
    pub allow_missing_ubs: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterAsset { asset: RegisteredAsset },
    ApproveSanitized { id: String, ubs_report_hash: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAsset { id: String },
}

#[entry_point]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    let gov = deps.api.addr_validate(&msg.governance_addr)?;
    GOVERNANCE.save(deps.storage, &gov)?;
    let allow = msg.allow_missing_ubs.unwrap_or(false);
    ALLOW_MISSING_UBS.save(deps.storage, &allow)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterAsset { asset } => register_asset(deps, info, asset),
        ExecuteMsg::ApproveSanitized { id, ubs_report_hash } => approve_sanitized(deps, info, id, ubs_report_hash),
    }
}

fn register_asset(deps: DepsMut, info: MessageInfo, asset: RegisteredAsset) -> StdResult<Response> {
    let gov = GOVERNANCE.load(deps.storage)?;
    if info.sender != gov { return Err(cosmwasm_std::StdError::generic_err("only governance can register assets")); }
    let allow_missing = ALLOW_MISSING_UBS.load(deps.storage)?;
    if asset.ubs_report_hash.is_none() && !allow_missing {
        return Err(cosmwasm_std::StdError::generic_err("ubs_report_hash is required for asset registration"));
    }
    ASSETS.save(deps.storage, asset.id.clone(), &asset)?;
    Ok(Response::new().add_attribute("action", "register_asset"))
}

fn approve_sanitized(deps: DepsMut, info: MessageInfo, id: String, ubs_report_hash: String) -> StdResult<Response> {
    let gov = GOVERNANCE.load(deps.storage)?;
    if info.sender != gov { return Err(cosmwasm_std::StdError::generic_err("only governance can approve sanitized")); }
    let mut a = ASSETS.load(deps.storage, id.clone())?;
    a.ubs_report_hash = Some(ubs_report_hash);
    a.sanitized_approved = true;
    ASSETS.save(deps.storage, id.clone(), &a)?;
    Ok(Response::new().add_attribute("action", "approve_sanitized").add_attribute("id", id))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAsset { id } => {
            let a = ASSETS.load(deps.storage, id.clone())?;
            Ok(to_binary(&a)?)
        }
    }
}
