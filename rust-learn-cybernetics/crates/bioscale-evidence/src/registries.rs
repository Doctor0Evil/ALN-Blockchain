use crate::{EvidenceBundle, EvidenceTag};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainRegistryId {
    Bci,
    Xr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BciEvidenceRegistry {
    pub tags: Vec<EvidenceTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrEvidenceRegistry {
    pub tags: Vec<EvidenceTag>,
}

impl BciEvidenceRegistry {
    pub fn default_bci_registry() -> Self {
        let tag = EvidenceTag {
            id: Uuid::new_v4(),
            label: "bci.eeg.signal_quality".into(),
            domain: "bci".into(),
            version: "v1".into(),
        };
        Self { tags: vec![tag] }
    }

    pub fn new_bundle(&self, source: String, collected_at_unix_ms: u64) -> EvidenceBundle {
        EvidenceBundle {
            bundle_id: Uuid::new_v4(),
            tags: self.tags.clone(),
            source,
            collected_at_unix_ms,
        }
    }
}

impl XrEvidenceRegistry {
    pub fn default_xr_registry() -> Self {
        let tag = EvidenceTag {
            id: Uuid::new_v4(),
            label: "xr.motion.latency".into(),
            domain: "xr".into(),
            version: "v1".into(),
        };
        Self { tags: vec![tag] }
    }
}
