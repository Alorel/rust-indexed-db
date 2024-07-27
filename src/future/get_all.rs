use super::{ArrayMapFuture, BasicRequest};
use crate::iter::GetAllPrimitiveIter;
use crate::primitive::TryFromJs;

#[cfg(feature = "serde")]
use {crate::iter::GetAllSerdeIter, crate::serde::DeserialiseFromJs};

type BaseReq = BasicRequest<js_sys::Array>;

/// A [`Future`](std::future::Future) resolving to an [`Array`](js_sys::Array) of primitives.
pub type GetAllPrimitiveRequest<T> = ArrayMapFuture<T, BaseReq>;

/// A [`Future`](std::future::Future) resolving to an [`Array`](js_sys::Array)
/// of [deserialisables](::serde::Deserialize).
#[cfg(feature = "serde")]
pub type GetAllSerdeRequest<T> = ArrayMapFuture<T, BaseReq>;

impl<T: TryFromJs> GetAllPrimitiveRequest<T> {
    pub(crate) fn get_all_primitive(req: web_sys::IdbRequest) -> Self {
        let req = BasicRequest::new_primitive(req);
        Self::new(req, GetAllPrimitiveIter::get_all_primitive)
    }
}

#[cfg(feature = "serde")]
impl<T: DeserialiseFromJs> GetAllSerdeRequest<T> {
    pub(crate) fn get_all_serde(req: web_sys::IdbRequest) -> Self {
        let req = BasicRequest::new_primitive(req);
        Self::new(req, GetAllSerdeIter::get_all_serde)
    }
}
