mod listener;
mod listeners;

use fancy_constructor::new;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

use internal_macros::{generic_bounds, FutureFromPollUnpinned};
use listener::Listener;
use listeners::Listeners;

use super::traits::*;
use super::VoidRequest;
use crate::database::Database;
use crate::error::Error;
use crate::internal_utils::SystemRepr;

/// Future for opening a database.
#[derive(Debug, new, FutureFromPollUnpinned)]
#[new(vis())]
pub struct OpenDbRequest {
    req: VoidRequest,
    listeners: Listeners,
}

impl OpenDbRequest {
    #[generic_bounds(upgrade_cb(F))]
    pub(crate) fn with_upgrade<F>(req: VoidRequest, on_upgrade_needed: F) -> Self {
        let listener = Listener::new_upgrade(on_upgrade_needed);
        let idb_req = req.as_sys().unchecked_ref();
        let listeners = Listeners::with_on_upgrade_needed(idb_req, listener);

        Self::new(req, listeners)
    }

    #[generic_bounds(blocked_cb(F))]
    pub(crate) fn with_block<F>(req: VoidRequest, on_blocked: F) -> Self {
        let listener = Listener::new_blocked(on_blocked);
        let idb_req = req.as_sys().unchecked_ref();
        let listeners = Listeners::with_block(idb_req, listener);

        Self::new(req, listeners)
    }

    #[generic_bounds(upgrade_cb(U), blocked_cb(B))]
    pub(crate) fn with_both<B, U>(req: VoidRequest, on_blocked: B, on_upgrade_needed: U) -> Self {
        let on_blocked = Listener::new_blocked(on_blocked);
        let on_upgrade_needed = Listener::new_upgrade(on_upgrade_needed);
        let idb_req = req.as_sys().unchecked_ref();
        let listeners = Listeners::with_both(idb_req, on_blocked, on_upgrade_needed);

        Self::new(req, listeners)
    }

    pub(crate) fn bare(req: VoidRequest) -> Self {
        Self::new(req, Listeners::with_neither())
    }

    fn take_ok(&self) -> crate::OpenDbResult<Database> {
        self.listeners.take_error()?;
        Database::from_req(self.as_sys()).map_err(Into::into)
    }

    fn take_err<V>(&self, err_thrown: Error) -> crate::OpenDbResult<V> {
        self.listeners.take_error()?;
        Err(err_thrown.into())
    }
}

#[::sealed::sealed]
impl PollUnpinned for OpenDbRequest {
    type Output = crate::OpenDbResult<Database>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self.req.poll_unpinned(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(self.take_ok()),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(self.take_err(e)),
        }
    }
}

impl Drop for OpenDbRequest {
    fn drop(&mut self) {
        self.listeners.drop_listeners(self.as_sys());
    }
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for OpenDbRequest {
    type Repr = web_sys::IdbOpenDbRequest;

    fn as_sys(&self) -> &Self::Repr {
        self.req.as_sys().unchecked_ref()
    }

    fn into_sys(self) -> Self::Repr {
        self.as_sys().clone()
    }
}
