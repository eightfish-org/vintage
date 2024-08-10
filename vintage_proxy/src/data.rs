use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InputOutputObject {
    pub action: String,
    pub model: String,
    pub data: Vec<u8>,
    pub ext: Vec<u8>,
}
