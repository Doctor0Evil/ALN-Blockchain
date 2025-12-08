# ALN Energy Market Spec — Code Mapping

This document maps high-level spec sections to code modules and artifacts in the repository.

1) Bridge architecture
  - Contracts: `contracts/bridge` (core claim logic, merkle verification), `contracts/aln_registry` (asset registry & UBS hash), `contracts/auet` & `contracts/aln20_csp` (non-mintable energy tokens).
  - Off-chain: `crates/aln_ubs` (UBS analyzer & report writer).

2) UBS pipeline
  - Tools: `tools/ubs_analyzer` (basic analysis for contracts), `crates/aln_ubs` (deterministic pipeline and report generation).
  - Contract triggers: `contracts/aln_registry` holds `ubs_report_hash`; `contracts/bridge` checks presence before allowing claims.

3) Token factory
  - `crates/aln_core::token_factory` — create token classes, enforce fees and non-transferability for energy classes.

4) Indexer & observability
  - `crates/aln_indexer` — block ingestion, reorg detection, replay, retention compaction, metrics.
  - Metrics: `aln_bridge_events_total`, `aln_energy_toxic_total`, `aln_energy_clean_total` included in `/metrics`.

5) Trader-Pod optimization & identity
  - `crates/aln_trader_pod` — allocation optimizer.
  - `crates/aln_core::identity` — DID role mapping stub used by the factory & high-level checks.
