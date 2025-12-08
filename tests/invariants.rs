use std::fs;
use std::path::Path;

#[test]
fn no_reverse_bridge_ops_in_contracts() {
    let contracts_dir = Path::new("contracts");
    let forbidden = ["send_packet", "IbcMsg", "ibc_transfer", "Withdraw", "Unlock", "Redeem", "release", "unlock", "redeem", "IBC"];
    for entry in fs::read_dir(contracts_dir).unwrap() {
        let e = entry.unwrap();
        let p = e.path();
        if p.is_dir() {
            let lib_rs = p.join("src").join("lib.rs");
            if lib_rs.exists() {
                let content = fs::read_to_string(&lib_rs).unwrap();
                for f in forbidden.iter() {
                    assert!(!content.contains(f), "Found forbidden pattern '{}' in {:?}", f, lib_rs);
                }
            }
        }
    }
}
