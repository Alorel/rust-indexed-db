use web_sys::IdbTransactionMode;

use crate::error::Error;
use crate::{Transaction, TransactionMode};

use super::{Database, ObjectStoreName, TransactionOptions};

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

    /// Set the transaction mode for this transaction. The default mode is [`Readonly`](IdbTransactionMode::Readonly).
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
    ($db: ident, $expr: expr) => {
        match $expr {
            Ok(tx) => Ok($crate::Transaction::new($db, tx)),
            Err(e) => Err(e.into()),
        }
    };
}

impl<'a, S> TryFrom<TransactionBuilder<'a, S>> for Transaction<'a>
where
    S: ObjectStoreName,
{
    type Error = Error;

    fn try_from(this: TransactionBuilder<'a, S>) -> Result<Self, Self::Error> {
        let TransactionBuilder {
            db,
            store_names,
            mode: _,
            opts: _,
        } = this;

        try_match!(db, store_names.transaction(db))
    }
}

impl<'a, S> TryFrom<TransactionBuilder<'a, S, TransactionMode>> for Transaction<'a>
where
    S: ObjectStoreName,
{
    type Error = Error;

    fn try_from(this: TransactionBuilder<'a, S, TransactionMode>) -> Result<Self, Self::Error> {
        let TransactionBuilder {
            db,
            store_names,
            mode,
            opts: _,
        } = this;

        try_match!(db, store_names.transaction_with_mode(db, mode))
    }
}

impl<'a, S> TryFrom<TransactionBuilder<'a, S, TransactionMode, TransactionOptions>>
    for Transaction<'a>
where
    S: ObjectStoreName,
{
    type Error = Error;

    fn try_from(
        this: TransactionBuilder<'a, S, IdbTransactionMode, TransactionOptions>,
    ) -> Result<Self, Self::Error> {
        let TransactionBuilder {
            db,
            store_names,
            mode,
            opts,
        } = this;

        match opts.try_into() {
            Ok(opts) => {
                try_match!(
                    db,
                    store_names.transaction_with_mode_and_options(db, mode, opts)
                )
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a, S> TryFrom<TransactionBuilder<'a, S, (), TransactionOptions>> for Transaction<'a>
where
    S: ObjectStoreName,
{
    type Error = Error;

    fn try_from(
        this: TransactionBuilder<'a, S, (), TransactionOptions>,
    ) -> Result<Self, Self::Error> {
        this.with_mode(IdbTransactionMode::Readonly).try_into()
    }
}
