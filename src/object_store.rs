//! An [`IDBObjectStore`](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore) implementation.

use accessory::Accessors;
use fancy_constructor::new;

use internal_macros::errdoc;

use crate::database::Database;
use crate::future::{Request, VoidRequest};
use crate::internal_utils::SystemRepr;
use crate::query_source::QuerySource;
use crate::transaction::Transaction;
use crate::KeyRange;
pub use add_put::{Add, AddPut, Put};
pub use delete::Delete;

mod add_put;

mod delete;

/// An [`IDBObjectStore`](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore) implementation.
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
    /// [`put`](Self::put) method if you want to update the value.
    ///
    /// The value should implement either [`TryToJs`](crate::primitive::TryToJs) or, if the `serde` feature is enabled,
    /// [`Serialize`](serde::Serialize).
    ///
    /// # Returns
    ///
    /// A builder that, when built and `await`ed, resolves to the key for the added record, or `()` if the key type
    /// cannot be determined ([`with_key`](Add::with_key)/[`with_key_type`](Add::with_key_type) not called or
    /// [`without_key_type`](Add::without_key_type) called).
    #[errdoc(ObjectStore(
        ReadOnlyError,
        TransactionInactiveError,
        DataErrorAdd,
        InvalidStateError,
        DataCloneError,
        ConstraintError,
    ))]
    #[inline]
    pub fn add<V>(&self, value: V) -> Add<V> {
        Add::new(self, value)
    }

    /// Add the value to the object store. Overwrites the record if the computed key already exists - use the
    /// [`add`](Self::add) method if you want to throw an error instead.
    ///
    /// The value should implement either [`TryToJs`](crate::primitive::TryToJs) or, if the `serde` feature is enabled,
    /// [`Serialize`](serde::Serialize).
    ///
    /// # Returns
    ///
    /// A builder that, when built and `await`ed, resolves to the key for the added record, or `()` if the key type
    /// cannot be determined ([`with_key`](Put::with_key)/[`with_key_type`](Put::with_key_type) not called or
    /// [`without_key_type`](Put::without_key_type) called).
    #[errdoc(ObjectStore(
        ReadOnlyError,
        TransactionInactiveError,
        DataErrorAdd,
        InvalidStateError,
        DataCloneError,
        ConstraintError,
    ))]
    #[inline]
    pub fn put<V>(&self, value: V) -> Put<V> {
        Put::new(self, value)
    }

    /// Return the value of the auto increment flag for this object store.
    ///
    /// Note that every object store has its own separate auto increment counter.
    #[inline]
    #[must_use]
    pub fn auto_increment(&self) -> bool {
        self.as_sys().auto_increment()
    }

    /// Clear all records from this object store.
    #[errdoc(ObjectStore(ReadOnlyError, TransactionInactiveError))]
    #[allow(clippy::missing_errors_doc)]
    pub fn clear(&self) -> crate::Result<VoidRequest> {
        let req = self.as_sys().clear()?;

        Ok(Request::new(req))
    }

    /// Delete the record(s) matching the given key or key range.
    ///
    /// The range should implement either [`TryToJs`](crate::primitive::TryToJs) or, if the `serde` feature is enabled,
    /// [`Serialize`](serde::Serialize).
    ///
    /// Bear in mind that if you are using a [`Cursor`](crate::cursor::Cursor)
    /// (requires `cursors` feature), you can use the
    /// [`Cursor::delete`](crate::cursor::Cursor::delete) method to more efficiently delete the
    /// current record — without having to explicitly look up the record's key.
    #[errdoc(ObjectStore(
        ReadOnlyError,
        TransactionInactiveError,
        InvalidStateError,
        DataErrorDelete,
    ))]
    pub fn delete<K, I>(&self, key_range: I) -> Delete<K>
    where
        I: Into<KeyRange<K>>,
    {
        Delete::new(self, key_range.into())
    }

    /// Delete this object store.
    #[errdoc(Database(
        InvalidStateErrorObjectStore,
        TransactionInactiveError,
        NotFoundErrorDeleteObjectStore
    ))]
    #[allow(clippy::missing_errors_doc)]
    pub fn delete_object_store(self) -> crate::Result<()> {
        self.db.delete_object_store(&self.name())
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
