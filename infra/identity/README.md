# DID config (demo)

This directory contains a small, public DID configuration used by CI jobs, deployment scripts, and backend services as a public identifier (no secrets).

Usage notes:

- `did-config.json` contains `primary_did` which may be read by CI or infra scripts to display or log a public identity for automation flows.
- **Private keys or tokens must never be committed to this repository.** Keep private material in local developer keyrings, local environment variables, or an external secret manager/vault.
- The DID here is a public identifier only; do not assume possession of any private key from this file.

Future scripts under `infra/` can read `did-config.json` to obtain `primary_did` for non-sensitive operations such as logging, verification, or public attestations.
