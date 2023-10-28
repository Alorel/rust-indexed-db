use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct MapFuture<Fut, M> {
    fut: Fut,
    mapper: M,
}

impl<Fut, M, O> Future for MapFuture<Fut, M>
where
    Fut: Future + Unpin,
    M: Unpin + Fn(Fut::Output) -> O,
{
    type Output = O;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.fut).poll(cx).map(&self.mapper)
    }
}

pub(crate) trait TMapFuture: Future {
    fn map<O, M>(self, mapper: M) -> MapFuture<Self, M>
    where
        M: Fn(Self::Output) -> O,
        Self: Sized;
}

impl<F: Future> TMapFuture for F {
    fn map<O, M>(self, mapper: M) -> MapFuture<Self, M>
    where
        M: Fn(Self::Output) -> O,
    {
        MapFuture { fut: self, mapper }
    }
}
