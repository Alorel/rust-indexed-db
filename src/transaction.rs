//! An [`IDBTransaction`](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction) implementation.

use crate::database::Database;
use crate::error::Error;
use crate::internal_utils::{StructName, SystemRepr};
use crate::iter::DomStringIter;
use crate::object_store::ObjectStore;
use accessory::Accessors;
use internal_macros::errdoc;
use listeners::TxListeners;
pub(crate) use options::TransactionOptionsSys;
pub use options::{TransactionDurability, TransactionOptions};
use std::fmt::{Debug, Formatter};
pub(crate) use tx_sys::TransactionSys;
pub use web_sys::IdbTransactionMode as TransactionMode;

mod listeners;
mod options;
mod tx_sys;

/// An [`IDBTransaction`](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction) implementation.
///
/// Unlike JS transactions, **this defaults to aborting the transaction instead of committing it** -
/// the opposite of the default behaviour in JS. Dropping the transaction without calling
/// [`commit`](Transaction::commit) will act the same as calling
/// [`abort`](Transaction::abort).
#[derive(Accessors, StructName)]
#[must_use]
pub struct Transaction<'a> {
    listeners: TxListeners,

    /// Reference to the database associated with the transaction.
    #[access(get(cp))]
    db: &'a Database,

    done: bool,
}

/// A [transaction's](Transaction) result.
#[derive(Debug, PartialEq, derive_more::From)]
enum TransactionResult {
    /// Transaction committed successfully.
    Ok,

    /// Transaction errored.
    Err(Error),

    /// Transaction aborted.
    Abort,
}

macro_rules! map_result {
    ($expr: expr, ok: $ok: ident, unexpected: $unexpect: ident => $err: ident) => {
        match $expr {
            TransactionResult::$ok => Ok(()),
            TransactionResult::Err(e) => Err(e),
            TransactionResult::$unexpect => Err(crate::error::UnexpectedDataError::$err.into()),
        }
    };
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(db: &'a Database, inner: web_sys::IdbTransaction) -> Self {
        Self {
            listeners: TxListeners::new(inner),
            db,
            done: false,
        }
    }

    /// Get an iterator of the names of [`IdbObjectStore`](ObjectStore) objects
    /// associated with the transaction.
    pub fn object_store_names(&self) -> DomStringIter {
        DomStringIter::new(self.as_sys().object_store_names())
    }

    /// Get an object store that's part of the transaction.
    #[errdoc(Transaction(NotFoundError, InvalidStateError))]
    #[allow(clippy::missing_errors_doc)]
    pub fn object_store(&self, name: &str) -> crate::Result<ObjectStore> {
        match self.as_sys().object_store(name) {
            Ok(store) => Ok(ObjectStore::from_tx(store, self)),
            Err(e) => Err(e.into()),
        }
    }

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
    #[allow(clippy::missing_errors_doc)]
    pub async fn abort(mut self) -> crate::Result<()> {
        self.done = true;
        self.as_sys().abort()?;

        map_result!(self.listeners.recv().await, ok: Abort, unexpected: Ok => TransactionCommitted)
    }

    /// Commits all the changes made to objects in the database associated with this transaction.
    #[allow(clippy::missing_errors_doc)]
    pub async fn commit(mut self) -> crate::Result<()> {
        self.done = true;
        self.as_sys().do_commit()?;

        map_result!(self.listeners.recv().await, ok: Ok, unexpected: Abort => TransactionAborted)
    }
}

impl SystemRepr for Transaction<'_> {
    type Repr = TransactionSys;

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

        if !self.done {
            let _ = self.as_sys().abort();
        }
    }
}

impl Debug for Transaction<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(Self::TYPE_NAME)
            .field("transaction", self.as_sys())
            .field("db", self.db())
            .field("done", &self.done)
            .finish()
    }
}
