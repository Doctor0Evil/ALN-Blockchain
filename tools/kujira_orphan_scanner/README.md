# Kujira Orphan Scanner

Simple Rust tool to scan Kujira IBC denom traces and detect potential orphan/undocumented tokens (first-pass heuristic).

Usage:
- `cargo run --manifest-path tools/kujira_orphan_scanner/Cargo.toml` (future arg parsing to come)
- Output: `artifacts/orphan_ibc.json` listing candidate orphans.

Add config/known_bases.json to customize the allowlist.
