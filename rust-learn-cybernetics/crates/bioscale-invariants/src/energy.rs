use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EnergyInvariantError {
    #[error("energy budget exceeded: {actual} J > limit {limit} J")]
    Exceeded { actual: f32, limit: f32 },
}

pub fn never_exceed_energy_joules(actual: f32, limit: f32) -> Result<(), EnergyInvariantError> {
    if actual > limit {
        Err(EnergyInvariantError::Exceeded { actual, limit })
    } else {
        Ok(())
    }
}
