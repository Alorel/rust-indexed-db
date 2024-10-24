mod listener;
mod listeners;

use cfg_if::cfg_if;
use fancy_constructor::new;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

use internal_macros::FutureFromPollUnpinned;
pub use listener::OpenDbListener;
use listeners::Listeners;

use super::traits::*;
use super::VoidRequest;
use crate::database::Database;
use crate::error::{Error, UnexpectedDataError};
use crate::internal_utils::SystemRepr;

/// Future for opening a database.
#[derive(Debug, new, FutureFromPollUnpinned)]
#[new(vis())]
pub struct OpenDbRequest {
    req: VoidRequest,
    listeners: Listeners,

    #[new(val(Self::phase_poll_req))]
    polling_fn: fn(&mut Self, &mut Context) -> Poll<crate::OpenDbResult<Database>>,
}

impl OpenDbRequest {
    pub(crate) fn with_upgrade(req: VoidRequest, on_upgrade_needed: OpenDbListener) -> Self {
        let idb_req = req.as_sys().unchecked_ref();
        let listeners = Listeners::with_on_upgrade_needed(idb_req, on_upgrade_needed);

        Self::new(req, listeners)
    }

    pub(crate) fn with_block(req: VoidRequest, on_blocked: OpenDbListener) -> Self {
        let idb_req = req.as_sys().unchecked_ref();
        let listeners = Listeners::with_block(idb_req, on_blocked);

        Self::new(req, listeners)
    }

    pub(crate) fn with_both(
        req: VoidRequest,
        on_blocked: OpenDbListener,
        on_upgrade_needed: OpenDbListener,
    ) -> Self {
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

impl OpenDbRequest {
    fn phase_poll_req(&mut self, cx: &mut Context) -> Poll<crate::OpenDbResult<Database>> {
        match self.req.poll_unpinned(cx) {
            Poll::Ready(Ok(())) => {
                cfg_if! {
                    if #[cfg(feature = "async-upgrade")] {
                        self.polling_fn = Self::phase_poll_listeners;
                        self.phase_poll_listeners(cx)
                    } else {
                        self.polling_fn = Self::phase_done;
                        Poll::Ready(self.take_ok())
                    }
                }
            }
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => {
                self.polling_fn = Self::phase_done;
                Poll::Ready(self.take_err(e))
            }
        }
    }

    #[cfg(feature = "async-upgrade")]
    fn phase_poll_listeners(&mut self, cx: &mut Context) -> Poll<crate::OpenDbResult<Database>> {
        match self.listeners.poll_unpinned(cx) {
            Poll::Ready(Ok(())) => {
                self.polling_fn = Self::phase_done;
                Poll::Ready(self.take_ok())
            }
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => {
                self.polling_fn = Self::phase_done;
                Poll::Ready(self.take_err(e))
            }
        }
    }

    #[inline]
    #[allow(clippy::unused_self)]
    fn phase_done(&mut self, _: &mut Context) -> Poll<crate::OpenDbResult<Database>> {
        Poll::Ready(Err(UnexpectedDataError::PollState.into()))
    }
}

#[::sealed::sealed]
impl PollUnpinned for OpenDbRequest {
    type Output = crate::OpenDbResult<Database>;

    #[inline]
    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        (self.polling_fn)(self, cx)
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
