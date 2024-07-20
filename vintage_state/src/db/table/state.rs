use vintage_msg::RowId;
use vintage_utils::define_redb_table;

define_redb_table! {
    pub(crate) (State, StateR, StateW) = (RowId, Vec<u8>, "state")
}
