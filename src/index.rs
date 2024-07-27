//! An [`IDBIndex`](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex) implementation.

use accessory::Accessors;
use fancy_constructor::new;
pub use index_builder::IndexBuilder;

mod object_store_ext;

mod index_builder;

use crate::object_store::ObjectStore;

/// An [`IDBIndex`](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex) implementation.
#[derive(Debug, Clone, Accessors, new)]
#[new(vis())]
pub struct Index<'a> {
    /// The object store this index is for.
    #[access(get(cp))]
    object_store: &'a ObjectStore<'a>,
    base: web_sys::IdbIndex,
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for Index<'_> {
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
