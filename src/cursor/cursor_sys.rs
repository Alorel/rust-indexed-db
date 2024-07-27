use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// [`web_sys::IdbCursor`] extension.
    #[wasm_bindgen(
        extends = web_sys::IdbCursorWithValue,
        extends = web_sys::IdbCursor,
        js_name = IDBCursorWithValue,
    )]
    #[derive(Clone, Debug)]
    pub type CursorSys;

    #[wasm_bindgen(
        method,
        structural,
        getter,
        js_class = "IDBCursorWithValue",
        js_name = request,
        skip_typescript
    )]
    pub(crate) fn maybe_req(this: &CursorSys) -> JsValue;

    #[wasm_bindgen(
        method,
        structural,
        getter,
        js_class = "IDBCursorWithValue",
        js_name = request,
        skip_typescript
    )]
    pub(crate) fn req(this: &CursorSys) -> web_sys::IdbRequest;
}
