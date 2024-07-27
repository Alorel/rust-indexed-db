use super::{Database, VersionChangeEvent};
use crate::internal_utils::SystemRepr;
use accessory::Accessors;
use tokio::sync::mpsc::{
    channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

type TOnClose = Closure<dyn FnMut()>;
type TOnVersionChange = Closure<dyn FnMut(web_sys::IdbVersionChangeEvent)>;

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
    rx_close: Receiver<()>,
    rx_change: UnboundedReceiver<web_sys::IdbVersionChangeEvent>,
    cb_close: TOnClose,
    cb_version_change: TOnVersionChange,
}

impl VersionChangeListener {
    pub(super) fn new(db: Database) -> crate::Result<Self> {
        let (tx_close, rx_close) = channel(1);
        let (tx_change, rx_change) = unbounded_channel();

        let cb_close = Self::create_on_close(tx_close);
        let cb_version_change = Self::create_on_change(tx_change);

        db.as_sys()
            .add_event_listener_with_callback(EVT_CLOSE, cb_close.as_ref().unchecked_ref())?;
        db.as_sys().add_event_listener_with_callback(
            EVT_CHANGE,
            cb_version_change.as_ref().unchecked_ref(),
        )?;

        Ok(Self {
            db,
            rx_close,
            rx_change,
            cb_close,
            cb_version_change,
        })
    }

    /// Receive the next event.
    ///
    /// Returns `None` if the database gets closed.
    pub async fn recv(&mut self) -> Option<VersionChangeEvent> {
        tokio::select! {
            _ = self.rx_close.recv() => None,
            opt = self.rx_change.recv() => opt.map(VersionChangeEvent::new),
        }
    }

    fn create_on_change(tx: UnboundedSender<web_sys::IdbVersionChangeEvent>) -> TOnVersionChange {
        TOnVersionChange::wrap(Box::new(move |evt| {
            let _ = tx.send(evt);
        }))
    }

    fn create_on_close(tx: Sender<()>) -> TOnClose {
        TOnClose::wrap(Box::new(move || {
            let tx = tx.clone();
            spawn_local(async move {
                let _ = tx.send(()).await;
            });
        }))
    }
}

impl Drop for VersionChangeListener {
    fn drop(&mut self) {
        let _ = self
            .db
            .as_sys()
            .remove_event_listener_with_callback(EVT_CLOSE, self.cb_close.as_ref().unchecked_ref());

        let _ = self.db.as_sys().remove_event_listener_with_callback(
            EVT_CHANGE,
            self.cb_version_change.as_ref().unchecked_ref(),
        );
    }
}

#[cfg(feature = "streams")]
const _: () = {
    use futures_core::Stream;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    impl Stream for VersionChangeListener {
        type Item = VersionChangeEvent;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.rx_change.poll_recv(cx) {
                Poll::Ready(opt) => Poll::Ready(opt.map(VersionChangeEvent::new)),
                Poll::Pending => match self.rx_close.poll_recv(cx) {
                    Poll::Ready(_) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                },
            }
        }
    }
};
