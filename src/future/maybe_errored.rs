use std::future;
use std::future::{Future, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use delegate_display::DelegateDebug;

/// A future that may have errored from the start
#[derive(Debug)]
pub struct MaybeErrored<F, E = crate::error::Error>(Inner<F, E>);

#[derive(DelegateDebug)]
#[ddebug(base_bounds)]
enum Inner<F, E> {
    Running(F),
    Errored(Ready<E>),
}

impl<F, E> MaybeErrored<F, E> {
    #[inline]
    pub(crate) fn running(f: F) -> MaybeErrored<F, E> {
        MaybeErrored(Inner::Running(f))
    }

    #[inline]
    pub(crate) fn errored(err: E) -> MaybeErrored<F, E> {
        MaybeErrored(Inner::Errored(future::ready(err)))
    }
}

impl<T, F, E> MaybeErrored<F, E>
where
    F: Future<Output = Result<T, E>> + Unpin,
{
    pub(super) fn do_poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<T, E>> {
        match &mut self.0 {
            Inner::Running(f) => Pin::new(f).poll(cx),
            Inner::Errored(f) => Pin::new(f).poll(cx).map(Err),
        }
    }
}

impl<T, F, E> Future for MaybeErrored<F, E>
where
    F: Future<Output = Result<T, E>> + Unpin,
    E: Unpin,
{
    type Output = Result<T, E>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(cx)
    }
}
