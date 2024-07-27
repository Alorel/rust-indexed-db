use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::error::SimpleValueError;
use crate::iter::ListDatabasesIter;

/// A [`Future`] returned by [`DBFactory::databases`](crate::DBFactory::databases).
#[derive(Debug)]
pub struct ListDatabasesFuture(JsFuture);

impl ListDatabasesFuture {
    pub(crate) fn new(promise: js_sys::Promise) -> Self {
        Self(promise.into())
    }
}

impl Future for ListDatabasesFuture {
    type Output = crate::Result<ListDatabasesIter>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Ready(Ok(arr)) => Poll::Ready(match arr.dyn_into::<js_sys::Array>() {
                Ok(arr) => Ok(ListDatabasesIter::new(arr)),
                Err(e) => Err(SimpleValueError::DynCast(e).into()),
            }),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
        }
    }
}
