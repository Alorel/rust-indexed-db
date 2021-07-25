use std::rc::Rc;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::DomException;

use super::IdbRequestFuture;

#[derive(Debug)]
pub(crate) struct IdbRequestRef(web_sys::IdbRequest);

impl IdbRequestRef {
    #[inline]
    pub fn new(inner: web_sys::IdbRequest) -> Self {
        Self(inner)
    }

    #[inline]
    pub fn inner(&self) -> &web_sys::IdbRequest {
        &self.0
    }

    #[inline]
    pub fn inner_as_idb_request(&self) -> &web_sys::IdbOpenDbRequest {
        self.inner().unchecked_ref()
    }

    pub fn result(&self) -> Result<JsValue, DomException> {
        match self.inner().result() {
            Ok(v) => Ok(v),
            Err(fallback) => self.error_with_fallback(fallback),
        }
    }

    pub fn error(&self) -> Option<DomException> {
        self.inner().error().ok()?
    }

    fn error_with_fallback(&self, fallback: JsValue) -> Result<JsValue, DomException> {
        Err(self
            .error()
            .unwrap_or_else(move || fallback.unchecked_into()))
    }

    #[inline]
    pub fn rc_to_future(self_rc: Rc<Self>, read_response: bool) -> IdbRequestFuture {
        IdbRequestFuture::new_with_rc(self_rc, read_response)
    }

    #[inline]
    pub fn into_future(self, read_response: bool) -> IdbRequestFuture {
        Self::rc_to_future(Rc::new(self), read_response)
    }
}
