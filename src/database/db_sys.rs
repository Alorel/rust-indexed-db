use crate::transaction::{TransactionMode, TransactionOptionsSys};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// [`web_sys::IdbDatabase`] extension.
    #[wasm_bindgen(extends = web_sys::IdbDatabase, js_name = IDBDatabase)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type DbSys;

    #[wasm_bindgen(catch, method, structural, js_class = "IDBDatabase", js_name = transaction, skip_typescript)]
    pub(crate) fn transaction_with_str_and_mode_and_opts(
        this: &DbSys,
        store_name: &str,
        mode: TransactionMode,
        opts: &TransactionOptionsSys,
    ) -> Result<web_sys::IdbTransaction, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "IDBDatabase", js_name = transaction, skip_typescript)]
    pub(crate) fn transaction_with_str_sequence_and_mode_and_opts(
        this: &DbSys,
        store_names: &js_sys::Array,
        mode: TransactionMode,
        opts: &TransactionOptionsSys,
    ) -> Result<web_sys::IdbTransaction, JsValue>;
}
