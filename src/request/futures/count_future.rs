use std::pin::Pin;

use wasm_bindgen::prelude::*;
use web_sys::DomException;

use crate::internal_utils::safe_unwrap_option;

use super::{IdbRequestFuture, ResponseFormattingFuture};

/// A [Future][std::future::Future] for [count][crate::idb_query_source::IdbQuerySource::count] and
/// related methods
#[derive(Debug)]
pub struct CountFuture(IdbRequestFuture);

impl CountFuture {
    impl_result_formatting_struct_constructor!();
}

impl ResponseFormattingFuture<u32> for CountFuture {
    fn format_response(v: Result<Option<JsValue>, DomException>) -> Result<u32, DomException> {
        Ok(safe_unwrap_option(v?).as_f64().unwrap() as u32)
    }

    #[inline]
    fn inner(&self) -> &IdbRequestFuture {
        &self.0
    }
}

impl_result_formatting_future_future!(CountFuture, u32);
