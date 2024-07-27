use serde::{Deserialize, Serialize};
use vintage_utils::WithId;

pub type ActId = u128; // uuid，用于检查重复Act消息
pub type ActContent = Vec<u8>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Act {
    pub id: ActId,
    pub content: ActContent,
}

impl Act {
    pub fn new_id() -> ActId {
        uuid::Uuid::new_v4().as_u128()
    }
}

impl WithId for Act {
    type Id = ActId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
