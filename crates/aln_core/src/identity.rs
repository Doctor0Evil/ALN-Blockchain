use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RoleProfile { Operator, Builder, Researcher }

pub fn map_did_to_role(did: &str) -> RoleProfile {
    // Deterministic stub: map based on DID suffix
    if did.contains("operator") { RoleProfile::Operator }
    else if did.contains("builder") { RoleProfile::Builder }
    else { RoleProfile::Researcher }
}
