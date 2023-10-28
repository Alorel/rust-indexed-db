use fancy_constructor::new;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A future that drops the non-error part of a `Result`
#[derive(new)]
#[new(vis(pub(crate)))]
pub struct VoidFuture<F>(F);

impl<F, S, E> Future for VoidFuture<F>
where
    F: Future<Output = Result<S, E>> + Unpin,
{
    type Output = Result<(), E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx).map(move |_| Ok(()))
    }
}
