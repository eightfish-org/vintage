use crate::db::blocks::BlockInDb;
use crate::db::{blocks, last_block_height, txs, DbTable};
use redb::ReadTransaction;
use vintage_msg::{BlockHeight, Tx, TxId};

pub(crate) enum DbRead {}

impl DbRead {
    pub fn check_tx_not_exists(transaction: &ReadTransaction, id: TxId) -> anyhow::Result<()> {
        let table_txs = txs::open_table(transaction)?;
        DbTable::check_tx_not_exists(&table_txs, id)
    }

    pub fn check_all_txs_not_exist_in_db(
        transaction: &ReadTransaction,
        txs: &Vec<Tx>,
    ) -> anyhow::Result<()> {
        let table_txs = txs::open_table(transaction)?;
        DbTable::check_all_txs_not_exist_in_db(&table_txs, txs)
    }

    pub fn get_last_block_height(transaction: &ReadTransaction) -> anyhow::Result<BlockHeight> {
        let table_lbh = last_block_height::open_table(&transaction)?;
        DbTable::get_last_block_height(&table_lbh)
    }

    pub fn get_block(
        transaction: &ReadTransaction,
        block_height: BlockHeight,
    ) -> anyhow::Result<Option<BlockInDb>> {
        let table_blocks = blocks::open_table(&transaction)?;
        DbTable::get_block(&table_blocks, block_height)
    }
}
