use accessory::Accessors;
use fancy_constructor::new;

use internal_macros::errdoc;

use crate::future::{MaybeErrored, Request, VoidRequest};
use crate::internal_utils::SystemRepr;
use crate::{Database, KeyRange, ToJs, Transaction, TryToJs};

#[cfg(feature = "serde")]
mod serde;

/// An [object store](web_sys::IdbObjectStore) on a database
#[derive(Accessors, new, Debug, Clone)]
#[new(vis())]
pub struct ObjectStore<'a> {
    inner: web_sys::IdbObjectStore,

    /// Reference to the database associated with the store
    #[access(get(cp))]
    db: &'a Database,

    /// Reference to the transaction associated with the store
    #[access(get(cp))]
    transaction: Option<&'a Transaction<'a>>,
}

impl<'a> ObjectStore<'a> {
    /// Add the value to the object store. Throws if the computed key already exists - use the
    /// [`put_primitive`](Self::put_primitive) method if you want to update the value.
    #[errdoc(ObjectStore(
        ReadOnlyError,
        TransactionInactiveError,
        DataErrorAdd,
        InvalidStateError,
        DataCloneError,
        ConstraintError,
    ))]
    pub fn add_primitive<V: ToJs>(&self, value: V) -> MaybeErrored<VoidRequest> {
        maybe_errored_dom!(self.as_sys().add(&value.to_js()), |req| Request::void(req))
    }

    /// Return the value of the auto increment flag for this object store.
    ///
    /// Note that every object store has its own separate auto increment counter.
    pub fn auto_increment(&self) -> bool {
        self.as_sys().auto_increment()
    }

    /// Clear all records from this object store.
    #[errdoc(ObjectStore(ReadOnlyError, TransactionInactiveError))]
    pub fn clear(&self) -> MaybeErrored<VoidRequest> {
        maybe_errored_dom!(self.as_sys().clear(), |req| Request::void(req))
    }

    /// Delete the record(s) matching the given key or key range.
    ///
    /// Bear in mind that if you are using a [`Cursor`](crate::Cursor) (requires `cursors` feature), you can use the
    /// [`Cursor::delete`](crate::Cursor::delete) method to more efficiently delete the current record — without having
    /// to explicitly look up the record's key.
    #[errdoc(ObjectStore(
        ReadOnlyError,
        TransactionInactiveError,
        InvalidStateError,
        DataErrorDelete,
    ))]
    pub fn delete<K: TryToJs>(&self, key_range: KeyRange<K>) -> MaybeErrored<VoidRequest> {
        match key_range.try_to_js() {
            Ok(js) => maybe_errored_dom!(self.as_sys().delete(&js), |req| Request::void(req)),
            Err(e) => MaybeErrored::errored(e),
        }
    }
}

impl<'a> ObjectStore<'a> {
    #[inline]
    pub(crate) fn from_version_change(inner: web_sys::IdbObjectStore, db: &'a Database) -> Self {
        Self::new(inner, db, None)
    }

    pub(crate) fn from_tx(inner: web_sys::IdbObjectStore, tx: &'a Transaction<'a>) -> Self {
        Self::new(inner, tx.db(), Some(tx))
    }
}

impl SystemRepr for ObjectStore<'_> {
    type Repr = web_sys::IdbObjectStore;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.inner
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.inner
    }
}
