//! Minimal non-minting energy ledger crate

use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::constants::{AE_CAP, CSP_CAP};

pub mod constants {
    pub const AE_CAP: u128 = 10_000_000_000_000u128;
    pub const CSP_CAP: u128 = 5_000_000_000_000u128;
}

#[derive(Clone, Debug)]
pub struct EnergyEvent {
    pub seq: u64,
    pub account: [u8; 32],
    pub delta_au: i128,
    pub delta_csp: i128,
    pub prev_hash: [u8; 32],
    pub self_hash: [u8; 32],
}

#[derive(Default)]
pub struct Ledger {
    pub balances_au: HashMap<[u8; 32], u128>,
    pub balances_csp: HashMap<[u8; 32], u128>,
    pub last_seq: u64,
    pub last_hash: [u8; 32],
    pub total_au: u128,
    pub total_csp: u128,
}

// UBSecurity + sealed refactor modules
pub mod ub_security;
pub mod sealed_refactor;

impl Ledger {
    pub fn hash_event(e: &EnergyEvent) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&e.seq.to_be_bytes());
        hasher.update(&e.account);
        hasher.update(&e.delta_au.to_be_bytes());
        hasher.update(&e.delta_csp.to_be_bytes());
        hasher.update(&e.prev_hash);
        let out = hasher.finalize();
        out.into()
    }

    pub fn apply(&mut self, mut e: EnergyEvent) -> Result<(), &'static str> {
        if e.seq != self.last_seq + 1 {
            return Err("seq_mismatch");
        }
        if e.prev_hash != self.last_hash {
            return Err("prev_hash_mismatch");
        }
        e.self_hash = Self::hash_event(&e);

        let bal_au = *self.balances_au.get(&e.account).unwrap_or(&0);
        let bal_csp = *self.balances_csp.get(&e.account).unwrap_or(&0);

        let new_au = bal_au as i128 + e.delta_au;
        let new_csp = bal_csp as i128 + e.delta_csp;
        if new_au < 0 || new_csp < 0 {
            return Err("negative_balance");
        }

        let total_au_i = self.total_au as i128 + e.delta_au;
        let total_csp_i = self.total_csp as i128 + e.delta_csp;
        if total_au_i < 0 || total_csp_i < 0 {
            return Err("negative_global");
        }

        let total_au_u = total_au_i as u128;
        let total_csp_u = total_csp_i as u128;
        if total_au_u > AE_CAP || total_csp_u > CSP_CAP {
            return Err("global_cap_exceeded");
        }

        self.balances_au.insert(e.account, new_au as u128);
        self.balances_csp.insert(e.account, new_csp as u128);
        self.total_au = total_au_u;
        self.total_csp = total_csp_u;
        self.last_seq = e.seq;
        self.last_hash = e.self_hash;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_account(seed: u8) -> [u8; 32] {
        let mut a = [0u8; 32];
        a[0] = seed;
        a
    }

    #[test]
    fn apply_valid_event_increases_balances() {
        let mut ledger = Ledger::default();
        let acct = sample_account(1);
        let ev = EnergyEvent {
            seq: ledger.last_seq + 1,
            account: acct,
            delta_au: 100i128,
            delta_csp: 50i128,
            prev_hash: ledger.last_hash,
            self_hash: [0u8; 32],
        };
        ledger.apply(ev).unwrap();
        assert_eq!(*ledger.balances_au.get(&acct).unwrap(), 100u128);
        assert_eq!(ledger.total_au, 100u128);
    }

    #[test]
    fn seq_mismatch_detected() {
        let mut ledger = Ledger::default();
        let acct = sample_account(2);
        let mut ev = EnergyEvent {
            seq: ledger.last_seq + 2, // wrong
            account: acct,
            delta_au: 1,
            delta_csp: 0,
            prev_hash: ledger.last_hash,
            self_hash: [0u8; 32],
        };
        assert_eq!(ledger.apply(ev).unwrap_err(), "seq_mismatch");
    }

    #[test]
    fn prev_hash_mismatch_detected() {
        let mut ledger = Ledger::default();
        let acct = sample_account(3);
        let mut ev = EnergyEvent {
            seq: ledger.last_seq + 1,
            account: acct,
            delta_au: 1,
            delta_csp: 0,
            prev_hash: [1u8; 32], // wrong
            self_hash: [0u8; 32],
        };
        assert_eq!(ledger.apply(ev).unwrap_err(), "prev_hash_mismatch");
    }

    #[test]
    fn negative_balance_prevented() {
        let mut ledger = Ledger::default();
        let acct = sample_account(4);
        let ev = EnergyEvent {
            seq: ledger.last_seq + 1,
            account: acct,
            delta_au: -1i128,
            delta_csp: 0i128,
            prev_hash: ledger.last_hash,
            self_hash: [0u8; 32],
        };
        assert_eq!(ledger.apply(ev).unwrap_err(), "negative_balance");
    }

    #[test]
    fn global_cap_exceeded() {
        let mut ledger = Ledger::default();
        // create a big incoming event that would exceed AE_CAP
        let acct = sample_account(5);
        let ev = EnergyEvent {
            seq: ledger.last_seq + 1,
            account: acct,
            delta_au: (AE_CAP as i128) + 1i128,
            delta_csp: 0i128,
            prev_hash: ledger.last_hash,
            self_hash: [0u8; 32],
        };
        assert_eq!(ledger.apply(ev).unwrap_err(), "global_cap_exceeded");
    }
}
