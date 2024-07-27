use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::SimpleValueError;
use crate::internal_utils::SystemRepr;

use super::Request;

/// A future that resolves to a number
#[derive(Debug)]
pub struct CountRequest(Request);

impl CountRequest {
    pub(crate) fn new(req: web_sys::IdbRequest) -> Self {
        Self(Request::jsval(req))
    }

    pub(super) fn do_poll(&mut self, cx: &mut Context<'_>) -> Poll<crate::Result<u32>> {
        match self.0.do_poll(cx) {
            Poll::Ready(Ok(js)) => Poll::Ready(match js.as_f64() {
                Some(v) => Ok(v as u32),
                None => Err(SimpleValueError::NotANumber(js).into()),
            }),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Future for CountRequest {
    type Output = crate::Result<u32>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(cx)
    }
}

impl SystemRepr for CountRequest {
    type Repr = <Request as SystemRepr>::Repr;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        self.0.as_sys()
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.0.into_sys()
    }
}
