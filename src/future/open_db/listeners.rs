use super::Listener;
use derive_more::Debug;
use smallvec::SmallVec;

type Req = web_sys::IdbOpenDbRequest;
type CleanupFn = fn(&Req);

#[derive(Debug)]
pub(super) struct Listeners {
    listeners: SmallVec<[Listener; 2]>,

    #[debug(skip)]
    cleanup_fn: CleanupFn,
}

impl Listeners {
    pub fn with_block(req: &Req, listener: Listener) -> Self {
        #[inline]
        fn cleanup_fn(req: &Req) {
            req.set_onblocked(None);
        }

        req.set_onblocked(Some(listener.as_ref()));

        Self::with_one(listener, cleanup_fn)
    }

    pub fn with_on_upgrade_needed(req: &Req, listener: Listener) -> Self {
        #[inline]
        fn cleanup_fn(req: &Req) {
            req.set_onupgradeneeded(None);
        }

        req.set_onupgradeneeded(Some(listener.as_ref()));

        Self::with_one(listener, cleanup_fn)
    }

    pub fn with_both(req: &Req, blocked: Listener, upgrade: Listener) -> Self {
        fn cleanup_fn(req: &Req) {
            req.set_onblocked(None);
            req.set_onupgradeneeded(None);
        }

        req.set_onblocked(Some(blocked.as_ref()));
        req.set_onupgradeneeded(Some(upgrade.as_ref()));

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

    fn with_one(listener: Listener, cleanup_fn: CleanupFn) -> Self {
        Self {
            listeners: {
                let mut out = SmallVec::new();
                out.push(listener);
                out
            },
            cleanup_fn,
        }
    }

    pub fn take_error(&self) -> crate::Result<()> {
        for listener in &self.listeners {
            listener.take_error()?;
        }

        Ok(())
    }

    #[inline]
    pub fn drop_listeners(&self, req: &Req) {
        (self.cleanup_fn)(req);
    }
}
