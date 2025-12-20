# Developer setup (Rust & testing)

> Reminder: this repository now includes a GitHub Actions workflow to run Rust crate tests automatically. The instructions below explain how to run tests locally and how CI will validate them even if `cargo` is not installed on a developer machine.

This project contains Rust crates and assumes a working Rust toolchain for local development and CI. Quick setup:

## Install Rust (rustup)

Linux / macOS:

```bash
# Install via rustup (interactive installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow prompts or run non-interactive: sh -s -- -y
```

Windows (PowerShell):

```powershell
# Download and run the installer (uses rustup)
irm https://sh.rustup.rs -UseBasicParsing | iex
# Or install from https://rustup.rs
```

## Verify installation

```bash
rustc --version
cargo --version
```

## Run tests for the crates (from repo root)

```bash
# Run tests for aln-energy crate
cargo test --manifest-path aln/energy/Cargo.toml

# Run tests for aln-energy-ledger crate
cargo test --manifest-path aln/aln-energy-ledger/Cargo.toml
```

Notes:

- The CI workflow (`.github/workflows/rust-tests.yml`) runs these tests automatically on GitHub Actions; you do not need to have `cargo` locally to get CI coverage.
- Use `rustup` to manage toolchain versions; the CI uses the stable toolchain by default.

---

