# Energy Market – Production Readiness Checklist

This checklist enumerates go-live criteria for the ALN energy market and associated bridge components.

1) Invariants & Tests
  - [x] All invariants tests pass (`tests/invariants.rs`) ensuring no reverse-bridge ops exist.
  - [x] UBS compliance test coverage present; `aln_ubs` tests pass.
  - [x] Replay & indexer tests include reorg & replay scenarios (branch, idempotency).

2) CI Coverage
  - [x] `rust-integration`, `indexer-tests`, and `cem-metrics-smoke` complete in CI.
  - [x] UBS analyzer runs for contracts, producing reports.

3) Governance & Keys
  - [x] Registry governance proofs configured (multisig/DAO via `GOVERNANCE` in contracts).

4) Observability
  - [x] JSON logs enabled for all services.
  - [x] Prometheus `/metrics` endpoints exposed and scrapped in CI.

5) Runbooks & Operator Scripts
  - [x] `docs/RUNBOOK_indexer.md` updated with `replay-from` command and observability toggles.
  - [x] `scripts/run_energy_market_smoke.sh` script added to run local smoke tests.
