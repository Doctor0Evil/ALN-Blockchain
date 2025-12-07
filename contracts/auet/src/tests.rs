#[cfg(test)]
mod tests {
    use super::super::{instantiate, execute, query, InstantiateMsg, SnapshotMeta};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, Uint128};
    use cw20::{Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse, BalanceResponse};
    use super::super::ExecuteMsg;
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

    #[test]
    fn instantiate_non_mintable() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            cw20: Cw20InstantiateMsg {
                name: "AU.ET".to_string(),
                symbol: "AUET".to_string(),
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
                height: 12_345_678,
                merkle_root: "abc123".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes, vec![attr("action", "instantiate")]);

        // total_supply is fixed
        let bin = query(
            deps.as_ref(),
            mock_env(),
            Cw20QueryMsg::TokenInfo {},
        )
        .unwrap();
        let token: TokenInfoResponse = cosmwasm_std::from_binary(&bin).unwrap();
        assert_eq!(token.total_supply, Uint128::new(1_000_000));
        assert!(token.mint.is_none());
    }

    #[test]
    fn transfer_disallowed_but_spend_works() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            cw20: Cw20InstantiateMsg {
                name: "AU.ET".to_string(),
                symbol: "AUET".to_string(),
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
        };

        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // transfer some tokens: should be disallowed (not a protocol module)
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

        // Now instantiate with a module whitelist and test Spend
        let mut deps2 = mock_dependencies();
        let msg2 = InstantiateMsg {
            cw20: Cw20InstantiateMsg {
                name: "AU.ET".to_string(),
                symbol: "AUET".to_string(),
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
            allowed_modules: Some(vec!["module".to_string()]),
        };
        instantiate(deps2.as_mut(), mock_env(), mock_info("creator", &[]), msg2).unwrap();

        // Spend as user to module should succeed
        let res = execute(
            deps2.as_mut(),
            mock_env(),
            mock_info("user", &[]),
            ExecuteMsg::Spend { action: "use".to_string(), data: None, recipient: "module".to_string(), amount: Uint128::new(100) },
        )
        .unwrap();
        // Ensure event action is present
        assert_eq!(res.attributes[0], attr("action", "transfer"));
    }
}
