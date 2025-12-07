#[cfg(test)]
mod tests {
    use super::super::{instantiate, execute, query, InstantiateMsg, ExecuteMsg, RegisteredAsset};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;

    #[test]
    fn register_and_approve_sanitized() {
        let mut deps = mock_dependencies();
        let gov = "gov".to_string();
        let msg = InstantiateMsg { governance_addr: gov.clone(), allow_missing_ubs: Some(true) };
        instantiate(deps.as_mut(), mock_env(), mock_info(&gov, &[]), msg).unwrap();

        let asset = RegisteredAsset {
            id: "a1".to_string(),
            source_chain: "kaiyo-1".to_string(),
            source_denom: "ibc/xxx".to_string(),
            snapshot_height: 123,
            merkle_root: "root".to_string(),
            ubs_report_hash: None,
            scaling_profile_id: "malicious_cleanup".to_string(),
            activation_height: 0,
            sanitized_approved: false,
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(&gov, &[]), ExecuteMsg::RegisterAsset { asset: asset.clone() }).unwrap();
        assert_eq!(res.attributes[0].value, "register_asset");

        // non gov cannot approve
        let err = execute(deps.as_mut(), mock_env(), mock_info("notgov", &[]), ExecuteMsg::ApproveSanitized { id: "a1".to_string(), ubs_report_hash: "h1".to_string() });
        assert!(err.is_err());

        // gov approves
        let res2 = execute(deps.as_mut(), mock_env(), mock_info(&gov, &[]), ExecuteMsg::ApproveSanitized { id: "a1".to_string(), ubs_report_hash: "h1".to_string() }).unwrap();
        assert_eq!(res2.attributes[0].value, "approve_sanitized");
    }
}
