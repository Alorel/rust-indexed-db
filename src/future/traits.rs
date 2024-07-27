use sealed::sealed;
use std::task::{Context, Poll};

/// A type that's pollable without needing to pin it.
#[sealed(pub(crate))]
pub trait PollUnpinned {
    /// Polling result type.
    type Output;

    /// Poll the type without pinning it.
    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output>;
}

#[sealed]
impl<T: PollUnpinned> PollUnpinned for &mut T {
    type Output = T::Output;

    #[inline]
    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        T::poll_unpinned(self, cx)
    }
}

#[sealed]
impl<T: PollUnpinned> PollUnpinned for Box<T> {
    type Output = T::Output;

    #[inline]
    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        T::poll_unpinned(self, cx)
    }
}
