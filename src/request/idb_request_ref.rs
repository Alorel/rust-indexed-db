use std::future::{Future, IntoFuture};
use std::rc::Rc;

use accessory::Accessors;
use fancy_constructor::new;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::DomException;

use super::IdbRequestFuture;

#[derive(Debug, new, Accessors)]
pub(crate) struct IdbRequestRef {
    #[access(get)]
    inner: web_sys::IdbRequest,
}

impl IdbRequestRef {
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
    pub(crate) fn into_future(self, read_response: bool) -> IdbRequestFuture {
        Self::rc_to_future(Rc::new(self), read_response)
    }
}

impl IntoFuture for IdbRequestRef {
    type Output = <IdbRequestFuture as Future>::Output;
    type IntoFuture = IdbRequestFuture;

    #[inline]
    fn into_future(self) -> Self::IntoFuture {
        Self::rc_to_future(Rc::new(self), true)
    }
}
