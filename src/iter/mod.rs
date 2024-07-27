//! Iterators used by the crate.

pub use dom_string_list::DomStringIter;

mod dom_string_list;

iffeat! {
    #[cfg(feature = "list-databases")]
    mod list_dbs;
    pub use list_dbs::ListDatabasesIter;
}
