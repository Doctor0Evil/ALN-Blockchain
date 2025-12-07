use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// compress raw minimal unit balance B (u128) into ALN minimal units
/// d_src: source decimals (e.g. 6)
/// d_aln: ALN decimals (e.g. 6)
/// c: compression factor (e.g. 1e-12)
pub fn compress_balance(b: u128, d_src: u8, d_aln: u8, c: f64) -> u128 {
    // A_src = b / 10^d_src
    let denom_src = Decimal::from(10u64.pow(d_src as u32));
    let b_dec = Decimal::from(b);
    let a_src = b_dec / denom_src;
    let a_aln = a_src * Decimal::from_f64(c).unwrap();
    let denom_aln = Decimal::from(10u64.pow(d_aln as u32));
    let b_aln_dec = (a_aln * denom_aln).floor();
    let b_aln = b_aln_dec.to_u128().unwrap_or(0u128);
    b_aln
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_balance() {
        // Assume: B = 1_010_000 * 10^6 minimal units (so a human readable 1,010,000)
        let b: u128 = 1_010_000u128 * 1_000_000u128; // 1,010,000 * 1e6
        let res = compress_balance(b, 6, 6, 1e-12);
        assert_eq!(res, 1u128);

        let res2 = compress_balance(b, 6, 6, 5e-13);
        assert_eq!(res2, 0u128);
    }
}
