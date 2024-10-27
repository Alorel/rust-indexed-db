//! An [`IDBTransaction`](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction) implementation.

use crate::database::Database;
use crate::error::Error;
use crate::internal_utils::{StructName, SystemRepr};
pub use base::TransactionRef;
use listeners::TxListeners;
pub(crate) use options::TransactionOptionsSys;
pub use options::{TransactionDurability, TransactionOptions};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
pub(crate) use tx_sys::TransactionSys;
pub use web_sys::IdbTransactionMode as TransactionMode;

mod base;
mod listeners;
mod options;
mod tx_sys;

/// An [`IDBTransaction`](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction) implementation.
///
/// Unlike JS transactions, **this defaults to aborting the transaction instead of committing it** -
/// the opposite of the default behaviour in JS. Dropping the transaction without calling
/// [`commit`](Transaction::commit) will act the same as calling
/// [`abort`](Transaction::abort) - see browser compatibility note on the `abort` fn for caveats.
#[derive(StructName)]
#[must_use]
pub struct Transaction<'a> {
    listeners: TxListeners<'a>,

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
            listeners: TxListeners::new(db, inner),
            done: false,
        }
    }

    /// Rolls back all the changes to objects in the database associated with this transaction.
    ///
    /// # Browser compatibility note
    ///
    /// Note that, depending on the browser, the this function may or may not roll back requests that have already been
    /// `await`ed. Chrome & Firefox, for example, appear to roll back `await`ed requests, while Safari only rolls back
    /// requests that have been built ([primitive](crate::BuildPrimitive) | [serde](crate::BuildSerde)), but not
    /// `await`ed.
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

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for Transaction<'_> {
    type Repr = TransactionSys;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        self.transaction()
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

impl<'a> Deref for Transaction<'a> {
    type Target = TransactionRef<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.listeners.tx_ref()
    }
}
