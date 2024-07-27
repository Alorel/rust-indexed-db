use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = web_sys::IdbTransaction, js_name = IDBTransaction)]
    #[derive(Clone, Debug)]
    pub type TransactionSys;

    #[wasm_bindgen(catch, method, structural, js_class="IDBTransaction", js_name=commit)]
    pub(crate) fn do_commit(this: &TransactionSys) -> Result<(), JsValue>;
}
