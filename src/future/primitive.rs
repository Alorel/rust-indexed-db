use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::internal_utils::{StructName, SystemRepr};
use crate::FromJs;

use super::Request;

/// Future that resolves to a JS primitive.
#[derive(StructName)]
pub struct PrimitiveRequest<T> {
    base: Request,
    _value: PhantomData<T>,
}

impl<T: FromJs> PrimitiveRequest<T> {
    pub(crate) fn new(req: web_sys::IdbRequest) -> Self {
        Self {
            base: Request::jsval(req),
            _value: PhantomData,
        }
    }

    pub(super) fn do_poll(&mut self, cx: &mut Context<'_>) -> Poll<crate::Result<T>> {
        match self.base.do_poll(cx) {
            Poll::Ready(Ok(js)) => Poll::Ready(<T as FromJs>::from_js(js).map_err(Into::into)),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

impl<T: FromJs + Unpin> Future for PrimitiveRequest<T> {
    type Output = crate::Result<T>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(cx)
    }
}

impl<T> Debug for PrimitiveRequest<T> {
    struct_name_debug!(inner self, &self.base);
}

impl<T> SystemRepr for PrimitiveRequest<T> {
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
