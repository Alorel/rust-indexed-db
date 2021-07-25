//! Transaction-related code

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use web_sys::{DomException, IdbTransactionMode};

pub(crate) use idb_transaction_listeners::*;
pub use idb_transaction_result::*;

use crate::dom_string_iterator::DomStringIterator;
use crate::idb_database::IdbDatabase;
use crate::idb_object_store::IdbObjectStore;

mod idb_transaction_listeners;
mod idb_transaction_result;

/// Wrapper around an IndexedDB transaction
#[derive(Debug)]
pub struct IdbTransaction<'db> {
    inner: web_sys::IdbTransaction,
    db: &'db IdbDatabase,
    listeners: IdbTransactionListeners,
}

impl IdbTransaction<'_> {
    /// Get a iterator of the names of [IdbObjectStore] objects associated with the transaction.
    #[inline]
    pub fn object_store_names(&self) -> impl Iterator<Item = String> {
        DomStringIterator::from(self.inner.object_store_names())
    }

    /// The mode for isolating access to data in the object stores that are in the scope of the
    /// transaction.
    #[inline]
    pub fn mode(&self) -> IdbTransactionMode {
        self.inner.mode().unwrap()
    }

    /// Return a DOMException indicating the type of error that occurred when there is an
    /// unsuccessful transaction. This property is `None` if the transaction is not finished, is
    /// finished and successfully committed, or was aborted with abort() function.
    #[inline]
    pub fn error(&self) -> Option<DomException> {
        self.inner.error()
    }

    /// Rolls back all the changes to objects in the database associated with this transaction.
    /// If this transaction has been aborted or completed, this method fires an error event.
    #[inline]
    pub fn abort(self) -> Result<(), DomException> {
        Ok(self.inner.abort()?)
    }
}

impl<'db> IdbTransaction<'db> {
    #[inline]
    pub(crate) fn new(inner: web_sys::IdbTransaction, db: &'db IdbDatabase) -> Self {
        let listeners = IdbTransactionListeners::new(&inner);
        Self {
            inner,
            db,
            listeners,
        }
    }

    /// The database connection with which this transaction is associated.
    #[inline]
    pub fn db(&self) -> &'db IdbDatabase {
        &self.db
    }

    /// Returns an [IdbObjectStore] object representing an object store that is part of the scope
    /// of this transaction.
    pub fn object_store(&'db self, name: &str) -> Result<IdbObjectStore<'db>, DomException> {
        let tx = self.inner.object_store(name)?;
        Ok(IdbObjectStore::from_tx(tx, self))
    }
}

impl Drop for IdbTransaction<'_> {
    fn drop(&mut self) {
        self.inner.set_oncomplete(None);
        self.inner.set_onerror(None);
        self.inner.set_onabort(None);
    }
}

impl Future for IdbTransaction<'_> {
    type Output = IdbTransactionResult;

    #[inline]
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.listeners.do_poll(&self.inner, ctx)
    }
}
