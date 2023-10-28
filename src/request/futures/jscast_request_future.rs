use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::DomException;

use crate::internal_utils::NightlyUnwrap;

use super::{super::IdbRequestRef, IdbRequestFuture, ResponseFormattingFuture};

/// A [Future] that casts the `IdbRequest` response to the given type
#[derive(Debug)]
pub struct JsCastRequestFuture<T: JsCast> {
    inner: IdbRequestFuture,
    _cast: PhantomData<T>,
}

impl<T: JsCast> JsCastRequestFuture<T> {
    pub(crate) fn new(req: Result<web_sys::IdbRequest, JsValue>) -> Result<Self, DomException> {
        let out = Self {
            inner: IntoFuture::into_future(IdbRequestRef::new(req?)),
            _cast: PhantomData,
        };
        Ok(out)
    }
}

impl<T: JsCast> ResponseFormattingFuture<T> for JsCastRequestFuture<T> {
    fn format_response(v: Result<Option<JsValue>, DomException>) -> Result<T, DomException> {
        Ok(v?.nightly_unwrap().unchecked_into())
    }

    #[inline]
    fn inner(&self) -> &IdbRequestFuture {
        &self.inner
    }
}

impl<T: JsCast> Future for JsCastRequestFuture<T> {
    type Output = Result<T, DomException>;

    #[inline]
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_with_formatting(ctx)
    }
}
