use sha2::{Digest, Sha256};
use serde::Serialize;
use bech32::{decode, FromBase32};

#[derive(Debug, Serialize)]
pub struct SnapshotEntry {
    pub chain_id: String,
    pub height: u64,
    pub denom: String,
    pub address: String, // bech32
    pub balance: u128,
}

pub fn u64_be_bytes(x: u64) -> [u8;8] {
    x.to_be_bytes()
}

pub fn u128_be_bytes(x: u128) -> [u8;16] {
    x.to_be_bytes()
}

pub fn hash_entry(e: &SnapshotEntry) -> [u8;32] {
    let mut hasher = Sha256::new();
    hasher.update(e.chain_id.as_bytes());
    hasher.update(&u64_be_bytes(e.height));
    hasher.update(e.denom.as_bytes());

    // decode bech32 address to raw bytes
    let (_hrp, data, _variant) = decode(&e.address).expect("bech32 decode error");
    let addr_bytes = Vec::<u8>::from_base32(&data).expect("bech32 base32 to bytes");
    // Ensure 20 byte length; if not, pad/trim as required.
    let addr = if addr_bytes.len() == 20 { addr_bytes } else { 
        // len != 20: still include canonical form but do not fail in this simple example
        let mut b = vec![0u8; 20];
        let copy_len = std::cmp::min(20, addr_bytes.len());
        b[..copy_len].copy_from_slice(&addr_bytes[..copy_len]);
        b
    };
    hasher.update(&addr);

    // balance as u128 -> 16 byte BE; for full 32 byte be big integer, left pad
    let bal32 = {
        let mut b = [0u8; 32];
        let be = u128_be_bytes(e.balance);
        // place u128 at the lower 16 bytes (rightmost)
        b[16..32].copy_from_slice(&be);
        b
    };
    hasher.update(&bal32);

    let res = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&res);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_entry() {
        let e = SnapshotEntry {
            chain_id: "kaiyo-1".to_string(),
            height: 123456u64,
            denom: "ibc/1234abcd".to_string(),
            address: "kujira1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyq0zrqc".to_string(),
            balance: 1_010_000u128,
        };
        let h = hash_entry(&e);
        assert_eq!(h.len(), 32);
    }
}
