use anyhow::Result;
use cw_multi_test::{App, ContractWrapper, Executor};
use cosmwasm_std::{Addr, Uint128, Binary};

// Import contracts' instantiate / execute functions
use aln_auet::{instantiate as auet_instantiate, execute as auet_execute, query as auet_query};
use aln_csp::{instantiate as csp_instantiate, execute as csp_execute, query as csp_query};
use aln_registry::{instantiate as reg_instantiate, execute as reg_execute, query as reg_query};
use aln_bridge::{instantiate as bridge_instantiate, execute as bridge_execute, query as bridge_query};
use energy_router::{instantiate as router_instantiate, execute as router_execute, query as router_query};

use cw_multi_test::AppResponse;
use cw20::Cw20Coin;
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};
use sha2::{Digest, Sha256};
use hex;

#[test]
fn happy_path_claim_and_spend() -> Result<()> {
    let mut app = App::default();

    // Wrap contracts
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let csp_contract = ContractWrapper::new(csp_instantiate, csp_execute, csp_query);
    let csp_code = app.store_code(Box::new(csp_contract));
    let reg_contract = ContractWrapper::new(reg_instantiate, reg_execute, reg_query);
    let reg_code = app.store_code(Box::new(reg_contract));
    let bridge_contract = ContractWrapper::new(bridge_instantiate, bridge_execute, bridge_query);
    let bridge_code = app.store_code(Box::new(bridge_contract));
    let router_contract = ContractWrapper::new(router_instantiate, router_execute, router_query);
    let router_code = app.store_code(Box::new(router_contract));

    // Setup addresses
    let gov = Addr::unchecked("gov");
    let bridge_admin = Addr::unchecked("bridge_admin");
    let bridge_addr = Addr::unchecked("bridge");
    let user = Addr::unchecked("user");
    let energy_router_label = "energy_router";

    // Instantiate energy router
    let router_msg = energy_router::InstantiateMsg { owner: "owner".to_string() };
    let router_addr = app.instantiate_contract(router_code, Addr::unchecked("creator"), &router_msg, &[], energy_router_label, None)?;

    // Instantiate AU.ET with bridge having initial balances
    let auet_instantiate_msg = aln_auet::InstantiateMsg {
        cw20: Cw20InstantiateMsg {
            name: "AU.ET".to_string(),
            symbol: "AUET".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(1_000_000) }],
            mint: None,
            marketing: None,
        },
        snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 123, merkle_root: "root".to_string() },
        allowed_modules: Some(vec![router_addr.to_string()]),
    };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &auet_instantiate_msg, &[], "AUET", None)?;

    // Instantiate CSP with bridge holding initial balances
    let csp_instantiate_msg = aln_csp::InstantiateMsg {
        cw20: Cw20InstantiateMsg {
            name: "CSP".to_string(),
            symbol: "CSP".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(100) }],
            mint: None,
            marketing: None,
        },
        snapshot: aln_csp::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 123, merkle_root: "root".to_string() },
        transfer_whitelist: Some(vec![router_addr.to_string()]),
    };
    let csp_addr = app.instantiate_contract(csp_code, Addr::unchecked("creator"), &csp_instantiate_msg, &[], "CSP", None)?;

    // Instantiate registry with governance
    let reg_instantiate_msg = aln_registry::InstantiateMsg { governance_addr: gov.to_string(), allow_missing_ubs: Some(true) };
    let reg_addr = app.instantiate_contract(reg_code, Addr::unchecked("creator"), &reg_instantiate_msg, &[], "REG", None)?;

    // Register and approve asset via governance. Build a snapshot entry for the user and make merkle_root == H_i for single-leaf tree
    let s_user = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 123, denom: "ibc/xxx".to_string(), address: user.to_string(), balance: "100".to_string() };
    let h_user = compute_snapshot_hash(&s_user);
    let asset = aln_registry::RegisteredAsset { id: "a1".to_string(), source_chain: "kaiyo-1".to_string(), source_denom: "ibc/xxx".to_string(), snapshot_height: 123, merkle_root: h_user.clone(), ubs_report_hash: Some("h1".to_string()), scaling_profile_id: "malicious_cleanup".to_string(), activation_height: 0, sanitized_approved: true };
    let reg_msg = aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() };
    app.execute_contract(gov.clone(), reg_addr.clone(), &reg_msg, &[])?;
    let approve_msg = aln_registry::ExecuteMsg::ApproveSanitized { id: "a1".to_string(), ubs_report_hash: "h1".to_string() };
    app.execute_contract(gov.clone(), reg_addr.clone(), &approve_msg, &[])?;

    // Instantiate bridge with references and governance
    let bridge_instantiate_msg = aln_bridge::InstantiateMsg { auet_contract: auet_addr.to_string(), csp_contract: Some(csp_addr.to_string()), registry_contract: reg_addr.to_string(), governance_addr: gov.to_string() };
    let bridge_addr_inst = app.instantiate_contract(bridge_code, Addr::unchecked("creator"), &bridge_instantiate_msg, &[], "BRIDGE", None)?;

    // For the bridge to perform transfers, it must have AU.ET/CSP balances; ensure bridge account holds them. They were set as initial balances earlier.

    // Happy-path claim: user claims some AUET and CSP
    let claim_msg = aln_bridge::ExecuteMsg::Claim { asset_id: "a1".to_string(), snapshot: s_user.clone(), snapshot_hash: h_user.clone(), merkle_proof: vec![], amount_auet: Uint128::new(100), amount_csp: Some(Uint128::new(10)) };
    app.execute_contract(user.clone(), bridge_addr_inst.clone(), &claim_msg, &[])?;

    // Verify claimed is set on bridge
    let q = aln_bridge::QueryMsg::IsClaimed { address: user.to_string(), asset_id: "a1".to_string(), snapshot_hash: h_user.clone() };
    let claimed_bin = app.wrap().query_wasm_smart(bridge_addr_inst.clone(), &q)?;
    let claimed_res: bool = serde_json::from_str(&serde_json::to_string(&claimed_bin)?)?;
    assert!(claimed_res);

    // Verify AUET balance of user increased
    let bal_q = Cw20QueryMsg::Balance { address: user.to_string() };
    let bal_bin = app.wrap().query_wasm_smart(auet_addr.clone(), &bal_q)?;
    let bal: cw20::BalanceResponse = serde_json::from_str(&serde_json::to_string(&bal_bin)?)?;
    assert_eq!(bal.balance, Uint128::new(100));

    // Now test Spend: user spends AUET to energy_router
    let spend = aln_auet::ExecuteMsg::Spend { action: "use".to_string(), data: None, recipient: router_addr.to_string(), amount: Uint128::new(50) };
    app.execute_contract(user.clone(), auet_addr.clone(), &spend, &[])?;

    // Verify energy router received funds - via Query on router or checking balances
    let router_balance_q = Cw20QueryMsg::Balance { address: router_addr.to_string() };
    let rb_bin = app.wrap().query_wasm_smart(auet_addr.clone(), &router_balance_q)?;
    let rb: cw20::BalanceResponse = serde_json::from_str(&serde_json::to_string(&rb_bin)?)?;
    assert_eq!(rb.balance, Uint128::new(50));

    Ok(())
}

#[test]
fn claim_before_sanitized_rejected() -> Result<()> {
    let mut app = App::default();
    // Setup minimal env like in the happy test but do not approve sanitized
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let reg_contract = ContractWrapper::new(reg_instantiate, reg_execute, reg_query);
    let reg_code = app.store_code(Box::new(reg_contract));
    let bridge_contract = ContractWrapper::new(bridge_instantiate, bridge_execute, bridge_query);
    let bridge_code = app.store_code(Box::new(bridge_contract));

    let gov = Addr::unchecked("gov");
    let bridge_addr = Addr::unchecked("bridge");

    let auet_instantiate_msg = aln_auet::InstantiateMsg {
        cw20: Cw20InstantiateMsg {
            name: "AU.ET".to_string(),
            symbol: "AUET".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(1_000_000) }],
            mint: None,
            marketing: None,
        },
        snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 123, merkle_root: "root".to_string() },
        allowed_modules: Some(vec![]),
    };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &auet_instantiate_msg, &[], "AUET", None)?;

    // instantiate registry
    let reg_msg = aln_registry::InstantiateMsg { governance_addr: gov.to_string(), allow_missing_ubs: Some(true) };
    let reg_addr = app.instantiate_contract(reg_code, Addr::unchecked("creator"), &reg_msg, &[], "REG", None)?;
    // register asset but do not approve sanitized
    let s2 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 0, denom: "ibc/yyy".to_string(), address: "user".to_string(), balance: "1".to_string() };
    let h2 = compute_snapshot_hash(&s2);
    let asset = aln_registry::RegisteredAsset { id: "a2".to_string(), source_chain: "kaiyo-1".to_string(), source_denom: "ibc/yyy".to_string(), snapshot_height: 0, merkle_root: h2.clone(), ubs_report_hash: None, scaling_profile_id: "malicious_cleanup".to_string(), activation_height: 0, sanitized_approved: false };
    let reg_msg2 = aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() };
    app.execute_contract(gov.clone(), reg_addr.clone(), &reg_msg2, &[])?;

    // instantiate bridge
    let bridge_msg = aln_bridge::InstantiateMsg { auet_contract: auet_addr.to_string(), csp_contract: None, registry_contract: reg_addr.to_string(), governance_addr: gov.to_string() };
    let bridge_addr_inst = app.instantiate_contract(bridge_code, Addr::unchecked("creator"), &bridge_msg, &[], "BRIDGE", None)?;

    // attempt claim should fail because not sanitized
    let claim_msg = aln_bridge::ExecuteMsg::Claim { asset_id: "a2".to_string(), snapshot: s2.clone(), snapshot_hash: h2.clone(), merkle_proof: vec![], amount_auet: Uint128::new(1), amount_csp: None };
    let res = app.execute_contract(Addr::unchecked("user"), bridge_addr_inst.clone(), &claim_msg, &[]);
    assert!(res.is_err());
    Ok(())
}

#[test]
fn double_claim_rejected() -> Result<()> {
    let mut app = App::default();
    // Similar setup to happy_path, register and approve
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let reg_contract = ContractWrapper::new(reg_instantiate, reg_execute, reg_query);
    let reg_code = app.store_code(Box::new(reg_contract));
    let bridge_contract = ContractWrapper::new(bridge_instantiate, bridge_execute, bridge_query);
    let bridge_code = app.store_code(Box::new(bridge_contract));

    let gov = Addr::unchecked("gov");
    let bridge_addr = Addr::unchecked("bridge");
    let auet_instantiate_msg = aln_auet::InstantiateMsg {
        cw20: Cw20InstantiateMsg {
            name: "AU.ET".to_string(),
            symbol: "AUET".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(1_000_000) }],
            mint: None,
            marketing: None,
        },
        snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 123, merkle_root: "root".to_string() },
        allowed_modules: Some(vec![]),
    };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &auet_instantiate_msg, &[], "AUET", None)?;
    let reg_msg = aln_registry::InstantiateMsg { governance_addr: gov.to_string(), allow_missing_ubs: Some(true) };
    let reg_addr = app.instantiate_contract(reg_code, Addr::unchecked("creator"), &reg_msg, &[], "REG", None)?;
    let s3 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 0, denom: "ibc/zzz".to_string(), address: "user".to_string(), balance: "1".to_string() };
    let h3 = compute_snapshot_hash(&s3);
    let asset = aln_registry::RegisteredAsset { id: "a3".to_string(), source_chain: "kaiyo-1".to_string(), source_denom: "ibc/zzz".to_string(), snapshot_height: 0, merkle_root: h3.clone(), ubs_report_hash: Some("h3".to_string()), scaling_profile_id: "malicious_cleanup".to_string(), activation_height: 0, sanitized_approved: true };
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() }, &[])?;
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::ApproveSanitized { id: "a3".to_string(), ubs_report_hash: "h3".to_string() }, &[])?;
    let bridge_msg = aln_bridge::InstantiateMsg { auet_contract: auet_addr.to_string(), csp_contract: None, registry_contract: reg_addr.to_string(), governance_addr: gov.to_string() };
    let bridge_addr_inst = app.instantiate_contract(bridge_code, Addr::unchecked("creator"), &bridge_msg, &[], "BRIDGE", None)?;

    // First claim should succeed
    let claim_msg = aln_bridge::ExecuteMsg::Claim { asset_id: "a3".to_string(), snapshot: s3.clone(), snapshot_hash: h3.clone(), merkle_proof: vec![], amount_auet: Uint128::new(1), amount_csp: None };
    let res = app.execute_contract(Addr::unchecked("user"), bridge_addr_inst.clone(), &claim_msg, &[])?;
    assert!(res.attributes.iter().any(|a| a.value == "claim_refactored"));

    // Second claim should fail
    let claim_msg2 = aln_bridge::ExecuteMsg::Claim { asset_id: "a3".to_string(), snapshot: s3.clone(), snapshot_hash: h3.clone(), merkle_proof: vec![], amount_auet: Uint128::new(1), amount_csp: None };
    let err = app.execute_contract(Addr::unchecked("user"), bridge_addr_inst.clone(), &claim_msg2, &[]);
    assert!(err.is_err());
    Ok(())
}

#[test]
fn auet_user_to_user_transfer_rejected() -> Result<()> {
    let mut app = App::default();
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let user = Addr::unchecked("user");
    let other = Addr::unchecked("other");
    let instantiate_msg = aln_auet::InstantiateMsg {
        cw20: Cw20InstantiateMsg {
            name: "AU.ET".to_string(),
            symbol: "AUET".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin { address: user.to_string(), amount: Uint128::new(1000) }],
            mint: None,
            marketing: None,
        },
        snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 0, merkle_root: "root".to_string() },
        allowed_modules: Some(vec![]),
    };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &instantiate_msg, &[], "AUET", None)?;
    let transfer_msg = Cw20ExecuteMsg::Transfer { recipient: other.to_string(), amount: Uint128::new(50) };
    let res = app.execute_contract(user.clone(), auet_addr.clone(), &transfer_msg, &[]);
    assert!(res.is_err());
    Ok(())
}

#[test]
fn csp_transfer_rejected() -> Result<()> {
    let mut app = App::default();
    let csp_contract = ContractWrapper::new(csp_instantiate, csp_execute, csp_query);
    let csp_code = app.store_code(Box::new(csp_contract));
    let user = Addr::unchecked("user");
    let other = Addr::unchecked("other");
    let instantiate_msg = aln_csp::InstantiateMsg {
        cw20: Cw20InstantiateMsg {
            name: "CSP".to_string(),
            symbol: "CSP".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin { address: user.to_string(), amount: Uint128::new(100) }],
            mint: None,
            marketing: None,
        },
        snapshot: aln_csp::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 0, merkle_root: "root".to_string() },
        transfer_whitelist: None,
    };
    let csp_addr = app.instantiate_contract(csp_code, Addr::unchecked("creator"), &instantiate_msg, &[], "CSP", None)?;
    let transfer_msg = Cw20ExecuteMsg::Transfer { recipient: other.to_string(), amount: Uint128::new(10) };
    let res = app.execute_contract(user.clone(), csp_addr.clone(), &transfer_msg, &[]);
    assert!(res.is_err());
    Ok(())
}

// Helper: compute snapshot H_i (0xhex) consistent with tooling & bridge contract
fn compute_snapshot_hash(s: &aln_bridge::SnapshotEntry) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.chain_id.as_bytes());
    hasher.update(&s.height.to_be_bytes());
    hasher.update(s.denom.as_bytes());
    hasher.update(s.address.as_bytes());
    let b: u128 = s.balance.parse().unwrap();
    hasher.update(&b.to_be_bytes());
    format!("0x{}", hex::encode(hasher.finalize()))
}

fn build_merkle_and_proofs(leaves: &Vec<[u8;32]>) -> (String, Vec<Vec<(Vec<u8>, bool)>>) {
    if leaves.is_empty() { return ("".to_string(), vec![]); }
    let mut levels: Vec<Vec<[u8;32]>> = vec![leaves.clone()];
    while levels.last().unwrap().len() > 1 {
        let prev = levels.last().unwrap().clone();
        let mut next: Vec<[u8;32]> = vec![];
        for i in (0..prev.len()).step_by(2) {
            let left = prev[i];
            let right = if i+1 < prev.len() { prev[i+1] } else { prev[i] };
            let mut hasher = Sha256::new();
            hasher.update(&left);
            hasher.update(&right);
            let mut out = [0u8;32];
            out.copy_from_slice(&hasher.finalize());
            next.push(out);
        }
        levels.push(next);
    }
    let root = levels.last().unwrap()[0];
    let mut proofs: Vec<Vec<(Vec<u8>, bool)>> = vec![];
    let leaf_count = leaves.len();
    for idx in 0..leaf_count {
        let mut cur_index = idx;
        let mut proof_steps: Vec<(Vec<u8>, bool)> = vec![];
        for lvl in 0..(levels.len()-1) {
            let level_nodes = &levels[lvl];
            let pair_idx = if cur_index % 2 == 0 { cur_index + 1 } else { cur_index - 1 };
            if pair_idx < level_nodes.len() {
                let sibling = level_nodes[pair_idx];
                proof_steps.push((sibling.to_vec(), pair_idx < cur_index));
            } else {
                let sibling = level_nodes[cur_index];
                proof_steps.push((sibling.to_vec(), pair_idx < cur_index));
            }
            cur_index = cur_index / 2;
        }
        proofs.push(proof_steps);
    }
    (format!("0x{}", hex::encode(root)), proofs)
}

#[test]
fn claim_with_valid_merkle_proof_succeeds() -> Result<()> {
    let mut app = App::default();
    // Register contracts and instantiate similar to happy_path
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let reg_contract = ContractWrapper::new(reg_instantiate, reg_execute, reg_query);
    let reg_code = app.store_code(Box::new(reg_contract));
    let bridge_contract = ContractWrapper::new(bridge_instantiate, bridge_execute, bridge_query);
    let bridge_code = app.store_code(Box::new(bridge_contract));

    // prepare two snapshot entries
    let s1 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 100, denom: "ibc/xxx".to_string(), address: "user".to_string(), balance: "1000".to_string() };
    let s2 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 100, denom: "ibc/xxx".to_string(), address: "other".to_string(), balance: "2000".to_string() };
    let h1 = compute_snapshot_hash(&s1);
    let h2 = compute_snapshot_hash(&s2);
    let mut leaves: Vec<[u8;32]> = vec![];
    for h in &vec![h1.clone(), h2.clone()] {
        let mut arr = [0u8;32];
        let bytes = hex::decode(h.trim_start_matches("0x")).unwrap();
        arr.copy_from_slice(&bytes);
        leaves.push(arr);
    }
    let (root, proofs) = build_merkle_and_proofs(&leaves);

    // Instantiate registry with merkle root and approve
    let gov = Addr::unchecked("gov");
    let reg_instantiate_msg = aln_registry::InstantiateMsg { governance_addr: gov.to_string(), allow_missing_ubs: Some(true) };
    let reg_addr = app.instantiate_contract(reg_code, Addr::unchecked("creator"), &reg_instantiate_msg, &[], "REG", None)?;
    let asset = aln_registry::RegisteredAsset { id: "m1".to_string(), source_chain: "kaiyo-1".to_string(), source_denom: "ibc/xxx".to_string(), snapshot_height: 100, merkle_root: root.clone(), ubs_report_hash: Some("h1".to_string()), scaling_profile_id: "malicious_cleanup".to_string(), activation_height: 0, sanitized_approved: true };
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() }, &[])?;
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::ApproveSanitized { id: "m1".to_string(), ubs_report_hash: "h1".to_string() }, &[])?;

    // Instantiate AU.ET with the bridge address to hold initial funds
    let bridge_addr = Addr::unchecked("bridge");
    let auet_instantiate_msg = aln_auet::InstantiateMsg {
        cw20: Cw20InstantiateMsg { name: "AU.ET".to_string(), symbol: "AUET".to_string(), decimals: 6, initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(1_000_000) }], mint: None, marketing: None },
        snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 100, merkle_root: root.clone() },
        allowed_modules: Some(vec![]),
    };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &auet_instantiate_msg, &[], "AUET", None)?;

    // instantiate bridge
    let bridge_instantiate_msg = aln_bridge::InstantiateMsg { auet_contract: auet_addr.to_string(), csp_contract: None, registry_contract: reg_addr.to_string(), governance_addr: gov.to_string() };
    let bridge_addr_inst = app.instantiate_contract(bridge_code, Addr::unchecked("creator"), &bridge_instantiate_msg, &[], "BRIDGE", None)?;

    // Build proof for s1 (index 0)
    let proof_bytes = proofs[0].clone();
    let proof_steps: Vec<aln_bridge::ProofStep> = proof_bytes.iter().map(|(b, is_left)| { aln_bridge::ProofStep { sibling: Binary(b.clone()), is_left: *is_left } }).collect();

    // claim
    let claim_msg = aln_bridge::ExecuteMsg::Claim { asset_id: "m1".to_string(), snapshot: s1.clone(), snapshot_hash: h1.clone(), merkle_proof: proof_steps, amount_auet: Uint128::new(100), amount_csp: None };
    app.execute_contract(Addr::unchecked("user"), bridge_addr_inst.clone(), &claim_msg, &[])?;

    // verify claimed and auet balance updated
    let q = aln_bridge::QueryMsg::IsClaimed { address: "user".to_string(), asset_id: "m1".to_string(), snapshot_hash: h1.clone() };
    let claimed_bin = app.wrap().query_wasm_smart(bridge_addr_inst.clone(), &q)?;
    let claimed_res: bool = serde_json::from_str(&serde_json::to_string(&claimed_bin)?)?;
    assert!(claimed_res);

    let bal_q = Cw20QueryMsg::Balance { address: "user".to_string() };
    let bal_bin = app.wrap().query_wasm_smart(auet_addr.clone(), &bal_q)?;
    let bal: cw20::BalanceResponse = serde_json::from_str(&serde_json::to_string(&bal_bin)?)?;
    assert_eq!(bal.balance, Uint128::new(100));

    Ok(())
}

#[test]
fn claim_with_invalid_merkle_proof_fails() -> Result<()> {
    let mut app = App::default();
    // similar setup, but we use an invalid proof (flip a byte)
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let reg_contract = ContractWrapper::new(reg_instantiate, reg_execute, reg_query);
    let reg_code = app.store_code(Box::new(reg_contract));
    let bridge_contract = ContractWrapper::new(bridge_instantiate, bridge_execute, bridge_query);
    let bridge_code = app.store_code(Box::new(bridge_contract));
    let s1 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 100, denom: "ibc/xxx".to_string(), address: "user".to_string(), balance: "1000".to_string() };
    let s2 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 100, denom: "ibc/xxx".to_string(), address: "other".to_string(), balance: "2000".to_string() };
    let h1 = compute_snapshot_hash(&s1);
    let h2 = compute_snapshot_hash(&s2);
    let mut leaves: Vec<[u8;32]> = vec![];
    for h in &vec![h1.clone(), h2.clone()] { let mut arr = [0u8;32]; let bytes = hex::decode(h.trim_start_matches("0x")).unwrap(); arr.copy_from_slice(&bytes); leaves.push(arr);}    
    let (root, proofs) = build_merkle_and_proofs(&leaves);
    let gov = Addr::unchecked("gov");
    let reg_instantiate_msg = aln_registry::InstantiateMsg { governance_addr: gov.to_string(), allow_missing_ubs: Some(true) };
    let reg_addr = app.instantiate_contract(reg_code, Addr::unchecked("creator"), &reg_instantiate_msg, &[], "REG", None)?;
    let asset = aln_registry::RegisteredAsset { id: "m2".to_string(), source_chain: "kaiyo-1".to_string(), source_denom: "ibc/xxx".to_string(), snapshot_height: 100, merkle_root: root.clone(), ubs_report_hash: Some("h2".to_string()), scaling_profile_id: "malicious_cleanup".to_string(), activation_height: 0, sanitized_approved: true };
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() }, &[])?;
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::ApproveSanitized { id: "m2".to_string(), ubs_report_hash: "h2".to_string() }, &[])?;
    let bridge_addr = Addr::unchecked("bridge");
    let auet_instantiate_msg = aln_auet::InstantiateMsg { cw20: Cw20InstantiateMsg { name: "AU.ET".to_string(), symbol: "AUET".to_string(), decimals: 6, initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(1_000_000) }], mint: None, marketing: None }, snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 100, merkle_root: root.clone() }, allowed_modules: Some(vec![]) };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &auet_instantiate_msg, &[], "AUET", None)?;
    let bridge_instantiate_msg = aln_bridge::InstantiateMsg { auet_contract: auet_addr.to_string(), csp_contract: None, registry_contract: reg_addr.to_string(), governance_addr: gov.to_string() };
    let bridge_addr_inst = app.instantiate_contract(bridge_code, Addr::unchecked("creator"), &bridge_instantiate_msg, &[], "BRIDGE", None)?;
    // create a proof but smash it
    let mut wrong_proof = proofs[0].clone();
    wrong_proof[0].0[0] ^= 0xff; // flip a byte
    let proof_steps: Vec<aln_bridge::ProofStep> = wrong_proof.iter().map(|(b, is_left)| aln_bridge::ProofStep { sibling: Binary(b.clone()), is_left: *is_left }).collect();
    let claim_msg = aln_bridge::ExecuteMsg::Claim { asset_id: "m2".to_string(), snapshot: s1.clone(), snapshot_hash: h1.clone(), merkle_proof: proof_steps, amount_auet: Uint128::new(100), amount_csp: None };
    let res = app.execute_contract(Addr::unchecked("user"), bridge_addr_inst.clone(), &claim_msg, &[]);
    assert!(res.is_err());
    Ok(())
}

#[test]
fn claim_with_modified_snapshot_entry_fails() -> Result<()> {
    let mut app = App::default();
    // similar setup as valid test
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let reg_contract = ContractWrapper::new(reg_instantiate, reg_execute, reg_query);
    let reg_code = app.store_code(Box::new(reg_contract));
    let bridge_contract = ContractWrapper::new(bridge_instantiate, bridge_execute, bridge_query);
    let bridge_code = app.store_code(Box::new(bridge_contract));
    let s1 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 100, denom: "ibc/xxx".to_string(), address: "user2".to_string(), balance: "1000".to_string() };
    let h1 = compute_snapshot_hash(&s1);
    let mut arr = [0u8;32];
    let bytes = hex::decode(h1.trim_start_matches("0x")).unwrap(); arr.copy_from_slice(&bytes);
    let (root, proofs) = build_merkle_and_proofs(&vec![arr]);
    let gov = Addr::unchecked("gov");
    let reg_instantiate_msg = aln_registry::InstantiateMsg { governance_addr: gov.to_string(), allow_missing_ubs: Some(true) };
    let reg_addr = app.instantiate_contract(reg_code, Addr::unchecked("creator"), &reg_instantiate_msg, &[], "REG", None)?;
    let asset = aln_registry::RegisteredAsset { id: "m3".to_string(), source_chain: "kaiyo-1".to_string(), source_denom: "ibc/xxx".to_string(), snapshot_height: 100, merkle_root: root.clone(), ubs_report_hash: Some("h3".to_string()), scaling_profile_id: "malicious_cleanup".to_string(), activation_height: 0, sanitized_approved: true };
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() }, &[])?;
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::ApproveSanitized { id: "m3".to_string(), ubs_report_hash: "h3".to_string() }, &[])?;
    let bridge_addr = Addr::unchecked("bridge");
    let auet_instantiate_msg = aln_auet::InstantiateMsg { cw20: Cw20InstantiateMsg { name: "AU.ET".to_string(), symbol: "AUET".to_string(), decimals: 6, initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(1_000_000) }], mint: None, marketing: None }, snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 100, merkle_root: root.clone() }, allowed_modules: Some(vec![]) };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &auet_instantiate_msg, &[], "AUET", None)?;
    let bridge_instantiate_msg = aln_bridge::InstantiateMsg { auet_contract: auet_addr.to_string(), csp_contract: None, registry_contract: reg_addr.to_string(), governance_addr: gov.to_string() };
    let bridge_addr_inst = app.instantiate_contract(bridge_code, Addr::unchecked("creator"), &bridge_instantiate_msg, &[], "BRIDGE", None)?;
    // using the proof for s1, but tamper with the snapshot entry balance
    let proof_bytes = proofs[0].clone();
    let proof_steps: Vec<aln_bridge::ProofStep> = proof_bytes.iter().map(|(b, is_left)| aln_bridge::ProofStep { sibling: Binary(b.clone()), is_left: *is_left }).collect();
    let mut tampered = s1.clone();
    tampered.balance = "1001".to_string();
    let claim_msg = aln_bridge::ExecuteMsg::Claim { asset_id: "m3".to_string(), snapshot: tampered, snapshot_hash: h1.clone(), merkle_proof: proof_steps, amount_auet: Uint128::new(10), amount_csp: None };
    let res = app.execute_contract(Addr::unchecked("user2"), bridge_addr_inst.clone(), &claim_msg, &[]);
    assert!(res.is_err());
    Ok(())
}

#[test]
fn claim_before_activation_height_fails() -> Result<()> {
    let mut app = App::default();
    let auet_contract = ContractWrapper::new(auet_instantiate, auet_execute, auet_query);
    let auet_code = app.store_code(Box::new(auet_contract));
    let reg_contract = ContractWrapper::new(reg_instantiate, reg_execute, reg_query);
    let reg_code = app.store_code(Box::new(reg_contract));
    let bridge_contract = ContractWrapper::new(bridge_instantiate, bridge_execute, bridge_query);
    let bridge_code = app.store_code(Box::new(bridge_contract));

    let s1 = aln_bridge::SnapshotEntry { chain_id: "kaiyo-1".to_string(), height: 100, denom: "ibc/xxx".to_string(), address: "delayed_user".to_string(), balance: "1000".to_string() };
    let h1 = compute_snapshot_hash(&s1);
    let mut arr = [0u8;32];
    let bytes = hex::decode(h1.trim_start_matches("0x")).unwrap(); arr.copy_from_slice(&bytes);
    let (root, proofs) = build_merkle_and_proofs(&vec![arr]);

    let gov = Addr::unchecked("gov");
    let reg_instantiate_msg = aln_registry::InstantiateMsg { governance_addr: gov.to_string(), allow_missing_ubs: Some(true) };
    let reg_addr = app.instantiate_contract(reg_code, Addr::unchecked("creator"), &reg_instantiate_msg, &[], "REG", None)?;
    let asset = aln_registry::RegisteredAsset { id: "d1".to_string(), source_chain: "kaiyo-1".to_string(), source_denom: "ibc/xxx".to_string(), snapshot_height: 100, merkle_root: root.clone(), ubs_report_hash: Some("h3".to_string()), scaling_profile_id: "malicious_cleanup".to_string(), activation_height: 1000, sanitized_approved: true };
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::RegisterAsset { asset: asset.clone() }, &[])?;
    app.execute_contract(gov.clone(), reg_addr.clone(), &aln_registry::ExecuteMsg::ApproveSanitized { id: "d1".to_string(), ubs_report_hash: "h3".to_string() }, &[])?;

    let bridge_addr = Addr::unchecked("bridge");
    let auet_instantiate_msg = aln_auet::InstantiateMsg { cw20: Cw20InstantiateMsg { name: "AU.ET".to_string(), symbol: "AUET".to_string(), decimals: 6, initial_balances: vec![Cw20Coin { address: bridge_addr.to_string(), amount: Uint128::new(1_000_000) }], mint: None, marketing: None }, snapshot: aln_auet::SnapshotMeta { chain_id: "kaiyo-1".to_string(), height: 100, merkle_root: root.clone() }, allowed_modules: Some(vec![]) };
    let auet_addr = app.instantiate_contract(auet_code, Addr::unchecked("creator"), &auet_instantiate_msg, &[], "AUET", None)?;
    let bridge_instantiate_msg = aln_bridge::InstantiateMsg { auet_contract: auet_addr.to_string(), csp_contract: None, registry_contract: reg_addr.to_string(), governance_addr: gov.to_string() };
    let bridge_addr_inst = app.instantiate_contract(bridge_code, Addr::unchecked("creator"), &bridge_instantiate_msg, &[], "BRIDGE", None)?;

    // set block to before activation height
    app.update_block(|b| b.height = 500);

    let proof_bytes = proofs[0].clone();
    let proof_steps: Vec<aln_bridge::ProofStep> = proof_bytes.iter().map(|(b, is_left)| aln_bridge::ProofStep { sibling: Binary(b.clone()), is_left: *is_left }).collect();
    let claim_msg = aln_bridge::ExecuteMsg::Claim { asset_id: "d1".to_string(), snapshot: s1.clone(), snapshot_hash: h1.clone(), merkle_proof: proof_steps, amount_auet: Uint128::new(10), amount_csp: None };
    let res = app.execute_contract(Addr::unchecked("del_user"), bridge_addr_inst.clone(), &claim_msg, &[]);
    assert!(res.is_err());
    
    // now set height to after activation
    app.update_block(|b| b.height = 2000);
    let res2 = app.execute_contract(Addr::unchecked("del_user"), bridge_addr_inst.clone(), &claim_msg, &[])?;
    assert!(res2.attributes.iter().any(|a| a.key == "action" && a.value == "claim"));
    Ok(())
}
