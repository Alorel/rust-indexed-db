use wasm_bindgen::prelude::*;
use web_sys::{DomException, IdbOpenDbRequest, IdbRequest, IdbRequestReadyState};

use crate::idb_database::IdbVersionChangeEvent;

pub(crate) trait IdbRequestLike {
    fn get_error(&self) -> Result<Option<DomException>, JsValue>;
    fn get_result(&self) -> Result<JsValue, JsValue>;
    fn set_on_error(&self, value: Option<&::js_sys::Function>);
    fn set_on_success(&self, value: Option<&::js_sys::Function>);
    fn get_ready_state(&self) -> IdbRequestReadyState;
}

/// Common trait for `IdbOpenDbRequest`s
pub trait IdbOpenDbRequestLike {
    /// Set the callback for the `upgradeneeded` event
    fn set_on_upgrade_needed<F>(&mut self, callback: Option<F>)
    where
        F: Fn(&IdbVersionChangeEvent) -> Result<(), JsValue> + 'static;

    /// Set the callback for the `blocked` event
    fn set_on_blocked<F>(&mut self, callback: Option<F>)
    where
        F: Fn(&IdbVersionChangeEvent) -> Result<(), JsValue> + 'static;
}

macro_rules! impl_request_like {
    ($tgt: ident) => {
        impl IdbRequestLike for $tgt {
            #[inline]
            fn set_on_error(&self, value: Option<&::js_sys::Function>) {
                self.set_onerror(value)
            }

            #[inline]
            fn get_ready_state(&self) -> web_sys::IdbRequestReadyState {
                self.ready_state()
            }

            #[inline]
            fn set_on_success(&self, value: Option<&::js_sys::Function>) {
                self.set_onsuccess(value)
            }

            #[inline]
            fn get_error(&self) -> Result<Option<DomException>, JsValue> {
                self.error()
            }

            #[inline]
            fn get_result(&self) -> Result<JsValue, JsValue> {
                self.result()
            }
        }
    };
}

impl_request_like!(IdbRequest);
impl_request_like!(IdbOpenDbRequest);
