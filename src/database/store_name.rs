use sealed::sealed;
use wasm_bindgen::prelude::*;

use super::{Database, TransactionOptionsSys};
use crate::internal_utils::slice_to_arr;
use crate::internal_utils::SystemRepr;
use crate::transaction::TransactionMode;

type TxResult = Result<web_sys::IdbTransaction, JsValue>;

/// Marks a type as usable for object store names.
#[sealed]
#[allow(clippy::missing_errors_doc)]
pub trait ObjectStoreName {
    /// Open a transaction on the given database.
    #[doc(hidden)]
    fn transaction(self, db: &Database) -> TxResult;

    /// Open a transaction on the given database with the given mode.
    #[doc(hidden)]
    fn transaction_with_mode(self, db: &Database, mode: TransactionMode) -> TxResult;

    /// Open a transaction on the given database with the given mode and options.
    #[doc(hidden)]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: TransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult;
}

#[sealed]
impl ObjectStoreName for &str {
    #[inline]
    fn transaction(self, db: &Database) -> TxResult {
        db.as_sys().transaction_with_str(self.as_ref())
    }

    #[inline]
    fn transaction_with_mode(self, db: &Database, mode: web_sys::IdbTransactionMode) -> TxResult {
        db.as_sys()
            .transaction_with_str_and_mode(self.as_ref(), mode)
    }

    #[inline]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: web_sys::IdbTransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        db.as_sys()
            .transaction_with_str_and_mode_and_opts(self.as_ref(), mode, &opts)
    }
}

#[sealed]
impl ObjectStoreName for &String {
    #[inline]
    fn transaction(self, db: &Database) -> TxResult {
        self.as_str().transaction(db)
    }

    #[inline]
    fn transaction_with_mode(self, db: &Database, mode: web_sys::IdbTransactionMode) -> TxResult {
        self.as_str().transaction_with_mode(db, mode)
    }

    #[inline]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: web_sys::IdbTransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        self.as_str()
            .transaction_with_mode_and_options(db, mode, opts)
    }
}

#[sealed]
impl ObjectStoreName for String {
    #[inline]
    fn transaction(self, db: &Database) -> TxResult {
        <&Self>::transaction(&self, db)
    }

    #[inline]
    fn transaction_with_mode(self, db: &Database, mode: web_sys::IdbTransactionMode) -> TxResult {
        <&Self>::transaction_with_mode(&self, db, mode)
    }

    #[inline]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: web_sys::IdbTransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        <&Self>::transaction_with_mode_and_options(&self, db, mode, opts)
    }
}

#[sealed]
impl<T: AsRef<str>> ObjectStoreName for &[T] {
    fn transaction(self, db: &Database) -> TxResult {
        db.as_sys()
            .transaction_with_str_sequence(&slice_to_arr(self))
    }

    fn transaction_with_mode(self, db: &Database, mode: web_sys::IdbTransactionMode) -> TxResult {
        db.as_sys()
            .transaction_with_str_sequence_and_mode(&slice_to_arr(self), mode)
    }

    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: web_sys::IdbTransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        db.as_sys().transaction_with_str_sequence_and_mode_and_opts(
            &slice_to_arr(self),
            mode,
            &opts,
        )
    }
}

#[sealed]
impl<T: AsRef<str>, const N: usize> ObjectStoreName for [T; N] {
    #[inline]
    fn transaction(self, db: &Database) -> TxResult {
        self.as_slice().transaction(db)
    }

    #[inline]
    fn transaction_with_mode(self, db: &Database, mode: TransactionMode) -> TxResult {
        self.as_slice().transaction_with_mode(db, mode)
    }

    #[inline]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: TransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        self.as_slice()
            .transaction_with_mode_and_options(db, mode, opts)
    }
}

#[sealed]
impl<T: AsRef<str>> ObjectStoreName for Vec<T> {
    fn transaction(self, db: &Database) -> TxResult {
        self.as_slice().transaction(db)
    }

    fn transaction_with_mode(self, db: &Database, mode: TransactionMode) -> TxResult {
        self.as_slice().transaction_with_mode(db, mode)
    }

    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: TransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        self.as_slice()
            .transaction_with_mode_and_options(db, mode, opts)
    }
}
