use aln_trader_pod::{allocate, AllocationRequest};

#[test]
fn test_simple_allocation() {
    let req = AllocationRequest { budget: 100, class_costs: vec![10, 20], class_caps: vec![10, 10], weights: vec![1.0, 2.0] };
    let res = allocate(req);
    // second class higher weight per cost, allocate there
    assert!(res.allocations[1] > 0);
}
