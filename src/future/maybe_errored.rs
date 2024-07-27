use std::task::{Context, Poll};

use crate::error::Error;
use delegate_display::DelegateDebug;
use internal_macros::FutureFromPollUnpinned;

const E_POLLED: &str = "MaybeErrored polled after completion";

/// A Future that may have errored from the start.
#[derive(Debug, FutureFromPollUnpinned)]
pub struct MaybeErrored<F, E = Error>(Inner<F, E>);

#[derive(DelegateDebug)]
enum Inner<F, E> {
    Running(F),
    Errored(Option<E>),
}

impl<F, E> MaybeErrored<F, E> {
    #[inline]
    pub(crate) fn running(f: F) -> MaybeErrored<F, E> {
        MaybeErrored(Inner::Running(f))
    }

    #[inline]
    pub(crate) fn errored(err: E) -> MaybeErrored<F, E> {
        MaybeErrored(Inner::Errored(Some(err)))
    }
}

#[sealed::sealed]
impl<T, F, E> super::PollUnpinned for MaybeErrored<F, E>
where
    F: super::PollUnpinned<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match &mut self.0 {
            Inner::Running(f) => f.poll_unpinned(cx),
            Inner::Errored(f) => Poll::Ready(Err(f.take().expect(E_POLLED))),
        }
    }
}
