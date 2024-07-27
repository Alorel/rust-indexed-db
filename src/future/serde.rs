use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use serde::de::DeserializeOwned;

use crate::internal_utils::StructName;
use crate::internal_utils::SystemRepr;

use super::Request;

/// A request returning a `serde`-deserialisable type.
#[derive(StructName)]
pub struct SerdeRequest<T> {
    base: Request,
    _value: PhantomData<T>,
}

impl<T: DeserializeOwned> SerdeRequest<T> {
    pub(crate) fn new(req: web_sys::IdbRequest) -> Self {
        Self {
            base: Request::jsval(req),
            _value: PhantomData,
        }
    }

    pub(super) fn do_poll(&mut self, cx: &mut Context<'_>) -> Poll<crate::Result<T>> {
        match self.base.do_poll(cx) {
            Poll::Ready(Ok(js)) => {
                Poll::Ready(serde_wasm_bindgen::from_value(js).map_err(Into::into))
            }
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

impl<T> Debug for SerdeRequest<T> {
    struct_name_debug!(inner self, &self.base);
}

impl<T> SystemRepr for SerdeRequest<T> {
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

impl<T: DeserializeOwned + Unpin> Future for SerdeRequest<T> {
    type Output = crate::Result<T>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(cx)
    }
}
