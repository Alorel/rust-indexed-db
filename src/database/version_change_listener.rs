use super::{Database, VersionChangeEvent};
use crate::internal_utils::SystemRepr;
use accessory::Accessors;
use std::task::{Context, Poll};
use wasm_evt_listener::Listener as EvtListener;

const EVT_CLOSE: &str = "close";
const EVT_CHANGE: &str = "versionchange";

/// A [`versionchange`](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/versionchange_event) event
/// listener.
///
/// Created via [`Database::version_changes`].
#[derive(Accessors)]
pub struct VersionChangeListener {
    /// The database associated with this listener
    #[access(get)]
    db: Database,
    on_close: EvtListener,
    on_change: EvtListener<web_sys::IdbVersionChangeEvent>,
}

impl VersionChangeListener {
    pub(super) fn new(db: Database) -> crate::Result<Self> {
        let on_close = EvtListener::builder().build()?;
        let on_change = EvtListener::builder().build()?;

        on_close.add_to(EVT_CLOSE, db.as_sys())?;
        on_change.add_to(EVT_CHANGE, db.as_sys())?;

        Ok(Self {
            db,
            on_close,
            on_change,
        })
    }

    /// Poll for the next event.
    ///
    /// Returns `None` if the database gets closed.
    pub fn poll_recv(&mut self, cx: &mut Context) -> Poll<Option<VersionChangeEvent>> {
        match self.on_change.poll_recv(cx) {
            Poll::Pending => match self.on_close.poll_recv(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => {
                    self.close();
                    Poll::Ready(None)
                }
            },
            Poll::Ready(evt) => Poll::Ready(Some(VersionChangeEvent::new(evt))),
        }
    }

    /// Receive the next event.
    ///
    /// Returns `None` if the database gets closed.
    pub async fn recv(&mut self) -> Option<VersionChangeEvent> {
        tokio::select! {
            _ = self.on_close.recv() => {
                self.close();
                None
            },
            evt = self.on_change.recv() => Some(VersionChangeEvent::new(evt)),
        }
    }

    /// Check if a `versionchange` event got emitted, return it if so.
    pub fn try_recv(&mut self) -> Option<VersionChangeEvent> {
        Some(VersionChangeEvent::new(self.on_change.try_recv()?))
    }

    fn close(&mut self) {
        self.on_close.close();
        self.on_change.close();
    }
}

impl Drop for VersionChangeListener {
    fn drop(&mut self) {
        let _ = self.on_close.rm_from(EVT_CLOSE, self.db.as_sys());
        let _ = self.on_change.rm_from(EVT_CHANGE, self.db.as_sys());
    }
}

#[cfg(feature = "streams")]
const _: () = {
    use futures_core::{FusedStream, Stream};
    use std::pin::Pin;
    use std::task::{Context, Poll};

    impl Stream for VersionChangeListener {
        type Item = VersionChangeEvent;

        #[inline]
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.poll_recv(cx)
        }
    }

    impl FusedStream for VersionChangeListener {
        fn is_terminated(&self) -> bool {
            self.on_close.is_terminated() && self.on_change.is_terminated()
        }
    }
};
