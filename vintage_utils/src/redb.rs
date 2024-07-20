use redb::{RedbKey, RedbValue, TypeName};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bytes<const N: usize>(pub [u8; N]);

impl<const N: usize> RedbValue for Bytes<N> {
    type SelfType<'a> = &'a [u8; N] where Self: 'a;
    type AsBytes<'a> = &'a [u8; N] where Self: 'a;

    fn fixed_width() -> Option<usize> {
        Some(N)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        data.try_into().unwrap()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Bytes<{N}>"))
    }
}

impl<const N: usize> RedbKey for Bytes<N> {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        data1.cmp(data2)
    }
}

pub type Hashed = Bytes<32>; // 256 bits

impl Serialize for Hashed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Hashed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(Deserialize::deserialize(deserializer)?))
    }
}

#[macro_export]
macro_rules! define_redb_table {
    ($vis:vis ($table:ident, $table_r:ident, $table_w:ident) = ($key:ty, $value:ty, $table_name:literal)) => {
        $vis struct $table<TABLE> {
            pub table: TABLE,
        }

        $vis type $table_r<'txn> = $table<redb::ReadOnlyTable<'txn, $key, $value>>;
        $vis type $table_w<'db, 'txn> = $table<redb::Table<'db, 'txn, $key, $value>>;

        impl<'txn> $table_r<'txn> {
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
