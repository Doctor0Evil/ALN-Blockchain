use criterion::{criterion_group, criterion_main, Criterion};
use sha2::{Sha256, Digest};

fn build_leaf(i: u64) -> [u8;32] {
    let mut hasher = Sha256::new();
    hasher.update(&i.to_be_bytes());
    let res = hasher.finalize();
    let mut arr = [0u8;32]; arr.copy_from_slice(&res); arr
}

fn verify_proof(cur: &[u8;32], proof: &Vec<([u8;32], bool)>, root: &[u8;32]) -> bool {
    let mut cur = *cur;
    for (sib, is_left) in proof.iter() {
        let mut h = Sha256::new();
        if *is_left { h.update(sib); h.update(&cur); } else { h.update(&cur); h.update(sib); }
        let r = h.finalize(); cur.copy_from_slice(&r);
    }
    &cur == root
}

fn bench_merkle(c: &mut Criterion) {
    // generate 64 leaf tree
    let mut leaves: Vec<[u8;32]> = (0u64..64).map(|i| build_leaf(i)).collect();
    // build a simple tree
    let mut levels: Vec<Vec<[u8;32]>> = vec![leaves.clone()];
    while levels.last().unwrap().len() > 1 {
        let prev = levels.last().unwrap().clone();
        let mut next = vec![];
        for i in (0..prev.len()).step_by(2) {
            let left = prev[i];
            let right = if i+1<prev.len() { prev[i+1] } else { prev[i] };
            let mut h = Sha256::new(); h.update(&left); h.update(&right); let res = h.finalize(); let mut arr=[0u8;32]; arr.copy_from_slice(&res); next.push(arr);
        }
        levels.push(next);
    }
    let root = levels.last().unwrap()[0];
    // build proof for leaf index 5
    let idx = 5usize;
    let mut proof: Vec<([u8;32], bool)> = vec![];
    let mut index = idx;
    for lvl in 0..(levels.len()-1) {
        let level_nodes = &levels[lvl];
        let pair_index = if index % 2 == 0 { index + 1 } else { index - 1 };
        let sibling = if pair_index < level_nodes.len() { level_nodes[pair_index] } else { level_nodes[index] };
        proof.push((sibling, pair_index < index));
        index = index / 2;
    }
    let leaf = leaves[idx];
    c.bench_function("merkle_verify_64", |b| { b.iter(|| { assert!(verify_proof(&leaf, &proof, &root)); }); });
}

criterion_group!(benches, bench_merkle);
criterion_main!(benches);
