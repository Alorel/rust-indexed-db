use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use wasm_bindgen::{prelude::*, JsCast};

use crate::idb_database::{IdbVersionChangeCallback, IdbVersionChangeEvent};
use crate::internal_utils::{safe_unwrap_option, safe_unwrap_result};

use super::{IdbOpenDbRequestFuture, IdbRequestRef};

#[derive(Debug)]
pub(crate) struct OpenDbRequestListeners {
    request: Weak<IdbRequestRef>,
    on_blocked: Option<IdbVersionChangeCallback>,
    on_upgrade_needed: Option<IdbVersionChangeCallback>,
}

impl OpenDbRequestListeners {
    #[inline]
    fn new(request: Weak<IdbRequestRef>) -> Self {
        Self {
            request,
            on_upgrade_needed: None,
            on_blocked: None,
        }
    }

    pub fn set_on_upgrade_needed<F>(&mut self, callback: Option<F>)
    where
        F: Fn(&IdbVersionChangeEvent) -> Result<(), JsValue> + 'static,
    {
        let base = safe_unwrap_option(self.request.upgrade());
        let req = base.inner_as_idb_request();
        self.on_upgrade_needed = match callback {
            Some(callback) => {
                let callback = IdbVersionChangeEvent::wrap_callback(callback);
                req.set_onupgradeneeded(Some(callback.as_ref().unchecked_ref()));
                Some(callback)
            }
            None => {
                req.set_onupgradeneeded(None);
                None
            }
        };
    }

    pub fn set_on_blocked<F>(&mut self, callback: Option<F>)
    where
        F: Fn(&IdbVersionChangeEvent) -> Result<(), JsValue> + 'static,
    {
        let base = safe_unwrap_option(self.request.upgrade());
        let req = base.inner_as_idb_request();
        self.on_blocked = match callback {
            Some(callback) => {
                let callback = IdbVersionChangeEvent::wrap_callback(callback);
                req.set_onblocked(Some(callback.as_ref().unchecked_ref()));
                Some(callback)
            }
            None => {
                req.set_onblocked(None);
                None
            }
        };
    }

    fn run_drop(&self, has_upgrade_needed: bool, has_blocked: bool) {
        if let Some(req) = self.request.upgrade() {
            let cast_req = req.inner_as_idb_request();
            if has_upgrade_needed {
                cast_req.set_onupgradeneeded(None);
            }
            if has_blocked {
                cast_req.set_onblocked(None);
            }
        }
    }
}

impl Drop for OpenDbRequestListeners {
    fn drop(&mut self) {
        let has_upgrade_needed = self.on_upgrade_needed.is_some();
        let has_blocked = self.on_blocked.is_some();

        if has_upgrade_needed || has_blocked {
            self.run_drop(has_upgrade_needed, has_blocked);
        }
    }
}

#[derive(Debug)]
pub(crate) struct IdbOpenDbRequestRef {
    base: Rc<IdbRequestRef>,
    listeners: OpenDbRequestListeners,
}

impl IdbOpenDbRequestRef {
    pub fn new(inner: web_sys::IdbOpenDbRequest) -> Self {
        let base = Rc::new(IdbRequestRef::new(inner.unchecked_into()));
        let listeners = OpenDbRequestListeners::new(Rc::downgrade(&base));

        Self { base, listeners }
    }

    pub fn into_future(self, read_response: bool) -> IdbOpenDbRequestFuture {
        // We need to take the request out of the Rc to turn it into a future
        let base = safe_unwrap_result(Rc::try_unwrap(self.base)).into_future(read_response);

        // Then we need to re-set the new weak ref
        let mut listeners = self.listeners;
        listeners.request = base.weak_request();

        IdbOpenDbRequestFuture::new(base, listeners)
    }
}

impl Deref for IdbOpenDbRequestRef {
    type Target = OpenDbRequestListeners;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.listeners
    }
}

impl DerefMut for IdbOpenDbRequestRef {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.listeners
    }
}
