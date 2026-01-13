use crate::bci_adapter::BciSample;
use aln_core::ALNComplianceParticle;
use bioscale_upgrade_store::{evaluate_upgrade, BrainSpecs, HostBudget, UpgradeDescriptor, UpgradeState};
use uuid::Uuid;

pub fn route_bci_sample(
    sample: BciSample,
    descriptor: &UpgradeDescriptor,
    budget: &HostBudget,
    brain: &BrainSpecs,
) -> (UpgradeState, Option<ALNComplianceParticle>) {
    let state = evaluate_upgrade(descriptor, budget, brain).unwrap_or(UpgradeState::Rejected);
    let particle = if matches!(state, UpgradeState::Approved) {
        Some(ALNComplianceParticle {
            id: Uuid::new_v4(),
            clause_id: "ALN:BCI:SAFETY:1".into(),
            evidence_envelope_id: Uuid::new_v4(), // placeholder
        })
    } else {
        None
    };
    let _ = sample;
    (state, particle)
}
