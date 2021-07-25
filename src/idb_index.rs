//! Index-related code
//!
//! Features required: `indices`

use crate::idb_object_store::IdbObjectStore;

/// A wrapper around an IndexedDB index
///
/// Features required: `indices`
#[derive(Debug)]
pub struct IdbIndex<'a> {
    inner: web_sys::IdbIndex,
    store: &'a IdbObjectStore<'a>,
}

impl<'a> IdbIndex<'a> {
    #[inline]
    pub(crate) fn new(inner: web_sys::IdbIndex, store: &'a IdbObjectStore<'a>) -> Self {
        Self { inner, store }
    }

    /// The index's object store
    #[inline]
    pub fn object_store(&self) -> &'a IdbObjectStore<'a> {
        &self.store
    }

    /// Affects how the index behaves when the result of evaluating the index's key path yields an
    /// array. If true, there is one record in the index for each item in an array of keys.
    /// If false, then there is one record for each key that is an array.
    #[inline]
    pub fn multi_entry(&self) -> bool {
        self.inner.multi_entry()
    }

    /// If true, this index does not allow duplicate values for a key.
    #[inline]
    pub fn unique(&self) -> bool {
        self.inner.unique()
    }
}

impl_query_source!(IdbIndex<'_>);
