use std::pin::Pin;

use wasm_bindgen::prelude::*;
use web_sys::DomException;

use crate::internal_utils::{optional_jsvalue_undefined, safe_unwrap_option};

use super::{IdbRequestFuture, ResponseFormattingFuture};

/// A [Future][std::future::Future] that resolves to `None` if the result is `undefined`
#[derive(Debug)]
pub struct OptionalJsValueFuture(IdbRequestFuture);

impl OptionalJsValueFuture {
    impl_result_formatting_struct_constructor!();
}

impl ResponseFormattingFuture<Option<JsValue>> for OptionalJsValueFuture {
    fn format_response(
        v: Result<Option<JsValue>, DomException>,
    ) -> Result<Option<JsValue>, DomException> {
        Ok(optional_jsvalue_undefined(safe_unwrap_option(v?)))
    }

    #[inline]
    fn inner(&self) -> &IdbRequestFuture {
        &self.0
    }
}

impl_result_formatting_future_future!(OptionalJsValueFuture, Option<JsValue>);
