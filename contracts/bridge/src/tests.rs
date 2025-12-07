#[cfg(test)]
mod tests {
    use super::super::{instantiate, execute, query, InstantiateMsg, ExecuteMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, Uint128, to_binary};
    use sha2::{Sha256, Digest};
    use hex;

    use crate::ExecuteMsg;

    #[test]
    fn claim_and_replay_protection() {
        let mut deps = mock_dependencies();

        // 1) Instantiate registry and register an asset by governance
        let gov = "gov".to_string();
        let reg_msg = crate::InstantiateMsg { governance_addr: gov.clone(), allow_missing_ubs: Some(true) };
        aln_registry::instantiate(deps.as_mut(), mock_env(), mock_info(&gov, &[]), reg_msg).unwrap();

        let s = crate::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 0, denom: "ibc/xxx".to_string(), address: "user".to_string(), balance: "1".to_string() };
        let mut hasher = Sha256::new();
        hasher.update(s.chain_id.as_bytes());
        hasher.update(&s.height.to_be_bytes());
        hasher.update(s.denom.as_bytes());
        hasher.update(s.address.as_bytes());
        let b: u128 = s.balance.parse().unwrap();
        hasher.update(&b.to_be_bytes());
        let digest = hasher.finalize();
        let hhex = format!("0x{}", hex::encode(digest));
        let asset = aln_registry::RegisteredAsset {
            id: "a1".to_string(),
            source_chain: "kaiyo-1".to_string(),
            source_denom: "ibc/xxx".to_string(),
            snapshot_height: 0,
            merkle_root: hhex.clone(),
            ubs_report_hash: None,
            scaling_profile_id: "malicious_cleanup".to_string(),
            activation_height: 0,
            sanitized_approved: false,
        };
        let register = aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() };
        let reg_addr = "competition"; // using placeholder as we call local function directly
        // call register as governance
        aln_registry::execute(deps.as_mut(), mock_env(), mock_info(&gov, &[]), register).unwrap();

        // 2) Instantiate bridge with registry and gov
        let bmsg = InstantiateMsg { auet_contract: "auet_addr".to_string(), csp_contract: None, registry_contract: "reg".to_string(), governance_addr: gov.clone() };
        instantiate(deps.as_mut(), mock_env(), mock_info(&gov, &[]), bmsg).unwrap();

        // Claim before sanitized approval should fail
        let claim_msg = ExecuteMsg::Claim { asset_id: "a1".to_string(), snapshot: s.clone(), snapshot_hash: hhex.clone(), merkle_proof: vec![], amount_auet: Uint128::new(1), amount_csp: None };
        let err = execute(deps.as_mut(), mock_env(), mock_info("user", &[]), claim_msg);
        assert!(err.is_err());

        // Approve sanitized as governance on registry
        let approve = aln_registry::ExecuteMsg::ApproveSanitized { id: "a1".to_string(), ubs_report_hash: "h1".to_string() };
        aln_registry::execute(deps.as_mut(), mock_env(), mock_info(&gov, &[]), approve).unwrap();

        // Now claim should succeed and set claimed
        let claim_msg2 = ExecuteMsg::Claim { asset_id: "a1".to_string(), snapshot: s.clone(), snapshot_hash: hhex.clone(), merkle_proof: vec![], amount_auet: Uint128::new(1), amount_csp: None };
        let res = execute(deps.as_mut(), mock_env(), mock_info("user", &[]), claim_msg2).unwrap();
        assert!(res.attributes.iter().any(|a| a.value == "claim_refactored"));

        // Second claim should fail
        let claim_msg3 = ExecuteMsg::Claim { asset_id: "a1".to_string(), snapshot: s.clone(), snapshot_hash: hhex.clone(), merkle_proof: vec![], amount_auet: Uint128::new(1), amount_csp: None };
        let err2 = execute(deps.as_mut(), mock_env(), mock_info("user", &[]), claim_msg3);
        assert!(err2.is_err());

        // Query claimed should be true
        let q = QueryMsg::IsClaimed { address: "user".to_string(), asset_id: "a1".to_string(), snapshot_hash: hhex.clone() };
        let bin = query(deps.as_ref(), mock_env(), q).unwrap();
        let claimed: bool = cosmwasm_std::from_binary(&bin).unwrap();
        assert!(claimed);
    }
}
