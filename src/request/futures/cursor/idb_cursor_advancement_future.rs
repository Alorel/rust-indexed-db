use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use delegate_display::DelegateDebug;
use fancy_constructor::new;
use wasm_bindgen::prelude::*;
use web_sys::DomException;

use crate::internal_utils::NightlyUnwrap;

use super::super::IdbRequestFuture;

/// Future for cursors' advance() and continue()
#[derive(DelegateDebug, new)]
pub(crate) struct IdbCursorAdvancementFuture(IdbRequestFuture);

impl IdbCursorAdvancementFuture {
    fn on_done(res: Result<Option<JsValue>, DomException>) -> Result<bool, DomException> {
        Ok(!res?.nightly_unwrap().is_null())
    }
}

impl Future for IdbCursorAdvancementFuture {
    type Output = Result<bool, DomException>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.do_poll(ctx).map(Self::on_done)
    }
}
