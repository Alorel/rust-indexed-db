use std::pin::Pin;

use delegate_display::DelegateDebug;
use wasm_bindgen::prelude::*;
use web_sys::DomException;

use crate::internal_utils::NightlyUnwrap;

use super::{IdbRequestFuture, ResponseFormattingFuture};

/// A [`Future`](std::future::Future) for [count](crate::idb_query_source::IdbQuerySource::count) and
/// related methods
#[derive(DelegateDebug)]
pub struct CountFuture(IdbRequestFuture);
impl_result_formatting_struct_commons!(CountFuture);

impl ResponseFormattingFuture<u32> for CountFuture {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn format_response(v: Result<Option<JsValue>, DomException>) -> Result<u32, DomException> {
        Ok(v?.nightly_unwrap().as_f64().unwrap() as u32)
    }

    #[inline]
    fn inner(&self) -> &IdbRequestFuture {
        &self.0
    }
}

impl_result_formatting_future_future!(CountFuture, u32);
