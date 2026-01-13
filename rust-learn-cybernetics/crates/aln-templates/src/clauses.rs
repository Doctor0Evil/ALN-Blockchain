use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlnClauseId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidClause {
    pub id: AlnClauseId,
    pub description: String,
}
