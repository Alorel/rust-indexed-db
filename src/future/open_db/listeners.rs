use super::OpenDbListener;
use derive_more::Debug;
use smallvec::SmallVec;

#[cfg(feature = "async-upgrade")]
use std::task::{Context, Poll};

type Req = web_sys::IdbOpenDbRequest;
type CleanupFn = fn(&Req);

#[derive(Debug)]
pub(crate) struct Listeners {
    listeners: SmallVec<[OpenDbListener; 2]>,

    #[debug(skip)]
    cleanup_fn: CleanupFn,
}

impl Listeners {
    pub(super) fn take_error(&self) -> crate::Result<()> {
        for listener in &self.listeners {
            listener.take_error()?;
        }

        Ok(())
    }

    #[inline]
    pub(super) fn drop_listeners(&self, req: &Req) {
        (self.cleanup_fn)(req);
    }
}

impl Listeners {
    pub fn with_block(req: &Req, listener: OpenDbListener) -> Self {
        #[inline]
        fn cleanup_fn(req: &Req) {
            req.set_onblocked(None);
        }

        req.set_onblocked(Some(listener.as_fn()));

        Self::with_one(listener, cleanup_fn)
    }

    pub fn with_on_upgrade_needed(req: &Req, listener: OpenDbListener) -> Self {
        #[inline]
        fn cleanup_fn(req: &Req) {
            req.set_onupgradeneeded(None);
        }

        req.set_onupgradeneeded(Some(listener.as_fn()));

        Self::with_one(listener, cleanup_fn)
    }

    pub fn with_both(req: &Req, blocked: OpenDbListener, upgrade: OpenDbListener) -> Self {
        fn cleanup_fn(req: &Req) {
            req.set_onblocked(None);
            req.set_onupgradeneeded(None);
        }

        req.set_onblocked(Some(blocked.as_fn()));
        req.set_onupgradeneeded(Some(upgrade.as_fn()));

        Self {
            listeners: SmallVec::from_buf([blocked, upgrade]),
            cleanup_fn,
        }
    }

    pub fn with_neither() -> Self {
        #[inline]
        fn cleanup_fn(_: &Req) {}

        Self {
            listeners: SmallVec::new(),
            cleanup_fn,
        }
    }

    fn with_one(listener: OpenDbListener, cleanup_fn: CleanupFn) -> Self {
        Self {
            listeners: {
                let mut out = SmallVec::new();
                out.push(listener);
                out
            },
            cleanup_fn,
        }
    }
}

#[cfg(feature = "async-upgrade")]
#[sealed::sealed]
impl crate::future::PollUnpinned for Listeners {
    type Output = crate::Result<()>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        for listener in &mut self.listeners {
            match listener.poll_unpinned(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(Ok(()))
    }
}
