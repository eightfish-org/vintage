#[macro_export]
macro_rules! define_redb_table {
    ($key_type:ty, $value_type:ty, $table_name:literal) => {
        const TABLE_DEFINITION: redb::TableDefinition<$key_type, $value_type> =
            redb::TableDefinition::new($table_name);

        pub fn open_writable_table<'db, 'txn>(
            transaction: &'txn redb::WriteTransaction<'db>,
        ) -> Result<redb::Table<'db, 'txn, $key_type, $value_type>, redb::TableError> {
            transaction.open_table(TABLE_DEFINITION)
        }

        pub fn open_table<'txn>(
            transaction: &'txn redb::ReadTransaction,
        ) -> Result<redb::ReadOnlyTable<'txn, $key_type, $value_type>, redb::TableError> {
            transaction.open_table(TABLE_DEFINITION)
        }
    };
}
