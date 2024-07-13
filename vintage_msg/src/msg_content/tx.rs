use serde::{Deserialize, Serialize};
use vintage_utils::WithId;

pub type TxId = u128; // uuid，用于检查重复交易消息
pub type TxContent = Vec<u8>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Tx {
    pub id: TxId,
    pub content: TxContent,
}

impl Tx {
    pub fn new_id() -> TxId {
        uuid::Uuid::new_v4().as_u128()
    }
}

impl WithId for Tx {
    type Id = TxId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
