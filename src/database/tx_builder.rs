use super::{Database, ObjectStoreName};
use crate::error::Error;
use crate::transaction::{Transaction, TransactionMode, TransactionOptions};
use sealed::sealed;

/// Start a transaction. Finish the builder with a call to [`Build::build`](crate::Build::build).
#[derive(Debug, Clone)]
#[must_use]
pub struct TransactionBuilder<'a, S, M = (), O = ()> {
    db: &'a Database,
    store_names: S,
    mode: M,
    opts: O,
}

impl<'a, S: ObjectStoreName> TransactionBuilder<'a, S> {
    #[inline]
    pub(super) fn new(db: &'a Database, store_names: S) -> Self {
        Self {
            db,
            store_names,
            mode: (),
            opts: (),
        }
    }
}

impl<'a, S, M, O> TransactionBuilder<'a, S, M, O> {
    /// Set the store name(s) this transaction will run on.
    #[inline]
    pub fn with_store_names<S2: ObjectStoreName>(
        self,
        store_names: S2,
    ) -> TransactionBuilder<'a, S2, M, O> {
        TransactionBuilder {
            db: self.db,
            store_names,
            mode: self.mode,
            opts: self.opts,
        }
    }

    /// Set the transaction mode for this transaction. The default mode is [`Readonly`](TransactionMode::Readonly).
    #[inline]
    pub fn with_mode(self, mode: TransactionMode) -> TransactionBuilder<'a, S, TransactionMode, O> {
        TransactionBuilder {
            db: self.db,
            store_names: self.store_names,
            mode,
            opts: self.opts,
        }
    }

    /// Set the options for this transaction.
    #[inline]
    pub fn with_options(
        self,
        opts: TransactionOptions,
    ) -> TransactionBuilder<'a, S, M, TransactionOptions> {
        TransactionBuilder {
            db: self.db,
            store_names: self.store_names,
            mode: self.mode,
            opts,
        }
    }
}

macro_rules! try_match {
    ($db: expr, $expr: expr) => {
        match $expr {
            Ok(tx) => Ok($crate::transaction::Transaction::new($db, tx)),
            Err(e) => Err(e.into()),
        }
    };
}

#[sealed]
impl<'a, S> crate::Build for TransactionBuilder<'a, S>
where
    S: ObjectStoreName,
{
    type Ok = Transaction<'a>;
    type Err = Error;

    fn build(self) -> crate::Result<Transaction<'a>> {
        try_match!(self.db, self.store_names.transaction(self.db))
    }
}

#[sealed]
impl<'a, S> crate::Build for TransactionBuilder<'a, S, TransactionMode>
where
    S: ObjectStoreName,
{
    type Ok = Transaction<'a>;
    type Err = Error;

    fn build(self) -> crate::Result<Transaction<'a>> {
        let req = self.store_names.transaction_with_mode(self.db, self.mode);
        try_match!(self.db, req)
    }
}

#[sealed]
impl<'a, S> crate::Build for TransactionBuilder<'a, S, TransactionMode, TransactionOptions>
where
    S: ObjectStoreName,
{
    type Ok = Transaction<'a>;
    type Err = Error;

    fn build(self) -> crate::Result<Transaction<'a>> {
        let opts = self.opts.try_into()?;
        let req = self
            .store_names
            .transaction_with_mode_and_options(self.db, self.mode, opts);
        try_match!(self.db, req)
    }
}

#[sealed]
impl<'a, S> crate::Build for TransactionBuilder<'a, S, (), TransactionOptions>
where
    S: ObjectStoreName,
{
    type Ok = Transaction<'a>;
    type Err = Error;

    fn build(self) -> crate::Result<Transaction<'a>> {
        self.with_mode(TransactionMode::Readonly).build()
    }
}
