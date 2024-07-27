use crate::database::{Database, VersionChangeEvent};
use crate::error::{Error, UnexpectedDataError};
use internal_macros::generic_bounds;
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

type TClosure = Closure<dyn FnMut(web_sys::IdbVersionChangeEvent) -> Result<(), JsValue> + 'static>;

#[derive(Debug, Default)]
pub(super) enum Status {
    #[default]
    Ok,
    Err(Error),
    Taken,
}

pub(super) struct Listener {
    status: Arc<Mutex<Status>>,
    listener: TClosure,
}

impl From<Status> for crate::Result<()> {
    fn from(status: Status) -> Self {
        match status {
            Status::Ok => Ok(()),
            Status::Err(e) => Err(e),
            Status::Taken => Err(UnexpectedDataError::PollState.into()),
        }
    }
}

impl Status {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }
}

impl Listener {
    #[generic_bounds(upgrade_cb(F))]
    pub fn new_upgrade<F>(callback: F) -> Self {
        let status = Status::new();
        Self {
            status: status.clone(),
            listener: Closure::once(move |evt: web_sys::IdbVersionChangeEvent| {
                let res = Database::from_event(&evt)
                    .and_then(move |db| callback(VersionChangeEvent::new(evt), db));

                Self::handle_result("onupgradeneeded", &status, res)
            }),
        }
    }

    #[generic_bounds(blocked_cb(F))]
    pub fn new_blocked<F>(callback: F) -> Self {
        let status = Status::new();
        Self {
            status: status.clone(),
            listener: Closure::once(move |evt: web_sys::IdbVersionChangeEvent| {
                let res = callback(VersionChangeEvent::new(evt));
                Self::handle_result("onblocked", &status, res)
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
            Err(e) => {
                let js_err = Self::create_error(label, &e);

                if let Ok(mut status) = status.lock() {
                    *status = Status::Err(e);
                }

                Err(js_err.unchecked_into())
            }
        }
    }

    fn create_error<L: Display, E: Display>(label: L, error: E) -> js_sys::Error {
        let msg = format!(
            "An error occurred during an `indexed_db_futures` `{label} event handler: {error}"
        );
        js_sys::Error::new(&msg)
    }

    pub fn take_error(&self) -> crate::Result<()> {
        match self.status.lock() {
            Ok(mut lock) => {
                let status = mem::replace(&mut *lock, Status::Taken);
                drop(lock);
                status.into()
            }
            Err(_) => Err(UnexpectedDataError::PoisonedLock.into()),
        }
    }
}

impl AsRef<js_sys::Function> for Listener {
    fn as_ref(&self) -> &js_sys::Function {
        self.listener.as_ref().unchecked_ref()
    }
}

impl Debug for Listener {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.status.lock() {
            Ok(lock) => Debug::fmt(&*lock, f),
            Err(_) => Debug::fmt(&UnexpectedDataError::PoisonedLock, f),
        }
    }
}
