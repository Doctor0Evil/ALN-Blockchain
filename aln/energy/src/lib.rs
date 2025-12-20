//! ALN energy compression utilities
//!
//! Provides `to_u128_floor` and `compress_asset` as the canonical implementation for
//! mapping external minimal units into AU.ET / CSP fixed‑point internal units.

pub const C_E: f64 = 1e-12;
pub const C_S: f64 = 5e-13;
pub const AE_CAP: u128 = 10_000_000_000_000u128;
pub const CSP_CAP: u128 = 5_000_000_000_000u128;

#[inline]
pub fn to_u128_floor(x: f64) -> u128 {
    if x.is_nan() || x <= 0.0 {
        0u128
    } else {
        let f = x.floor();
        if !f.is_finite() || f <= 0.0 { 0u128 } else { f as u128 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnergyVector {
    pub au_et: u128,
    pub csp: u128,
}

/// Compress an external asset amount (minimal units) into an `EnergyVector`.
/// The result saturates at the configured global caps.
pub fn compress_asset(a_src_min_unit: u128) -> EnergyVector {
    // Convert to f64 for fixed-point multiplication using canonical constants.
    let src = a_src_min_unit as f64;
    let au = to_u128_floor(src * C_E);
    let cs = to_u128_floor(src * C_S);

    EnergyVector {
        au_et: au.min(AE_CAP),
        csp: cs.min(CSP_CAP),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn known_values() {
        // 1e12 minimal units -> 1 AU.ET
        let a = 1_000_000_000_000u128; // 1e12
        let e = compress_asset(a);
        assert_eq!(e.au_et, 1);
        // 0 maps to zero
        assert_eq!(compress_asset(0u128), EnergyVector { au_et: 0, csp: 0 });
    }

    proptest! {
        #[test]
        fn monotonicity_and_caps(a in any::<u64>(), b in any::<u64>()) {
            let a = a as u128;
            let b = b as u128;
            prop_assume!(a <= b);
            let ea = compress_asset(a);
            let eb = compress_asset(b);
            prop_assert!(ea.au_et <= eb.au_et, "au monotonic");
            prop_assert!(ea.csp <= eb.csp, "csp monotonic");
            prop_assert!(ea.au_et <= AE_CAP);
            prop_assert!(ea.csp <= CSP_CAP);
            prop_assert!(eb.au_et <= AE_CAP);
            prop_assert!(eb.csp <= CSP_CAP);
        }
    }
}
