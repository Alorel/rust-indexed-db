use crate::database::{Database, VersionChangeEvent};
use crate::error::{Error, UnexpectedDataError};
use crate::transaction::{OnTransactionDrop, Transaction};
use internal_macros::generic_bounds;
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

type TClosure = Closure<dyn FnMut(web_sys::IdbVersionChangeEvent) -> Result<(), JsValue> + 'static>;

const LBL_UPGRADE: &str = "onupgradeneeded";
const LBL_BLOCKED: &str = "onblocked";

#[derive(Debug, Default)]
pub(super) enum Status {
    #[default]
    Ok,
    #[cfg(feature = "async-upgrade")]
    Pending,
    Err(Error),
    Taken,
}

/// A Open DB request event listener.
pub struct OpenDbListener {
    status: Arc<Mutex<Status>>,
    #[cfg(feature = "async-upgrade")]
    async_notify: tokio::sync::mpsc::UnboundedReceiver<()>,
    listener: TClosure,
}

impl Status {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }
}

impl From<Status> for crate::Result<()> {
    fn from(status: Status) -> Self {
        match status {
            Status::Ok => Ok(()),
            Status::Err(e) => Err(e),
            Status::Taken => Err(UnexpectedDataError::PollState.into()),
            #[cfg(feature = "async-upgrade")]
            Status::Pending => Err(UnexpectedDataError::PollState.into()),
        }
    }
}

impl OpenDbListener {
    #[generic_bounds(upgrade_cb(F))]
    pub(crate) fn new_upgrade<F>(callback: F) -> Self {
        let status = Status::new();
        Self {
            status: status.clone(),
            #[cfg(feature = "async-upgrade")]
            async_notify: Self::fake_rx(),
            listener: Closure::once(move |evt: web_sys::IdbVersionChangeEvent| {
                let res = Database::from_event(&evt).and_then(|db| {
                    Transaction::from_raw_version_change_event(&db, &evt).and_then(|mut tx| {
                        callback(VersionChangeEvent::new(evt), &tx).inspect(|_| {
                            // If the callback succeeded, we want to ensure that
                            // the transaction is committed when dropped and not
                            // aborted.
                            tx.on_drop(OnTransactionDrop::Commit);
                        })
                    })
                });
                Self::handle_result(LBL_UPGRADE, &status, res)
            }),
        }
    }

    #[generic_bounds(blocked_cb(F))]
    pub(crate) fn new_blocked<F>(callback: F) -> Self {
        let status = Status::new();
        Self {
            status: status.clone(),
            #[cfg(feature = "async-upgrade")]
            async_notify: Self::fake_rx(),
            listener: Closure::once(move |evt: web_sys::IdbVersionChangeEvent| {
                let res = callback(VersionChangeEvent::new(evt));
                Self::handle_result(LBL_BLOCKED, &status, res)
            }),
        }
    }

    fn handle_result<L>(
        label: L,
        status: &Mutex<Status>,
        res: crate::Result<()>,
    ) -> Result<(), JsValue>
    where
        L: Display,
    {
        match res {
            Ok(()) => Ok(()),
            Err(e) => Self::handle_error_result(label, status, e),
        }
    }

    fn handle_error_result<L>(label: L, status: &Mutex<Status>, e: Error) -> Result<(), JsValue>
    where
        L: Display,
    {
        let js_err = Self::create_error(&label, &e);
        let _ = Self::set_status(status, Status::Err(e), label);

        Err(js_err.unchecked_into())
    }

    fn create_error<L: Display, E: Display>(label: L, error: E) -> js_sys::Error {
        let msg = format!(
            "An error occurred during an `indexed_db_futures` `{label} event handler: {error}"
        );
        js_sys::Error::new(&msg)
    }

    fn set_status<L>(mutex: &Mutex<Status>, status: Status, label: L) -> Result<(), JsValue>
    where
        L: Display,
    {
        match mutex.lock() {
            Ok(mut guard) => {
                *guard = status;
                Ok(())
            }
            Err(e) => Err(Self::create_error(label, &e).unchecked_into()),
        }
    }

    pub(super) fn take_error(&self) -> crate::Result<()> {
        self.take_status()?.into()
    }

    fn take_status(&self) -> crate::Result<Status> {
        Ok(mem::replace(&mut *self.status.lock()?, Status::Taken))
    }

    pub(super) fn as_fn(&self) -> &js_sys::Function {
        self.listener.as_ref().unchecked_ref()
    }
}

impl Debug for OpenDbListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.status.lock() {
            Ok(lock) => Debug::fmt(&*lock, f),
            Err(_) => Debug::fmt(&UnexpectedDataError::PoisonedLock, f),
        }
    }
}

#[cfg(feature = "async-upgrade")]
const _: () = {
    use crate::future::PollUnpinned;
    use std::task::{Context, Poll};

    impl OpenDbListener {
        fn fake_rx() -> tokio::sync::mpsc::UnboundedReceiver<()> {
            tokio::sync::mpsc::unbounded_channel().1
        }

        #[generic_bounds(upgrade_async_cb(Fn))]
        pub(crate) fn new_upgrade_fut<Fn>(callback: Fn) -> Self {
            let status = Status::new();
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            Self {
                status: status.clone(),
                async_notify: rx,
                listener: Closure::once(move |evt: web_sys::IdbVersionChangeEvent| {
                    let db = match Database::from_event(&evt) {
                        Ok(db) => db,
                        Err(e) => return Self::handle_error_result(LBL_UPGRADE, &status, e),
                    };

                    Self::set_status(&status, Status::Pending, LBL_UPGRADE)?;
                    let fut = callback(VersionChangeEvent::new(evt), db);

                    wasm_bindgen_futures::spawn_local(async move {
                        let result = match fut.await {
                            Ok(()) => Status::Ok,
                            Err(e) => Status::Err(e),
                        };
                        let _ = Self::set_status(&status, result, LBL_UPGRADE);
                        let _ = tx.send(());
                    });

                    Ok(())
                }),
            }
        }

        fn poll_rx(&mut self, cx: &mut Context<'_>) -> Poll<crate::Result<()>> {
            match self.async_notify.poll_recv(cx) {
                Poll::Ready(Some(())) => {
                    self.async_notify.close();
                    self.poll_unpinned(cx)
                }
                Poll::Pending => Poll::Pending,
                Poll::Ready(None) => Poll::Ready(Err(UnexpectedDataError::ChannelDropped.into())),
            }
        }
    }

    #[sealed::sealed]
    #[allow(unused_qualifications)]
    impl crate::future::PollUnpinned for OpenDbListener {
        type Output = crate::Result<()>;

        fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
            match self.status.lock() {
                Ok(mut lock) => match *lock {
                    Status::Ok => return Poll::Ready(Ok(())),
                    Status::Err(_) => {
                        match mem::replace(&mut *lock, Status::Taken) {
                            Status::Err(e) => return Poll::Ready(Err(e)),
                            _ => unreachable!(),
                        };
                    }
                    Status::Pending => {}
                    Status::Taken => {
                        return Poll::Ready(Err(UnexpectedDataError::PollState.into()))
                    }
                },
                Err(e) => return Poll::Ready(Err(e.into())),
            }

            self.poll_rx(cx)
        }
    }
};
