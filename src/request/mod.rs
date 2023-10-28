//! IDB requests and futures

pub use futures::*;
use idb_open_db_request_ref::*;
pub(crate) use idb_request_ref::*;
pub use open_db_request::*;
pub use request_like::*;
pub use void_open_db_request::*;
pub use void_request::*;

macro_rules! impl_void_request {
    (for $for: ty, raw $raw_ty: ty, ref $ref_ty: ty, fut $fut: ty) => {
        impl $for {
            #[inline]
            pub(crate) fn new(req: $raw_ty) -> Self {
                Self(<$ref_ty>::new(req))
            }
        }

        impl ::std::future::IntoFuture for $for {
            type Output = Result<(), ::web_sys::DomException>;
            type IntoFuture = $crate::request::futures::VoidFuture<$fut>;

            /// Turn the request into a future. This is when event listeners get set.
            fn into_future(self) -> Self::IntoFuture {
                let inner = self.0.into_future(false);
                $crate::request::futures::VoidFuture::new(inner)
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
