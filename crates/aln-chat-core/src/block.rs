use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use realityos_ids::{HostDid, BostromAddress};
use aln_particles::ALNComplianceParticle;
use bioscale_evidence::EvidenceBundle;
use bioscale_upgrade_store::UpgradeDescriptor;

/// Minimal, chainable ALN chat block for BCI / neuromotor domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnChatBlock {
    // (1) Session identity & environment
    pub host_did: HostDid,
    pub bostrom: BostromAddress,
    pub env_label: String,          // e.g. "phoenix-lab-bci-main"
    pub domain: String,             // e.g. "bci", "neuromorphic"

    // (2) Requested action / syntax
    pub intent_kind: ChatIntentKind,
    pub upgrade: Option<UpgradeDescriptor>, // Some for upgrade/evolution intents
    pub raw_grammar: String,        // Original SESSION|INTENT|SAFETY|EVIDENCE|TERMINAL text

    // (3) Enforced ALN clauses & envelopes
    pub aln_compliance: ALNComplianceParticle,
    pub evidence: EvidenceBundle,   // MUST carry exactly 10 registered tags
    pub created_at: OffsetDateTime,

    // (4) Hex‑stamped audit header (internal ALN chain)
    pub block_hash_hex: String,
    pub parent_hash_hex: Option<String>,
}

/// High-level intent categories aligned with the chat grammar.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatIntentKind {
    BciUpgrade,         // upgrade.bci.*
    NeuromorphicUpgrade,
    SmartCityRoute,
    GovernanceChange,
    TelemetryReview,
}
