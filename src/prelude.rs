//! The file to `use` everything from in most cases

#[cfg(feature = "cursors")]
pub use {crate::idb_cursor::*, web_sys::IdbCursorDirection};
#[cfg(feature = "indices")]
pub use {crate::idb_index::*, web_sys::IdbIndexParameters};
pub use {
    crate::{
        idb_database::*,
        idb_key_path::*,
        idb_object_store::{IdbObjectStore, IdbObjectStoreParameters},
        idb_query_source::IdbQuerySource,
        idb_transaction::{IdbTransaction, IdbTransactionResult},
        request::*,
    },
    web_sys::IdbTransactionMode,
};
