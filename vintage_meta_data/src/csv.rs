use crate::MetaDataDb;
use vintage_msg::{BlockHeight, Proto};

pub async fn read_sql_migrations_csv(
    db: &MetaDataDb,
    proto: Proto,
    block_height_begin: BlockHeight,
    block_height_end: BlockHeight,
) -> anyhow::Result<String> {
    let rows = db
        .query_sql_migrations(proto, block_height_begin, block_height_end)
        .await?;
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(["block height", "timestamp", "tx id", "sql"])?;
    for row in rows {
        writer.write_record([
            row.height.to_string(),
            row.timestamp.to_string(),
            row.tx_id,
            row.sql,
        ])?;
    }
    Ok(String::from_utf8(writer.into_inner()?)?)
}

pub async fn read_entities_csv(
    db: &MetaDataDb,
    proto: Proto,
    block_height_begin: BlockHeight,
    block_height_end: BlockHeight,
) -> anyhow::Result<String> {
    let rows = db
        .query_entities(proto, block_height_begin, block_height_end)
        .await?;
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record([
        "block height",
        "timestamp",
        "tx id",
        "module",
        "entity id",
        "entity hash",
    ])?;
    for row in rows {
        writer.write_record([
            row.height.to_string(),
            row.timestamp.to_string(),
            row.tx_id,
            row.module,
            row.entity_id,
            row.entity_hash,
        ])?;
    }
    Ok(String::from_utf8(writer.into_inner()?)?)
}
