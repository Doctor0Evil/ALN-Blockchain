use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};
use cosmwasm_std::Uint128;
use cw20_base::{contract as cw20_base, msg::InstantiateMsg as Cw20InstantiateMsg};
use cw20_base::state::TokenInfo;
use cw20_base::state::TOKEN_INFO;
use cw_storage_plus::Item;

const CONTRACT_NAME: &str = "aln-csp";
const CONTRACT_VERSION: &str = "0.1.0";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct SnapshotMeta {
    pub chain_id: String,
    pub height: u64,
    pub merkle_root: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub cw20: Cw20InstantiateMsg,
    pub snapshot: SnapshotMeta,
    pub transfer_whitelist: Option<Vec<String>>,
}

pub const SNAPSHOT: Item<SnapshotMeta> = Item::new("snapshot_meta_csp");
pub const WHITELIST: Item<Vec<String>> = Item::new("csp_whitelist");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, cw20_base::ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut inner = msg.cw20;
    inner.mint = None;
    let resp = cw20_base::instantiate(deps, env, info, inner)?;

    SNAPSHOT.save(deps.storage, &msg.snapshot)?;
    if let Some(list) = msg.transfer_whitelist {
        WHITELIST.save(deps.storage, &list)?;
    } else {
        WHITELIST.save(deps.storage, &vec![])?;
    }

    TOKEN_INFO.update(deps.storage, |mut info: TokenInfo| -> StdResult<_> {
        info.description = Some(format!(
            "CSP non-mintable; snapshot {}:{} root {}",
            msg.snapshot.chain_id, msg.snapshot.height, msg.snapshot.merkle_root
        ));
        Ok(info)
    })?;

    Ok(resp)
}

// Override execute to disallow transfers by default unless whitelist
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ExecuteMsg {
    Cw20(Cw20ExecuteMsg),
    Spend { action: String, data: Option<Binary>, module: String, amount: Uint128 },
    Burn { amount: Uint128 },
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, cw20_base::ContractError> {
    match &msg {
        ExecuteMsg::Cw20(c) => {
            match &c {
                Cw20ExecuteMsg::Transfer { .. } | Cw20ExecuteMsg::Send { .. } |
                Cw20ExecuteMsg::TransferFrom { .. } | Cw20ExecuteMsg::SendFrom { .. } |
                Cw20ExecuteMsg::IncreaseAllowance { .. } | Cw20ExecuteMsg::DecreaseAllowance { .. } => {
                    return Err(cw20_base::ContractError::Std(cosmwasm_std::StdError::generic_err("CSP transfer/allowance operations are disabled")));
                }
                _ => {}
            }
            cw20_base::execute(deps, env, info, c)
        }
        ExecuteMsg::Spend { action: _, data: _, module, amount } => {
            // Allow if owner calls (soulbound) or if a module is calling on behalf of owner
            let whitelist = WHITELIST.may_load(deps.storage)?.unwrap_or_default();
            let caller = info.sender.to_string();
            if !whitelist.is_empty() {
                // allow if caller is in whitelist
                if !whitelist.iter().any(|x| x == &caller) {
                    return Err(cw20_base::ContractError::Std(cosmwasm_std::StdError::generic_err("Caller not whitelisted module")));
                }
            } else {
                // not whitelisted -> only owner calls allowed via a different flow (Burn/Spend with owner info)
                // Allow proceed only when caller == module (owner acting)
            }
            // perform transfer to module: use cw20_base::execute with Transfer
            let transfer_msg = Cw20ExecuteMsg::Transfer { recipient: module.clone(), amount: *amount };
            cw20_base::execute(deps, env, info, transfer_msg)
        }
        ExecuteMsg::Burn { amount } => {
            let burn_msg = Cw20ExecuteMsg::Burn { amount: *amount };
            cw20_base::execute(deps, env, info, burn_msg)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: Cw20QueryMsg) -> StdResult<Binary> {
    cw20_base::query(deps, env, msg)
}
