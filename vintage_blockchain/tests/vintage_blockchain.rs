use redb::TableDefinition;
use vintage_blockchain::{Block, BlockChain};

const TABLE_BLOCKS_FORTEST: TableDefinition<&str, &str> = TableDefinition::new("blocks_fortest");

#[test]
fn test_block_hash() {
    let block1 = Block::new(10, "aaabbbcccdddeeefff".to_string(), vec![]);
    let block2 = Block::new(10, "aaabbbcccdddeeefff".to_string(), vec![]);
    assert_eq!(block1.header.height, block2.header.height);
    assert_eq!(block1.header.prev_hash, block2.header.prev_hash);
    // XXX: have little probabilities to fail
    assert_eq!(block1.header.timestamp, block2.header.timestamp);
    // XXX: have little probabilities to fail
    assert_eq!(block1.header.hash, block2.header.hash);

    assert_eq!(block1.body, block2.body);
}

#[test]
fn test_store_block_and_restore_block() {
    let mut blockchain = BlockChain::new_to_table(TABLE_BLOCKS_FORTEST);

    // initialization
    let genesis_block = BlockChain::genesis();
    let prev_hash = genesis_block.header.hash.clone();
    blockchain.add_block_to_table(TABLE_BLOCKS_FORTEST, genesis_block);

    let b1 = Block::new(1, prev_hash, vec![]);
    let prev_hash = b1.header.hash.clone();
    blockchain.add_block_to_table(TABLE_BLOCKS_FORTEST, b1);

    let b2 = Block::new(2, prev_hash, vec![]);
    blockchain.add_block_to_table(TABLE_BLOCKS_FORTEST, b2);

    let block_vec = blockchain.blocks.clone();

    blockchain
        .populate_from_db_table(TABLE_BLOCKS_FORTEST)
        .expect("error when populate from db");

    _ = blockchain.db.drop_table(TABLE_BLOCKS_FORTEST);

    for (i, block) in block_vec.into_iter().enumerate() {
        let block_tmp = blockchain.blocks[i].clone();
        assert_eq!(block, block_tmp);
    }
}
