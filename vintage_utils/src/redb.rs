#[macro_export]
macro_rules! define_redb_table {
    ($vis:vis ($table:ident, $table_w:ident) = ($key:ty, $value:ty, $table_name:literal)) => {
        $vis struct $table<TABLE> {
            pub table: TABLE,
        }

        $vis type $table_w<'db, 'txn> = $table<redb::Table<'db, 'txn, $key, $value>>;

        impl<'txn> $table<redb::ReadOnlyTable<'txn, $key, $value>> {
            #[inline]
            pub fn open_table(txn: &'txn redb::ReadTransaction) -> std::result::Result<Self, redb::TableError> {
                let table = txn.open_table(redb::TableDefinition::new($table_name))?;
                Ok(Self { table })
            }
        }

        impl<'db, 'txn> $table_w<'db, 'txn> {
            #[inline]
            pub fn open_writable_table(txn: &'txn redb::WriteTransaction<'db>) -> std::result::Result<Self, redb::TableError> {
                let table = txn.open_table(redb::TableDefinition::new($table_name))?;
                Ok(Self { table })
            }

            #[inline]
            pub fn insert<'k, 'v>(
                &mut self,
                key: impl std::borrow::Borrow<<$key as redb::RedbValue>::SelfType<'k>>,
                value: impl std::borrow::Borrow<<$value as redb::RedbValue>::SelfType<'v>>,
            ) -> std::result::Result<std::option::Option<redb::AccessGuard<$value>>, redb::StorageError> {
                self.table.insert(key, value)
            }
        }

        impl<TABLE> $table<TABLE>
        where
            TABLE: redb::ReadableTable<$key, $value>,
        {
            #[inline]
            pub fn get<'k>(
                &self,
                key: impl std::borrow::Borrow<<$key as redb::RedbValue>::SelfType<'k>>,
            ) -> std::result::Result<std::option::Option<redb::AccessGuard<$value>>, redb::StorageError> {
                redb::ReadableTable::<$key, $value>::get(&self.table, key)
            }

            #[inline]
            pub fn exists<'k>(
                &self,
                key: impl std::borrow::Borrow<<$key as redb::RedbValue>::SelfType<'k>>,
            ) -> std::result::Result<bool, redb::StorageError> {
                let option = redb::ReadableTable::<$key, $value>::get(&self.table, key)?;
                Ok(option.is_some())
            }
        }
    };
}
