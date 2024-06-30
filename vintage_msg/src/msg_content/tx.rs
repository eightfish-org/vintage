use serde::{Deserialize, Serialize};

pub type TxId = u128; // uuid，用于检查重复交易消息
pub type TxContent = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tx {
    pub id: TxId,
    pub content: TxContent,
}
