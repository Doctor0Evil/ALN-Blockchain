use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllocationRequest { pub budget: u128, pub class_costs: Vec<u128>, pub class_caps: Vec<u128>, pub weights: Vec<f64> }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllocationResult { pub allocations: Vec<u128> }

/// Simple greedy allocator maximizing weighted sum under caps and budget
pub fn allocate(req: AllocationRequest) -> AllocationResult {
    let mut remaining = req.budget as f64;
    let mut allocations: Vec<u128> = vec![0; req.class_costs.len()];
    // compute utility per unit = weight / cost
    let mut per_unit: Vec<(usize, f64)> = req.weights.iter().enumerate().map(|(i,w)| (i, *w / (req.class_costs[i] as f64))).collect();
    per_unit.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());
    for (idx, _v) in per_unit {
        let cost = req.class_costs[idx] as f64;
        let cap = req.class_caps[idx] as f64;
        if cost <= 0.0 { continue; }
        let max_units = (remaining / cost).floor().min(cap);
        let assign = max_units as u128;
        allocations[idx] = assign;
        remaining -= (assign as f64) * cost;
    }
    AllocationResult { allocations }
}
