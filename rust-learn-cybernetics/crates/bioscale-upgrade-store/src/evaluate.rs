use crate::types::{BrainSpecs, HostBudget, UpgradeDescriptor, UpgradeState};
use bioscale_invariants::{always_within_latency_ms, never_exceed_energy_joules};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvaluationError {
    #[error("energy budget violation")]
    Energy,
    #[error("latency violation")]
    Latency,
}

pub fn evaluate_upgrade(
    upgrade: &UpgradeDescriptor,
    budget: &HostBudget,
    _brain: &BrainSpecs,
) -> Result<UpgradeState, EvaluationError> {
    never_exceed_energy_joules(upgrade.required_energy_joules, budget.max_energy_joules)
        .map_err(|_| EvaluationError::Energy)?;
    always_within_latency_ms(upgrade.worst_case_latency_ms, budget.max_latency_ms)
        .map_err(|_| EvaluationError::Latency)?;
    Ok(UpgradeState::Approved)
}

#[cfg(feature = "kani")]
mod proofs {
    use super::*;
    use kani::any; // harness example [web:1][web:2]

    #[kani::proof]
    fn energy_and_latency_within_budget_leads_to_approved() {
        let desc = UpgradeDescriptor {
            id: any(),
            name: String::new(),
            required_energy_joules: 1.0,
            worst_case_latency_ms: 1,
        };
        let budget = HostBudget {
            max_energy_joules: 2.0,
            max_latency_ms: 10,
        };
        let brain = BrainSpecs {
            subject_id: String::new(),
            region: String::new(),
        };
        let state = evaluate_upgrade(&desc, &budget, &brain).unwrap();
        assert_eq!(state, UpgradeState::Approved);
    }
}
