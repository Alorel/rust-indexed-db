//! Futures in use by the crate.

pub use array_map::ArrayMapFuture;
pub use basic::BasicRequest;
pub use get_all::*;
pub use maybe_errored::MaybeErrored;
pub use open_db::{OpenDbListener, OpenDbRequest};
pub use request::{Request, VoidRequest};
pub use traits::*;

mod array_map;
mod basic;
mod get_all;
mod maybe_errored;
mod open_db;
pub(crate) mod request;
mod traits;

iffeat! {
    #[cfg(feature = "list-databases")]
    mod list_dbs;
    pub use list_dbs::ListDatabasesFuture;
}

iffeat! {
    #[cfg(feature = "cursors")]
    mod cursor_next;
    pub(crate) mod cursor;
    pub use cursor_next::CursorNextRequest;
    pub use cursor::CursorRequest;
}
