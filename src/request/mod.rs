//! IDB requests and futures

use std::future::Future;

pub use futures::*;
use idb_open_db_request_ref::*;
pub(crate) use idb_request_ref::*;
pub use open_db_request::*;
pub use request_like::*;
pub use void_open_db_request::*;
pub use void_request::*;

macro_rules! impl_void_request {
    ($for: ty, $raw_ty: ty, $ref_ty: ty) => {
        impl $for {
            #[inline]
            pub(crate) fn new(req: $raw_ty) -> Self {
                Self(<$ref_ty>::new(req))
            }

            /// Turn the request into a future. This is when event listeners get set.
            #[inline]
            pub fn into_future(
                self,
            ) -> impl std::future::Future<Output = Result<(), web_sys::DomException>> {
                $crate::request::await_void_future(self.0.into_future(false))
            }
        }
    };
}

macro_rules! impl_idb_open_request_like {
    ($for: ty) => {
        impl crate::request::IdbOpenDbRequestLike for $for {
            #[inline]
            fn set_on_upgrade_needed<F>(&mut self, callback: Option<F>)
            where
                F: Fn(
                        &crate::idb_database::IdbVersionChangeEvent,
                    ) -> Result<(), wasm_bindgen::JsValue>
                    + 'static,
            {
                self.0.set_on_upgrade_needed(callback);
            }

            #[inline]
            fn set_on_blocked<F>(&mut self, callback: Option<F>)
            where
                F: Fn(
                        &crate::idb_database::IdbVersionChangeEvent,
                    ) -> Result<(), wasm_bindgen::JsValue>
                    + 'static,
            {
                self.0.set_on_blocked(callback);
            }
        }
    };
}

mod idb_open_db_request_ref;
mod idb_request_ref;
mod open_db_request;
mod request_like;
mod void_open_db_request;
mod void_request;

mod futures;

async fn await_void_future<T: Future<Output = Result<S, E>>, S, E>(fut: T) -> Result<(), E> {
    fut.await?;
    Ok(())
}
