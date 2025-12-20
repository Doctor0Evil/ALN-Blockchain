# Developer setup (Rust & testing)

This project contains Rust crates and assumes a working Rust toolchain for local development and CI. Quick setup:

1. Install Rust via rustup: https://rustup.rs
   - Use default toolchain (stable) or `rustup default stable`
2. Verify `cargo` is available:
   - `cargo --version`
3. Run tests for the `aln-energy` crate:
   - `cd aln/energy`
   - `cargo test`

CI note: A GitHub Actions workflow `rust-tests.yml` runs `cargo test` for relevant crates on push/PR to ensure test coverage across platforms.
