use std::fmt::{Debug, Formatter};

use accessory::Accessors;

use internal_macros::errdoc;
use listeners::TxListeners;
pub use options::*;

use crate::error::Error;
use crate::internal_utils::{StructName, SystemRepr};
use crate::iter::DomStringIter;
use crate::{Database, ObjectStore, TransactionMode};

mod listeners;
mod options;

/// Wrapper around a [transaction]((web_sys::IdbTransaction)).
///
/// Unlike JS transactions, **this defaults to aborting the transaction instead of committing it** -
/// the opposite of the default behaviour in JS. Dropping the transaction without calling
/// [`commit`](Transaction::commit) will act the same as calling
/// [`abort`](Transaction::abort).
#[derive(Accessors, StructName)]
#[must_use]
pub struct Transaction<'a> {
    listeners: TxListeners,

    /// Reference to the database associated with the transaction
    #[access(get(cp))]
    db: &'a Database,

    committed: bool,
}

/// A [transaction's](Transaction) result
#[derive(Debug, derive_more::From)]
pub enum IdbTransactionResult {
    /// Transaction committed successfully
    Success,

    /// Transaction errored
    Error(Error),

    /// Transaction aborted
    Abort,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(db: &'a Database, inner: web_sys::IdbTransaction) -> Self {
        Self {
            listeners: TxListeners::new(inner),
            db,
            committed: false,
        }
    }

    /// Get an iterator of the names of [`IdbObjectStore`](ObjectStore) objects
    /// associated with the transaction.
    pub fn object_store_names(&self) -> DomStringIter {
        DomStringIter::new(self.as_sys().object_store_names())
    }

    /// Get an object store that's part of the transaction.
    #[errdoc(Transaction(NotFoundError, InvalidStateError))]
    pub fn object_store(&self, name: &str) -> crate::Result<ObjectStore> {
        match self.as_sys().object_store(name) {
            Ok(store) => Ok(ObjectStore::from_tx(store, self)),
            Err(e) => Err(e.into()),
        }
    }
}

impl Transaction<'_> {
    /// The mode for isolating access to data in the object stores that are in the scope of the
    /// transaction.
    #[must_use]
    pub fn mode(&self) -> TransactionMode {
        self.as_sys().mode().unwrap_or(TransactionMode::Readonly)
    }

    /// # Returns
    /// `None` if the transaction is not finished, is finished and successfully committed,
    /// or was aborted with `abort()` function.
    #[must_use]
    pub fn error(&self) -> Option<Error> {
        self.as_sys().error().map(Into::into)
    }

    /// Rolls back all the changes to objects in the database associated with this transaction.
    pub async fn abort(mut self) -> IdbTransactionResult {
        self.committed = true;
        self.do_abort();
        self.listeners.recv().await
    }

    /// Commits all the changes made to objects in the database associated with this transaction.
    pub async fn commit(mut self) -> IdbTransactionResult {
        self.committed = true;
        let _ = self.as_sys().commit();
        self.listeners.recv().await
    }

    #[inline]
    fn do_abort(&self) {
        let _ = self.as_sys().abort();
    }
}

impl SystemRepr for Transaction<'_> {
    type Repr = web_sys::IdbTransaction;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        self.listeners.transaction()
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.as_sys().clone()
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        self.listeners.free_listeners();

        if !self.committed {
            self.do_abort();
        }
    }
}

impl Debug for Transaction<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(Self::TYPE_NAME)
            .field("transaction", self.as_sys())
            .field("db", self.db())
            .field("committed", &self.committed)
            .finish()
    }
}
