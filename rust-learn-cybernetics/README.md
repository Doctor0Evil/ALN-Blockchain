# rust-learn-cybernetics

EvidenceBundle → envelopes → ALN/Kani spine for bioscale upgrades, with BCI/XR specifics isolated in leaf crates.

This workspace exposes:

- **Evidence** primitives and registries for bioscale domains.
- Invariant helpers for energy, latency, and rollback safety.
- Upgrade descriptor store plus macros that derive envelopes from evidence.
- ALN core particles, clauses, and templates for neurorights-aware compliance.
- Chat-front ALN blocks and header macros for on-chain style logging.
- BCI/XR routing and nanoswarm guards that consume envelopes.
- Metrics, daily evolution loop, and IEC 62304 / ISO 14971 mapping bridge.

See `docs/whitepaper/evidence_envelope_kani_aln.md` for a narrative walkthrough.
