use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaDataConfig {
    pub db_path: String,
}
