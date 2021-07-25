use super::IdbRequestFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use web_sys::DomException;

use crate::internal_utils::safe_unwrap_option;
use wasm_bindgen::prelude::*;

/// Future for cursors' advance() and continue()
///
/// Features required: `cursors`
#[derive(Debug)]
pub(crate) struct IdbCursorAdvancementFuture(IdbRequestFuture);

impl IdbCursorAdvancementFuture {
    #[inline]
    pub fn new(base: IdbRequestFuture) -> Self {
        Self(base)
    }

    fn on_done(res: Result<Option<JsValue>, DomException>) -> Result<bool, DomException> {
        Ok(!safe_unwrap_option(res?).is_null())
    }
}

impl Future for IdbCursorAdvancementFuture {
    type Output = Result<bool, DomException>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.do_poll(ctx).map(Self::on_done)
    }
}
