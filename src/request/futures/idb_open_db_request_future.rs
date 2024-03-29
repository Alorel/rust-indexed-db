use fancy_constructor::new;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use wasm_bindgen::prelude::*;
use web_sys::DomException;

use super::{super::OpenDbRequestListeners, IdbRequestFuture};

/// Base `IdbOpenDbRequest` future implementation
#[derive(Debug, new)]
#[new(vis(pub(crate)))]
pub struct IdbOpenDbRequestFuture {
    base: IdbRequestFuture,
    _listeners: OpenDbRequestListeners,
}

impl Future for IdbOpenDbRequestFuture {
    type Output = Result<Option<JsValue>, DomException>;

    #[inline]
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.base.do_poll(ctx)
    }
}
