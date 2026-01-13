use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BciSample {
    pub channel: u8,
    pub value_uv: f32,
}
