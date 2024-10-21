//! Iterators used by the crate.

pub use dom_string_list::DomStringIter;

pub use array_map::ArrayMapIter;

pub use get_all::*;

mod array_map;
mod dom_string_list;
mod get_all;

iffeat! {
    #[cfg(feature = "list-databases")]
    mod list_dbs;
    pub use list_dbs::ListDatabasesIter;
}
