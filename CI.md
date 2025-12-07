# CI Requirements and Merge Gates

This repository uses GitHub Actions to run a set of validation jobs. To ensure safe merges and artifact quality, we recommend setting branch protection rules (in GitHub Settings) to require the following jobs to pass before merging into `main` (or other protected branches):

- `ubs-policy` — UBS analyzer must produce reports and hashes
- `did-admin-check` — Verify operator DID identity matches controller DID
- `rust-integration` — Run cw-multi-test integration tests
- `wasm-artifacts` — Build wasm artifacts, optimize with wasm-opt, and produce provenance
- `indexer-tests` — Run indexer crate tests and migrations against a Postgres service (if change touches indexer code)

How to configure this in GitHub (admin required):
1. Go to `Settings` > `Branches` > `Branch protection rules`.
2. Create a rule for `main` or `develop`.
3. Under "Require status checks to pass before merging", add the jobs above as required checks.

Note: Branch protection rules cannot be enabled automatically from a workflow file and require repository admin access to configure.

`SETUP.md` references these jobs and how to run locally.
