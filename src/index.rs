use accessory::Accessors;
use fancy_constructor::new;
use wasm_bindgen::prelude::*;

use internal_macros::errdoc;
pub use params::IndexParameters;

use crate::{KeyPath, ObjectStore};
use crate::internal_utils::SystemRepr;
use crate::iter::DomStringIter;

mod params;

/// An [`IDBIndex`](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex) implementation.
#[derive(Debug, Clone, Accessors, new)]
#[new(vis())]
pub struct Index<'a> {
    /// The object store this index is for
    #[access(get(cp))]
    object_store: &'a ObjectStore<'a>,
    base: web_sys::IdbIndex,
}

/// Index creation impls.
impl<'a> ObjectStore<'a> {
    /// Create and return a new index on the connected database. Note that this method must be called only from a
    /// [`VersionChange`](crate::TransactionMode::Versionchange) transaction mode callback.
    #[errdoc(Index(
        ConstraintError,
        InvalidStateError,
        SyntaxError,
        TransactionInactiveError
    ))]
    pub fn create_index<'k, I, S>(
        &'a self,
        name: &str,
        key_path: &'k KeyPath<'k, S>,
    ) -> crate::Result<Index<'a>>
    where
        &'k S: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        let res = self
            .as_sys()
            .create_index_with_str_sequence(name, &key_path.to_js_value());

        self.create_index_fmt(res)
    }

    /// Create and return a new index on the connected database. Note that this method must be called only from a
    /// [`VersionChange`](crate::TransactionMode::Versionchange) transaction mode callback.
    #[errdoc(Index(
        ConstraintError,
        InvalidAccessError,
        InvalidStateError,
        SyntaxError,
        TransactionInactiveError
    ))]
    pub fn create_index_with_opts<'k, I, S>(
        &'a self,
        name: &str,
        key_path: &'k KeyPath<'k, S>,
        opts: &IndexParameters,
    ) -> crate::Result<Index<'a>>
    where
        &'k S: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        let key_path = key_path.to_js_value();
        let res = self
            .as_sys()
            .create_index_with_str_sequence_and_optional_parameters(name, &key_path, opts.as_sys());

        self.create_index_fmt(res)
    }

    /// Delete the index with the given name. Must me called within a
    /// [`Versionchange`](crate::TransactionMode::Versionchange) transaction.
    #[errdoc(Index(InvalidStateError, TransactionInactiveError, NotFoundError))]
    pub fn delete_index(&self, name: &str) -> crate::Result<()> {
        if let Err(e) = self.as_sys().delete_index(name) {
            Err(e.into())
        } else {
            Ok(())
        }
    }

    /// Return the names of the indices on this object store.
    pub fn index_names(&self) -> DomStringIter {
        DomStringIter::new(self.as_sys().index_names())
    }

    fn create_index_fmt(&self, res: Result<web_sys::IdbIndex, JsValue>) -> crate::Result<Index> {
        match res {
            Ok(sys) => Ok(Index::new(self, sys)),
            Err(e) => Err(e.into()),
        }
    }
}

impl SystemRepr for Index<'_> {
    type Repr = web_sys::IdbIndex;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.base
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.base
    }
}
