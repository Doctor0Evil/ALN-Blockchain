use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnChatBlockHeader {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnChatBlock {
    pub header: AlnChatBlockHeader,
    pub payload: String,
}
