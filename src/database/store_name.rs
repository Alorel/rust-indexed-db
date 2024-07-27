use sealed::sealed;
use wasm_bindgen::prelude::*;
use web_sys::IdbTransaction as IdbTransactionBase;
use web_sys::IdbTransactionMode;

use crate::internal_utils::slice_to_arr;
use crate::internal_utils::SystemRepr;

use super::{Database, TransactionOptionsSys};

type TxResult = Result<IdbTransactionBase, JsValue>;

#[sealed]
pub trait ObjectStoreName {
    fn transaction(self, db: &Database) -> TxResult;

    fn transaction_with_mode(self, db: &Database, mode: IdbTransactionMode) -> TxResult;

    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: IdbTransactionMode,
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
    fn transaction_with_mode(self, db: &Database, mode: IdbTransactionMode) -> TxResult {
        db.as_sys()
            .transaction_with_str_and_mode(self.as_ref(), mode)
    }

    #[inline]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: IdbTransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        db.as_sys()
            .transaction_with_str_and_mode_and_opts(self.as_ref(), mode, &opts)
    }
}

#[sealed]
impl ObjectStoreName for &[&str] {
    fn transaction(self, db: &Database) -> TxResult {
        db.as_sys()
            .transaction_with_str_sequence(&slice_to_arr(self))
    }

    fn transaction_with_mode(self, db: &Database, mode: IdbTransactionMode) -> TxResult {
        db.as_sys()
            .transaction_with_str_sequence_and_mode(&slice_to_arr(self), mode)
    }

    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: IdbTransactionMode,
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
impl ObjectStoreName for &String {
    #[inline]
    fn transaction(self, db: &Database) -> TxResult {
        self.as_str().transaction(db)
    }

    #[inline]
    fn transaction_with_mode(self, db: &Database, mode: IdbTransactionMode) -> TxResult {
        self.as_str().transaction_with_mode(db, mode)
    }

    #[inline]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: IdbTransactionMode,
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
        <&String>::transaction(&self, db)
    }

    #[inline]
    fn transaction_with_mode(self, db: &Database, mode: IdbTransactionMode) -> TxResult {
        <&String>::transaction_with_mode(&self, db, mode)
    }

    #[inline]
    fn transaction_with_mode_and_options(
        self,
        db: &Database,
        mode: IdbTransactionMode,
        opts: TransactionOptionsSys,
    ) -> TxResult {
        <&String>::transaction_with_mode_and_options(&self, db, mode, opts)
    }
}
