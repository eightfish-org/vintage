use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;
use vintage_msg::{BlockHeight, BlockTimestamp, Entity, EntityHash, EntityId, Model, Proto, TxId};
use vintage_utils::{HashBytes, Hashed};

#[derive(Clone)]
pub struct MetaDataDb {
    db: Arc<Mutex<MetaDataDbInner>>,
}

impl MetaDataDb {
    pub async fn create(path: String) -> anyhow::Result<Self> {
        let db = spawn_blocking(|| MetaDataDbInner::create(path)).await??;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn query_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db = self.db.clone();
        spawn_blocking(move || db.lock().unwrap().query_block_height()).await?
    }

    pub async fn save_block(
        &self,
        block_height: BlockHeight,
        timestamp: BlockTimestamp,
        entities: Vec<(TxId, usize, Proto, Entity)>, // tx_id, index, proto, entity
        sql_migrations: Vec<(TxId, Proto, String, u64)>, // tx_id, proto, sql_migration, after_blocks
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        Ok(spawn_blocking(move || {
            db.lock()
                .unwrap()
                .save_block(block_height, timestamp, entities, sql_migrations)
        })
        .await??)
    }

    pub async fn query_entities(
        &self,
        proto: Proto,
        block_height_begin: BlockHeight,
        block_height_end: BlockHeight,
    ) -> anyhow::Result<Vec<EntityRow>> {
        let db = self.db.clone();
        Ok(spawn_blocking(move || {
            db.lock()
                .unwrap()
                .query_entities(proto, block_height_begin, block_height_end)
        })
        .await??)
    }

    pub async fn query_sql_migrations(
        &self,
        proto: Proto,
        block_height_begin: BlockHeight,
        block_height_end: BlockHeight,
    ) -> anyhow::Result<Vec<SqlMigrationRow>> {
        let db = self.db.clone();
        Ok(spawn_blocking(move || {
            db.lock()
                .unwrap()
                .query_sql_migrations(proto, block_height_begin, block_height_end)
        })
        .await??)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct MetaDataDbInner {
    connection: Connection,
}

impl MetaDataDbInner {
    fn create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let connection = Connection::open(path)?;

        // block height
        connection.execute(
            "CREATE TABLE IF NOT EXISTS key_value (key TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY (key));",
            (),
        )?;

        // entity
        connection.execute(
            "CREATE TABLE IF NOT EXISTS entity (tx_id BLOB NOT NULL, entity_index INTEGER NOT NULL, block_timestamp INTEGER NOT NULL, block_height INTEGER NOT NULL, proto TEXT NOT NULL, module TEXT NOT NULL, entity_id TEXT NOT NULL, entity_hash TEXT NOT NULL, PRIMARY KEY (tx_id, entity_index));",
            (),
        )?;
        connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_proto_block_height_on_entity ON entity (proto, block_height);",
            (),
        )?;

        // sql_migration
        connection.execute(
            "CREATE TABLE IF NOT EXISTS sql_migration (tx_id BLOB NOT NULL, block_height INTEGER NOT NULL, block_timestamp INTEGER NOT NULL, proto TEXT NOT NULL, sql TEXT NOT NULL, PRIMARY KEY (tx_id));",
            (),
        )?;
        connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_proto_block_height_on_sql_migration ON sql_migration (proto, block_height);",
            (),
        )?;

        Ok(Self { connection })
    }

    fn query_block_height(&self) -> anyhow::Result<BlockHeight> {
        let mut stmt = self
            .connection
            .prepare("SELECT value FROM key_value WHERE key = ?1")?;
        let mut rows = stmt.query(params!["block_height"])?;
        let row = match rows.next()? {
            Some(row) => row,
            None => return Ok(0),
        };

        let block_height_string: String = row.get(0)?; // 获取第一列的值
        let block_height = block_height_string.parse::<BlockHeight>()?;
        Ok(block_height)
    }

    fn save_block(
        &mut self,
        block_height: BlockHeight,
        block_timestamp: BlockTimestamp,
        entities: Vec<(TxId, usize, Proto, Entity)>,
        sql_migrations: Vec<(TxId, Proto, String, u64)>,
    ) -> rusqlite::Result<()> {
        // 开始事务
        let transaction = self.connection.transaction()?;

        transaction.execute(
            "INSERT OR REPLACE INTO key_value (key, value) VALUES (?1, ?2);",
            rusqlite::params!["block_height", block_height],
        )?;

        // 插入 entities 数据
        for (tx_id, entity_index, proto, entity) in entities {
            transaction.execute(
                "INSERT OR REPLACE INTO entity (tx_id, entity_index, block_height, block_timestamp, proto, module, entity_id, entity_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                rusqlite::params![tx_id.as_bytes(), entity_index, block_height, block_timestamp, proto, entity.model, entity.id, entity.hash],
            )?;
        }

        // 插入 sql_migrations 数据
        for (tx_id, proto, sql_migration, after_blocks) in sql_migrations {
            transaction.execute(
                "INSERT INTO sql_migration (tx_id, block_height, block_timestamp, proto, sql) VALUES (?1, ?2, ?3, ?4, ?5);",
                rusqlite::params![tx_id.as_bytes(), block_height + after_blocks, block_timestamp, proto, sql_migration],
            )?;
        }

        // 提交事务
        transaction.commit()
    }

    fn query_entities(
        &self,
        proto: Proto,
        block_height_begin: BlockHeight,
        block_height_end: BlockHeight,
    ) -> rusqlite::Result<Vec<EntityRow>> {
        let mut stmt = self.connection.prepare(
            "SELECT tx_id, entity_index, block_height, block_timestamp, module, entity_id, entity_hash FROM entity WHERE proto = ?1 AND block_height >= ?2 AND block_height <= ?3;",
        )?;
        let row_iter = stmt.query_map(
            params![proto, block_height_begin, block_height_end],
            |row| {
                let tx_id: HashBytes = row.get(0)?;
                Ok(EntityRow {
                    tx_id: Hashed::from(&tx_id).to_string(),
                    entity_index: row.get(1)?,
                    height: row.get(2)?,
                    timestamp: row.get(3)?,
                    module: row.get(4)?,
                    entity_id: row.get(5)?,
                    entity_hash: row.get(6)?,
                })
            },
        )?;

        let mut rows = Vec::new();
        for row in row_iter {
            rows.push(row?);
        }
        Ok(rows)
    }

    fn query_sql_migrations(
        &self,
        proto: Proto,
        block_height_begin: BlockHeight,
        block_height_end: BlockHeight,
    ) -> rusqlite::Result<Vec<SqlMigrationRow>> {
        let mut stmt = self.connection.prepare(
            "SELECT tx_id, block_height, block_timestamp, sql FROM sql_migration WHERE proto = ?1 AND block_height >= ?2 AND block_height <= ?3;",
        )?;
        let row_iter = stmt.query_map(
            params![proto, block_height_begin, block_height_end],
            |row| {
                let tx_id: HashBytes = row.get(0)?;
                Ok(SqlMigrationRow {
                    tx_id: Hashed::from(&tx_id).to_string(),
                    height: row.get(1)?,
                    timestamp: row.get(2)?,
                    sql: row.get(3)?,
                })
            },
        )?;

        let mut rows = Vec::new();
        for row in row_iter {
            rows.push(row?);
        }
        Ok(rows)
    }
}

#[derive(Serialize)]
pub struct EntityRow {
    pub tx_id: String,
    pub entity_index: u32,
    pub height: BlockHeight,
    pub timestamp: BlockTimestamp,
    pub module: Model,
    pub entity_id: EntityId,
    pub entity_hash: EntityHash,
}

#[derive(Serialize)]
pub struct SqlMigrationRow {
    pub tx_id: String,
    pub height: BlockHeight,
    pub timestamp: BlockTimestamp,
    pub sql: String,
}
