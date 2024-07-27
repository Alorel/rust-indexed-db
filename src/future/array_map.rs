use crate::error::{Error, SimpleValueError};
use crate::iter::ArrayMapIter;
use fancy_constructor::new;
use sealed::sealed;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

/// A future returning an [`ArrayMapIter`].
#[derive(new)]
#[new(vis(pub(crate)))]
pub struct ArrayMapFuture<T, F> {
    src: F,
    constructor: fn(js_sys::Array) -> ArrayMapIter<T>,
}

impl<T, F> ArrayMapFuture<T, F> {
    fn on_polled<V, E>(&self, poll: Poll<Result<V, E>>) -> Poll<crate::Result<ArrayMapIter<T>>>
    where
        V: JsCast,
        E: Into<Error>,
    {
        match poll {
            Poll::Ready(Ok(js)) => Poll::Ready(match js.dyn_into::<js_sys::Array>() {
                Ok(arr) => Ok((self.constructor)(arr)),
                Err(js) => Err(SimpleValueError::DynCast(js.unchecked_into()).into()),
            }),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
        }
    }
}

#[sealed]
impl<T, F, V, E> super::PollUnpinned for ArrayMapFuture<T, F>
where
    F: super::PollUnpinned<Output = Result<V, E>>,
    V: JsCast,
    E: Into<Error>,
{
    type Output = crate::Result<ArrayMapIter<T>>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        let poll = self.src.poll_unpinned(cx);
        self.on_polled(poll)
    }
}

impl<T, F, V, E> Future for ArrayMapFuture<T, F>
where
    F: Future<Output = Result<V, E>> + Unpin,
    V: JsCast,
    E: Into<Error>,
{
    type Output = crate::Result<ArrayMapIter<T>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let poll = Pin::new(&mut self.src).poll(cx);
        self.on_polled(poll)
    }
}
