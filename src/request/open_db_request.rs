use std::future::IntoFuture;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::DomException;

use crate::idb_database::IdbDatabase;
use crate::internal_utils::NightlyUnwrap;

use super::futures::{MapFuture, TMapFuture};
use super::{IdbOpenDbRequestFuture, IdbOpenDbRequestRef};

/// Request for opening an [`IdbDatabase`]
#[derive(Debug)]
pub struct OpenDbRequest(IdbOpenDbRequestRef);

type FutFn = fn(Result<Option<JsValue>, DomException>) -> Result<IdbDatabase, DomException>;

impl OpenDbRequest {
    #[inline]
    pub(crate) fn new(req: web_sys::IdbOpenDbRequest) -> Self {
        Self(IdbOpenDbRequestRef::new(req))
    }

    fn instantiate(
        raw: Result<Option<JsValue>, DomException>,
    ) -> Result<IdbDatabase, DomException> {
        Ok(IdbDatabase::new(raw?.nightly_unwrap().unchecked_into()))
    }
}

impl IntoFuture for OpenDbRequest {
    type Output = Result<IdbDatabase, DomException>;
    type IntoFuture = MapFuture<IdbOpenDbRequestFuture, FutFn>;

    /// Turn the request into a future. This is when event listeners get set.
    fn into_future(self) -> Self::IntoFuture {
        IntoFuture::into_future(self.0).map::<_, FutFn>(Self::instantiate)
    }
}

impl_idb_open_request_like!(OpenDbRequest);
