use super::{TransactionMode, TransactionSys};
use crate::database::Database;
use crate::error::Error;
use crate::internal_utils::{StructName, SystemRepr};
use crate::iter::DomStringIter;
use crate::object_store::ObjectStore;
use accessory::Accessors;
use internal_macros::errdoc;
use sealed::sealed;
use std::fmt::{Debug, Formatter};
use wasm_bindgen::prelude::*;

/// Reference to a [`Transaction`](super::Transaction) that cannot be explicitly committed or rolled back.
#[derive(Accessors, StructName)]
pub struct TransactionRef<'a> {
    /// Reference to the database associated with the transaction.
    #[access(get(cp))]
    db: &'a Database,

    #[access(all(vis(pub(super))), get)]
    transaction: TransactionSys,
}

impl<'a> TransactionRef<'a> {
    pub(crate) fn new(db: &'a Database, transaction: web_sys::IdbTransaction) -> Self {
        Self {
            db,
            transaction: transaction.unchecked_into(),
        }
    }

    /// Get an object store that's part of the transaction.
    #[errdoc(Transaction(NotFoundError, InvalidStateError))]
    #[allow(clippy::missing_errors_doc)]
    pub fn object_store(&self, name: &str) -> crate::Result<ObjectStore<'_>> {
        match self.as_sys().object_store(name) {
            Ok(store) => Ok(ObjectStore::new(store, self)),
            Err(e) => Err(e.into()),
        }
    }

    /// Get an iterator of the names of [`IdbObjectStore`](ObjectStore) objects
    /// associated with the transaction.
    pub fn object_store_names(&self) -> DomStringIter<'_> {
        DomStringIter::new(self.as_sys().object_store_names())
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

    /// Return a Future that resolves when the transaction finishes, successfully or not.
    ///
    /// The primary use case is awaiting a `versionchange` transaction. You must make sure that the transaction hasn't
    /// finished already before you call this fn otherwise the future will never complete.
    #[cfg(feature = "tx-done")]
    #[allow(clippy::missing_errors_doc)]
    pub fn on_done(&self) -> crate::Result<super::TransactionDone> {
        super::TransactionDone::new(self.as_sys().clone())
    }
}

impl Debug for TransactionRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(Self::TYPE_NAME)
            .field("db", self.db())
            .field("transaction", self.transaction())
            .finish()
    }
}

#[sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for TransactionRef<'_> {
    type Repr = TransactionSys;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.transaction
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.transaction
    }
}
