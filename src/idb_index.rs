//! Index-related code

use crate::idb_object_store::IdbObjectStore;
use accessory::Accessors;
use fancy_constructor::new;

/// A wrapper around an [`IndexedDB` index](web_sys::IdbIndex)
#[derive(Debug, new, Accessors)]
#[new(vis(pub(crate)))]
pub struct IdbIndex<'a> {
    inner: web_sys::IdbIndex,

    /// The index's object store
    #[access(get(cp))]
    object_store: &'a IdbObjectStore<'a>,
}

impl<'a> IdbIndex<'a> {
    /// Affects how the index behaves when the result of evaluating the index's key path yields an
    /// array. If true, there is one record in the index for each item in an array of keys.
    /// If false, then there is one record for each key that is an array.
    #[inline]
    #[must_use]
    pub fn multi_entry(&self) -> bool {
        self.inner.multi_entry()
    }

    /// If true, this index does not allow duplicate values for a key.
    #[inline]
    #[must_use]
    pub fn unique(&self) -> bool {
        self.inner.unique()
    }
}

impl_query_source!(IdbIndex<'_>);
