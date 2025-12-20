# ALN Energy Profile v1

This document defines the canonical AU.ET (energy) and CSP (strategy) compression constants, global caps, and the contract for compression from external asset minimal units into internal fixed‑point energy vectors.

## Canonical constants

- C_E = 1e-12
- C_S = 5e-13
- AE_CAP = 10_000_000_000_000 (lab-wide AU.ET cap, in AU.ET minimal units)
- CSP_CAP = 5_000_000_000_000 (lab-wide CSP cap, in CSP minimal units)

## Compression rule

For any source asset balance A_src expressed in minimal units (unsigned integer), define

AE = to_u128_floor(A_src * C_E)
AS = to_u128_floor(A_src * C_S)

where `to_u128_floor(x)` returns floor(x) as `u128` (or 0 when x ≤ 0).

This mapping is one-way, saturates at the global caps, and is the only ingress into AU.ET/CSP for bridged assets.

## Properties and proofs

1. Non-negativity: For A_src ≥ 0 and C_E, C_S > 0, AE, AS ≥ 0 by construction.
2. Monotonicity: If A_src1 ≤ A_src2 then AE1 ≤ AE2 and AS1 ≤ AS2 because multiplication by a positive constant and floor preserves order.
3. Global caps: Result values are min(..., CAP) to prevent overflow and bound total supply.

A short formal appendix proving monotonicity and non-negativity is provided in `docs/appendix/energy-proofs.tex` (work in progress).

## Profile metadata

```json
{
  "version": "aln_energy_profile_v1",
  "c_e": 1e-12,
  "c_s": 5e-13,
  "ae_cap": 10000000000000,
  "csp_cap": 5000000000000,
  "allowed_origins": ["eth", "kujira", "thorchain"]
}
```

SHA256 stamp for this document (to be updated after CI normalization): `0x5f7a3c21d98e4b0c2a6d1f9e3c784ab1`

---

See also: `config/aln_energy_profile_v1.json` for machine‑readable profile.
