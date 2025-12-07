use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};
use hex;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProofStep {
    pub sibling: String, // hex
    pub is_left: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Proof { pub proof: Vec<ProofStep> }

/// Build a Merkle tree from a vector of leaves (32-byte arrays), returning root and per-index proofs.
pub fn build_merkle_and_proofs(leaves: &Vec<[u8;32]>) -> (String, Vec<Proof>) {
    if leaves.is_empty() { return (String::from(""), vec![]); }
    let mut level: Vec<[u8;32]> = leaves.clone();
    let mut tree_levels: Vec<Vec<[u8;32]>> = vec![level.clone()];
    while level.len() > 1 {
        let mut next: Vec<[u8;32]> = vec![];
        for i in (0..level.len()).step_by(2) {
            let left = level[i];
            let right = if i + 1 < level.len() { level[i+1] } else { level[i] };
            let mut h = Sha256::new();
            h.update(&left);
            h.update(&right);
            let mut out = [0u8; 32];
            out.copy_from_slice(&h.finalize());
            next.push(out);
        }
        level = next.clone();
        tree_levels.push(level.clone());
    }
    let root = tree_levels.last().unwrap()[0];

    // Build proofs for each original leaf index
    let mut proofs: Vec<Proof> = Vec::new();
    let leaf_count = leaves.len();
    for idx in 0..leaf_count {
        let mut proof_steps: Vec<ProofStep> = Vec::new();
        let mut index = idx;
        for lvl in 0..(tree_levels.len()-1) {
            let level_nodes = &tree_levels[lvl];
            let pair_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling = if pair_index < level_nodes.len() { level_nodes[pair_index] } else { level_nodes[index] };
            let is_left = pair_index < index; // if sibling idx < index, sibling is on left
            proof_steps.push(ProofStep { sibling: format!("0x{}", hex::encode(sibling)), is_left });
            index = index / 2;
        }
        proofs.push(Proof { proof: proof_steps });
    }

    (format!("0x{}", hex::encode(root)), proofs)
}

/// Verify a proof given leaf and proof steps.
pub fn verify_merkle_proof(leaf: &[u8;32], proof: &Proof, root_hex: &str) -> bool {
    let mut cur = *leaf;
    for step in proof.proof.iter() {
        let sib_bytes = match hex::decode(step.sibling.trim_start_matches("0x")) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let mut sib_arr = [0u8;32];
        if sib_bytes.len() != 32 { return false; }
        sib_arr.copy_from_slice(&sib_bytes);
        let mut h = Sha256::new();
        if step.is_left {
            h.update(&sib_arr);
            h.update(&cur);
        } else {
            h.update(&cur);
            h.update(&sib_arr);
        }
        let res = h.finalize();
        cur.copy_from_slice(&res);
    }
    let root_bytes = match hex::decode(root_hex.trim_start_matches("0x")) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let mut root_arr = [0u8;32];
    if root_bytes.len() != 32 { return false; }
    root_arr.copy_from_slice(&root_bytes);
    cur == root_arr
}
