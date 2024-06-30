use vintage_msg::{TxContent, TxId};
use vintage_utils::define_redb_table;

define_redb_table!(TxId, TxContent, "txs");
