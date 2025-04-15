pub(crate) mod listeners;
mod untyped;

use crate::internal_utils::SystemRepr;
use internal_macros::{FutureFromPollUnpinned, StructNameDebug};
use listeners::Listeners;
use sealed::sealed;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

use super::traits::*;
use untyped::UntypedRequest;

/// Alias for [`Request<()>`](Request).
pub type VoidRequest = Request<()>;

/// Future for a [`web-sys` request](web_sys::IdbRequest).
#[derive(StructNameDebug, FutureFromPollUnpinned)]
#[debug(expr(self.as_sys()))]
pub struct Request<T = JsValue> {
    inner: UntypedRequest,
    _marker: PhantomData<T>,
}

impl<T> Request<T> {
    pub(crate) fn new(req: web_sys::IdbRequest) -> Self {
        Self {
            inner: UntypedRequest::Bare(req),
            _marker: PhantomData,
        }
    }
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl<T> crate::internal_utils::SystemRepr for Request<T> {
    type Repr = web_sys::IdbRequest;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        self.inner.as_sys()
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.inner.into_sys()
    }
}

#[sealed]
impl PollUnpinned for Request {
    type Output = crate::Result<JsValue>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self.inner.poll_unpinned(cx) {
            Poll::Ready(Ok(_)) => Poll::Ready(self.as_sys().result().map_err(Into::into)),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

#[sealed]
#[cfg(feature = "cursors")]
impl PollUnpinned for Request<listeners::EventTargetResult> {
    type Output = crate::Result<listeners::EventTargetResult>;

    #[inline]
    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        self.inner.poll_unpinned(cx)
    }
}

#[sealed]
impl PollUnpinned for Request<()> {
    type Output = crate::Result<()>;

    #[inline]
    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        self.inner.poll_unpinned(cx).map(|res| res.map(|_| ()))
    }
}
