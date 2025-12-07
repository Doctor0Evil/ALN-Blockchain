use sha2::{Sha256, Digest};

pub fn compute_ibc_hash(path: &str, base_denom: &str) -> String {
    let input = format!("{}/{}", path, base_denom);
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    format!("ibc/{}", hex::encode(h.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_ibc_hash() {
        let p = "transfer/channel-1";
        let base = "uatom";
        let s = compute_ibc_hash(p, base);
        assert!(s.starts_with("ibc/"));
    }
}
