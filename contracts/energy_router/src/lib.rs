use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive { sender: String, amount: Uint128, msg: Option<Binary> },
    UseAbility { ability_id: String, cost_energy: Uint128 },
    UnlockAbility { ability_id: String, cost_csp: Uint128 },
}

#[entry_point]
pub fn instantiate(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: InstantiateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive { sender, amount, msg: _ } => {
            Ok(Response::new().add_attribute("action", "receive").add_attribute("from", sender).add_attribute("amount", amount.to_string()))
        }
        ExecuteMsg::UseAbility { ability_id, cost_energy } => {
            Ok(Response::new().add_attribute("action", "use_ability").add_attribute("ability_id", ability_id).add_attribute("cost", cost_energy.to_string()))
        }
        ExecuteMsg::UnlockAbility { ability_id, cost_csp } => {
            Ok(Response::new().add_attribute("action", "unlock_ability").add_attribute("ability_id", ability_id).add_attribute("cost", cost_csp.to_string()))
        }
    }
}

// TODO: Provide query API and implementation when needed
