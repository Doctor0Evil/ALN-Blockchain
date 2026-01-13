use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Minimal tag describing a single bioscale evidence sample.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceTag {
    pub id: Uuid,
    pub label: String,
    pub domain: String,
    pub version: String,
}

/// Bundle of evidence tags with opaque payload metadata.
///
/// The payload is intentionally abstract; leaf crates (BCI/XR) can wrap this
/// with domain-specific structures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub bundle_id: Uuid,
    pub tags: Vec<EvidenceTag>,
    pub source: String,
    pub collected_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EnvelopeKind {
    BioscaleUpgradePrecheck,
    RuntimeGuardDecision,
}

/// Envelope is what flows into ALN, upgrade store, and guards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEnvelope {
    pub envelope_id: Uuid,
    pub kind: EnvelopeKind,
    pub bundle: EvidenceBundle,
    pub created_unix_ms: u64,
    pub signer_did: Option<String>,
}

/// Convenience constant namespace for biophysics evidence labels.
pub mod BCIBIOPHYS_EVIDENCE {
    pub const EEG_SIGNAL_QUALITY: &str = "bci.eeg.signal_quality";
    pub const STIM_ENERGY_BUDGET: &str = "bci.stim.energy_budget";
    pub const XR_MOTION_LATENCY: &str = "xr.motion.latency";
}

/// Create a basic envelope from a bundle.
///
/// Higher layers can enrich this with DID signatures and ALN particles.
pub fn bundle_to_envelope(bundle: EvidenceBundle, kind: EnvelopeKind) -> EvidenceEnvelope {
    EvidenceEnvelope {
        envelope_id: Uuid::new_v4(),
        kind,
        bundle,
        created_unix_ms: current_unix_ms(),
        signer_did: None,
    }
}

fn current_unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
