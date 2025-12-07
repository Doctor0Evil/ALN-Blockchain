#[cfg(test)]
mod tests {
    use super::super::{instantiate, execute, query, InstantiateMsg, ExecuteMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, Uint128, to_binary, Binary};
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

#[test]
fn claim_with_valid_merkle_proof_succeeds() {
    let mut deps = mock_dependencies();
    let gov = "gov".to_string();
    let reg_msg = crate::InstantiateMsg { governance_addr: gov.clone(), allow_missing_ubs: Some(true) };
    aln_registry::instantiate(deps.as_mut(), mock_env(), mock_info(&gov, &[]), reg_msg).unwrap();

    // create 3 snapshot entries and compute their hashes
    let s0 = crate::SnapshotEntry { chain_id: "k1".to_string(), height: 0, denom: "ibc/aaa".to_string(), address: "u0".to_string(), balance: "1".to_string() };
    let s1 = crate::SnapshotEntry { chain_id: "k1".to_string(), height: 0, denom: "ibc/bbb".to_string(), address: "u1".to_string(), balance: "2".to_string() };
    let s2 = crate::SnapshotEntry { chain_id: "k1".to_string(), height: 0, denom: "ibc/ccc".to_string(), address: "u2".to_string(), balance: "3".to_string() };
    // helper to compute hash
    fn compute_h(s: &crate::SnapshotEntry) -> [u8;32] {
        let mut hasher = Sha256::new();
        hasher.update(s.chain_id.as_bytes());
        hasher.update(&s.height.to_be_bytes());
        hasher.update(s.denom.as_bytes());
        hasher.update(s.address.as_bytes());
        let b: u128 = s.balance.parse().unwrap();
        hasher.update(&b.to_be_bytes());
        let res = hasher.finalize();
        let mut arr = [0u8;32];
        arr.copy_from_slice(&res);
        arr
    }

    let l0 = compute_h(&s0);
    let l1 = compute_h(&s1);
    let l2 = compute_h(&s2);

    // compute merkle root of 3 leaves: pair l0||l1 then parent || l2
    let mut hasher = Sha256::new();
    hasher.update(&l0); hasher.update(&l1);
    let p01 = hasher.finalize_reset();
    hasher.update(&p01); hasher.update(&l2);
    let root = hasher.finalize();
    let root_hex = format!("0x{}", hex::encode(root));

    // Build proof for leaf l1: sibling is l0 (left), and parent sibling is l2 (right)
    use cosmwasm_std::Binary;
    let proof = vec![
        crate::ProofStep { sibling: Binary(l0.to_vec()), is_left: true },
        crate::ProofStep { sibling: Binary(l2.to_vec()), is_left: false },
    ];

    // register asset with root
    let asset = aln_registry::RegisteredAsset {
        id: "b1".to_string(),
        source_chain: "k1".to_string(),
        source_denom: "ibc/x".to_string(),
        snapshot_height: 0,
        merkle_root: root_hex.clone(),
        ubs_report_hash: None,
        scaling_profile_id: "clean".to_string(),
        activation_height: 0,
        sanitized_approved: true,
    };
    let register = aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() };
    aln_registry::execute(deps.as_mut(), mock_env(), mock_info(&gov, &[]), register).unwrap();

    // instantiate bridge
    let bmsg = crate::InstantiateMsg { auet_contract: "auet_addr".to_string(), csp_contract: None, registry_contract: "reg".to_string(), governance_addr: gov.clone() };
    crate::instantiate(deps.as_mut(), mock_env(), mock_info(&gov, &[]), bmsg).unwrap();

    // claim with s1
    let hex_h = format!("0x{}", hex::encode(l1));
    let claim_msg = crate::ExecuteMsg::Claim { asset_id: "b1".to_string(), snapshot: s1.clone(), snapshot_hash: hex_h.clone(), merkle_proof: proof, amount_auet: Uint128::new(1), amount_csp: None };
    let res = crate::execute(deps.as_mut(), mock_env(), mock_info("u1", &[]), claim_msg).unwrap();
    assert!(res.attributes.iter().any(|a| a.value == "claim"));
}

#[test]
fn claim_with_invalid_merkle_proof_fails() {
    let mut deps = mock_dependencies();
    let gov = "gov".to_string();
    let reg_msg = crate::InstantiateMsg { governance_addr: gov.clone(), allow_missing_ubs: Some(true) };
    aln_registry::instantiate(deps.as_mut(), mock_env(), mock_info(&gov, &[]), reg_msg).unwrap();

    let s = crate::SnapshotEntry { chain_id: "k1".to_string(), height: 0, denom: "ibc/xxx".to_string(), address: "user".to_string(), balance: "1".to_string() };
    let mut hasher = Sha256::new();
    hasher.update(s.chain_id.as_bytes());
    hasher.update(&s.height.to_be_bytes());
    hasher.update(s.denom.as_bytes());
    hasher.update(s.address.as_bytes());
    let b: u128 = s.balance.parse().unwrap();
    hasher.update(&b.to_be_bytes());
    let digest = hasher.finalize();
    let hhex = format!("0x{}", hex::encode(digest));
    // Make a random different root
    let mut h2 = Sha256::new(); h2.update(b"other"); let r2 = h2.finalize(); let root = format!("0x{}", hex::encode(r2));

    let asset = aln_registry::RegisteredAsset { id: "c1".to_string(), source_chain: "k1".to_string(), source_denom: "ibc/xxx".to_string(), snapshot_height: 0, merkle_root: root.clone(), ubs_report_hash: None, scaling_profile_id: "clean".to_string(), activation_height: 0, sanitized_approved: true };
    let register = aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() };
    aln_registry::execute(deps.as_mut(), mock_env(), mock_info(&gov, &[]), register).unwrap();

    // instantiate bridge
    let bmsg = crate::InstantiateMsg { auet_contract: "auet_addr".to_string(), csp_contract: None, registry_contract: "reg".to_string(), governance_addr: gov.clone() };
    crate::instantiate(deps.as_mut(), mock_env(), mock_info(&gov, &[]), bmsg).unwrap();

    // Use empty proof which won't match root
    let claim_msg = crate::ExecuteMsg::Claim { asset_id: "c1".to_string(), snapshot: s.clone(), snapshot_hash: hhex.clone(), merkle_proof: vec![], amount_auet: Uint128::new(1), amount_csp: None };
    let err = crate::execute(deps.as_mut(), mock_env(), mock_info("user", &[]), claim_msg);
    assert!(err.is_err());
}
