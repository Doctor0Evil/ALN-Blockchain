use aln_ubs::DefaultUBS;
use std::fs;

#[test]
fn test_ubs_sanitize_deterministic() {
    let u = DefaultUBS;
    let wasm = b"module";
    let r = u.sanitize("k1", "ibc/xxx", wasm).unwrap();
    assert!(r.report_hash.starts_with("0x"));
    // Re-run and ensure same hash
    let r2 = u.sanitize("k1", "ibc/xxx", wasm).unwrap();
    assert_eq!(r.report_hash, r2.report_hash);
}
