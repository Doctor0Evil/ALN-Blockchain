use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse};
use cw20_base::state::{TokenInfo, TOKEN_INFO};
use cw20_base::{contract as cw20_base, msg::InstantiateMsg as Cw20InstantiateMsg};
use cw_storage_plus::Item;

const CONTRACT_NAME: &str = "aln-auet-nonmint";
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
    pub allowed_modules: Option<Vec<String>>,
}

pub const SNAPSHOT: Item<SnapshotMeta> = Item::new("snapshot_meta");
use cw_storage_plus::Map;
pub const ALLOWED_MODULE: Map<&Addr, bool> = Map::new("allowed_module");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, cw20_base::ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Force non-mintable CW20: override any minter in inner msg
    let mut inner = msg.cw20;
    inner.mint = None;

    // Use cw20-base instantiate for balances/metadata
    let resp = cw20_base::instantiate(deps, env, info, inner)?;

    // Store snapshot metadata in storage for queries
    SNAPSHOT.save(deps.storage, &msg.snapshot)?;
    if let Some(list) = msg.allowed_modules {
        for addr in list {
            let a = deps.api.addr_validate(&addr)?;
            ALLOWED_MODULE.save(deps.storage, &a, &true)?;
        }
    }

    // Optionally put a human readable description in TOKEN_INFO
    TOKEN_INFO.update(deps.storage, |mut info: TokenInfo| -> StdResult<_> {
        info.description = Some(format!(
            "AU.ET non-mintable; snapshot {}:{} root {}",
            msg.snapshot.chain_id, msg.snapshot.height, msg.snapshot.merkle_root
        ));
        Ok(info)
    })?;

    Ok(resp)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ExecuteMsg {
    Cw20(Cw20ExecuteMsg),
    Spend { action: String, data: Option<Binary>, recipient: String, amount: Uint128 },
    Burn { amount: Uint128 },
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, cw20_base::ContractError> {
    match msg {
        ExecuteMsg::Cw20(c) => {
            // Intercept Transfer/Send/Allowance/TransferFrom & block if not to allowed module
            match &c {
                Cw20ExecuteMsg::Transfer { recipient, amount: _ } => {
                    let addr = deps.api.addr_validate(recipient)?;
                    if !ALLOWED_MODULE.may_load(deps.storage, &addr)?.unwrap_or(false) {
                        return Err(cw20_base::ContractError::Std(cosmwasm_std::StdError::generic_err("Transfers to non-protocol modules are disabled")));
                    }
                }
                Cw20ExecuteMsg::Send { contract, amount: _ } => {
                    let addr = deps.api.addr_validate(contract)?;
                    if !ALLOWED_MODULE.may_load(deps.storage, &addr)?.unwrap_or(false) {
                        return Err(cw20_base::ContractError::Std(cosmwasm_std::StdError::generic_err("Sends to non-protocol modules are disabled")));
                    }
                }
                Cw20ExecuteMsg::TransferFrom { .. } | Cw20ExecuteMsg::SendFrom { .. } => {
                    return Err(cw20_base::ContractError::Std(cosmwasm_std::StdError::generic_err("TransferFrom/SendFrom disabled for AU.ET")));
                }
                Cw20ExecuteMsg::IncreaseAllowance { spender, .. } | Cw20ExecuteMsg::DecreaseAllowance { spender, .. } => {
                    // Only allow allowance to protocol modules
                    let addr = deps.api.addr_validate(spender)?;
                    if !ALLOWED_MODULE.may_load(deps.storage, &addr)?.unwrap_or(false) {
                        return Err(cw20_base::ContractError::Std(cosmwasm_std::StdError::generic_err("Allowances to non-protocol modules are disabled")));
                    }
                }
                _ => {}
            }
            // Forward allowed messages
            cw20_base::execute(deps, env, info, c)
        }
        ExecuteMsg::Spend { action, data: _, recipient, amount } => {
            // only allow spend to allowed module
            let recipient_addr = deps.api.addr_validate(&recipient)?;
            if !ALLOWED_MODULE.may_load(deps.storage, &recipient_addr)?.unwrap_or(false) {
                return Err(cw20_base::ContractError::Std(cosmwasm_std::StdError::generic_err("Spend to non-protocol module not allowed")));
            }
            // perform Transfer via cw20 base execute
            let transfer_msg = Cw20ExecuteMsg::Transfer { recipient: recipient.clone(), amount };
            cw20_base::execute(deps, env, info, transfer_msg)
        }
        ExecuteMsg::Burn { amount } => {
            // Use cw20-base burn path
            let burn_msg = Cw20ExecuteMsg::Burn { amount };
            cw20_base::execute(deps, env, info, burn_msg)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: Cw20QueryMsg) -> StdResult<Binary> {
    cw20_base::query(deps, env, msg)
}
