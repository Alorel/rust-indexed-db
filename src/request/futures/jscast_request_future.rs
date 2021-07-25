use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::DomException;

use crate::internal_utils::safe_unwrap_option;

use super::{super::IdbRequestRef, IdbRequestFuture, ResponseFormattingFuture};

/// A [Future] that casts the IdbRequest response to the given type
#[derive(Debug)]
pub struct JsCastRequestFuture<T: JsCast> {
    inner: IdbRequestFuture,
    _cast: PhantomData<T>,
}

impl<T: JsCast> JsCastRequestFuture<T> {
    pub(crate) fn new(req: Result<web_sys::IdbRequest, JsValue>) -> Result<Self, DomException> {
        let out = Self {
            inner: IdbRequestRef::new(req?).into_future(true),
            _cast: PhantomData::default(),
        };
        Ok(out)
    }
}

impl<T: JsCast> ResponseFormattingFuture<T> for JsCastRequestFuture<T> {
    fn format_response(v: Result<Option<JsValue>, DomException>) -> Result<T, DomException> {
        Ok(safe_unwrap_option(v?).unchecked_into())
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
