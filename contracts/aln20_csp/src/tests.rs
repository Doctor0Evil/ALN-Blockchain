#[cfg(test)]
mod tests {
    use super::super::{instantiate, execute, query, InstantiateMsg, SnapshotMeta};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, Uint128};
    use cw20::{Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse, TokenInfoResponse};
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

    #[test]
    fn csp_transfer_restricted_by_default() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            cw20: Cw20InstantiateMsg {
                name: "CSP".to_string(),
                symbol: "CSP".to_string(),
                decimals: 6,
                initial_balances: vec![cw20::Cw20Coin {
                    address: "user".to_string(),
                    amount: Uint128::new(1_000_000),
                }],
                mint: None,
                marketing: None,
            },
            snapshot: SnapshotMeta {
                chain_id: "kaiyo-1".to_string(),
                height: 1,
                merkle_root: "root".to_string(),
            },
            transfer_whitelist: None,
        };

        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // transfer some tokens: should fail due to whitelist empty
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("user", &[]),
            Cw20ExecuteMsg::Transfer {
                recipient: "other".to_string(),
                amount: Uint128::new(100),
            },
        );
        assert!(err.is_err());
    }
}
