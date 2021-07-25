use std::future::Future;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::DomException;

use crate::idb_database::IdbDatabase;
use crate::internal_utils::safe_unwrap_option;

use super::IdbOpenDbRequestRef;

/// Request for opening an [IdbDatabase]
#[derive(Debug)]
pub struct OpenDbRequest(IdbOpenDbRequestRef);

impl OpenDbRequest {
    #[inline]
    pub(crate) fn new(req: web_sys::IdbOpenDbRequest) -> Self {
        Self(IdbOpenDbRequestRef::new(req))
    }

    fn instantiate(
        raw: Result<Option<JsValue>, DomException>,
    ) -> Result<IdbDatabase, DomException> {
        Ok(IdbDatabase::new(safe_unwrap_option(raw?).unchecked_into()))
    }

    /// Turn the request into a future. This is when event listeners get set.
    pub fn into_future(self) -> impl Future<Output = Result<IdbDatabase, DomException>> {
        let fut = self.0.into_future(true);
        async move { Self::instantiate(fut.await) }
    }
}

impl_idb_open_request_like!(OpenDbRequest);
