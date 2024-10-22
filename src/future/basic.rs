use super::Request;
use crate::internal_utils::SystemRepr;
use crate::primitive::{TryFromJs, TryFromJsExt};
use fancy_constructor::new;
use internal_macros::{FutureFromPollUnpinned, StructNameDebug};
use sealed::sealed;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

/// A basic [`Request`] that only performs basic [`JsValue`] conversion.
#[derive(StructNameDebug, FutureFromPollUnpinned, new)]
#[new(vis(), args(req: web_sys::IdbRequest))]
pub struct BasicRequest<T> {
    #[new(val(Request::new(req)))]
    #[debug]
    base: Request,
    map_fn: fn(JsValue) -> crate::Result<T>,
}

impl<T> BasicRequest<T> {
    #[inline]
    pub(crate) fn new_primitive(req: web_sys::IdbRequest) -> Self
    where
        T: TryFromJs,
    {
        Self::new(req, T::from_js_base)
    }

    #[cfg(feature = "serde")]
    #[inline]
    pub(crate) fn new_ser(req: web_sys::IdbRequest) -> Self
    where
        T: crate::serde::DeserialiseFromJs,
    {
        Self::new(req, T::deserialise_from_js)
    }
}

#[sealed]
impl<T> super::PollUnpinned for BasicRequest<T> {
    type Output = crate::Result<T>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self.base.poll_unpinned(cx) {
            Poll::Ready(Ok(js)) => Poll::Ready((self.map_fn)(js)),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl<T> crate::internal_utils::SystemRepr for BasicRequest<T> {
    type Repr = <Request as SystemRepr>::Repr;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        self.base.as_sys()
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.base.into_sys()
    }
}
