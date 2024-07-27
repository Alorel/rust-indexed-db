//! Internal type representations

pub use crate::database::db_sys::DbSys;
pub use crate::internal_utils::{SystemRepr, Void};
pub use crate::object_store::add_put::kind::InsertKind;
pub use crate::query_source::get_all::kind::GetAllKind;
pub use crate::query_source::internal::QuerySourceInternal;

#[cfg(feature = "cursors")]
pub use crate::{
    cursor::cursor_sys::CursorSys, future::cursor::CursorKind as CursorFutureKind,
    query_source::cursor::kind::CursorKind,
};

#[allow(missing_docs)]
pub mod record_insert_kind {
    pub use crate::object_store::add_put::kind::{Add, Put};
}

#[allow(missing_docs)]
pub mod get_all_kind {
    pub use crate::query_source::get_all::kind::{Key, Record};
}

#[cfg(feature = "cursors")]
#[allow(missing_docs)]
pub mod cursor_kind {
    pub use crate::query_source::cursor::kind::{Key, Record};
}
