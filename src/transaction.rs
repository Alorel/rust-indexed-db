//! An [`IDBTransaction`](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction) implementation.

use crate::database::Database;
use crate::error::{Error, SimpleValueError, UnexpectedDataError};
use crate::internal_utils::{StructName, SystemRepr};
pub use base::TransactionRef;
use listeners::TxListeners;
pub(crate) use options::TransactionOptionsSys;
pub use options::{TransactionDurability, TransactionOptions};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
pub(crate) use tx_sys::TransactionSys;
use wasm_bindgen::JsCast;
pub use web_sys::IdbTransactionMode as TransactionMode;

mod base;
mod listeners;
mod options;
mod tx_sys;

iffeat! {
    #[cfg(feature = "tx-done")]
    mod on_done;
    pub use on_done::{TransactionDone, TransactionFinishKind};
}

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
    on_drop: OnTransactionDrop,
}

/// An enum representing the possible behavior which a [`Transaction`] may exhibit
/// when it is dropped.
///
/// Note that unlike JavaScript's [`IDBTransaction`][1], this crate's [`Transaction`]
/// defaults to aborting - i.e., [`OnTransactionDrop::Abort`] - instead of
/// committing - i.e., [`OnTransactionDrop::Commit`] - the transaction!
///
/// [1]: https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction
#[derive(Debug, Copy, Clone, Default)]
pub enum OnTransactionDrop {
    /// Abort the [`Transaction`] when it is dropped. This is the default
    /// behavior of [`Transaction`].
    #[default]
    Abort,
    /// Commit the [`Transaction`] when it is dropped. This is the default
    /// behavior of an [`IDBTransaction`][1] in JavaScript.
    ///
    /// [1]: https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction
    Commit,
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
            on_drop: OnTransactionDrop::default(),
        }
    }

    /// Create a [`Transaction`] from an [`web_sys::IdbVersionChangeEvent`].
    ///
    /// This is useful for extracting the transaction being used to upgrade
    /// the database.
    pub(crate) fn from_raw_version_change_event(
        db: &'a Database,
        event: &web_sys::IdbVersionChangeEvent,
    ) -> crate::Result<Self> {
        let inner = match event.target() {
            Some(target) => match target.dyn_ref::<web_sys::IdbOpenDbRequest>() {
                Some(req) => req
                    .transaction()
                    .ok_or(Error::from(UnexpectedDataError::TransactionNotFound)),
                None => Err(SimpleValueError::DynCast(target.unchecked_into()).into()),
            },
            None => Err(UnexpectedDataError::NoEventTarget.into()),
        }?;
        Ok(Self::new(db, inner))
    }

    /// Set the behavior for when the [`Transaction`] is dropped
    pub fn on_drop(&mut self, on_drop: OnTransactionDrop) {
        self.on_drop = on_drop;
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

        // Given that the default behavior in JavaScript is to commit the
        // transaction when it is dropped, we only need to perform an action
        // when we want to abort the transaction.
        //
        // Typically, it would make sense to explicitly commit the transaction
        // with `TransactionSys::do_commit` when encountering `OnTransactionDrop::Commit`.
        // However, for some reason, explicitly committing the transaction causes
        // tests in a headless Chrome browser to hang, even though they pass in
        // all other contexts, including a non-headless Chrome browser. So, until
        // this is resolved, it is best to let `OnTransactionDrop::Commit` be
        // handled implicitly by the JavaScript runtime.
        if !self.done & matches!(self.on_drop, OnTransactionDrop::Abort) {
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
            .field("on_drop", &self.on_drop)
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
