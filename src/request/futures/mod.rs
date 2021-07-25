use std::task::{Context, Poll};

use wasm_bindgen::prelude::*;
use web_sys::DomException;

pub use count_future::*;
pub(crate) use idb_open_db_request_future::*;
pub(crate) use idb_request_future::*;
pub use jscast_request_future::*;
pub use optional_jsval_future::*;

macro_rules! impl_result_formatting_struct_constructor {
    () => {
        pub(crate) fn new(
            req: Result<web_sys::IdbRequest, wasm_bindgen::JsValue>,
        ) -> Result<Self, DomException> {
            let base = $crate::request::IdbRequestRef::new(req?).into_future(true);
            Ok(Self(base))
        }
    };
}

macro_rules! impl_result_formatting_future_future {
    ($for: ty, $rsp: ty) => {
        impl std::future::Future for $for {
            type Output = Result<$rsp, DomException>;

            #[inline]
            fn poll(
                self: Pin<&mut Self>,
                ctx: &mut std::task::Context,
            ) -> std::task::Poll<Self::Output> {
                self.poll_with_formatting(ctx)
            }
        }
    };
}

mod count_future;
mod idb_open_db_request_future;
mod idb_request_future;
mod jscast_request_future;
mod optional_jsval_future;

cfg_if::cfg_if! {
    if #[cfg(feature = "cursors")] {
        mod idb_cursor_future;
        mod idb_cursor_with_value_future;
        mod idb_cursor_advancement_future;

        pub(crate) use idb_cursor_advancement_future::*;

        pub use {
            idb_cursor_future::*,
            idb_cursor_with_value_future::*
        };
    }
}

trait ResponseFormattingFuture<T> {
    /// Format the raw response into what the implementing struct expects
    fn format_response(v: Result<Option<JsValue>, DomException>) -> Result<T, DomException>;

    fn inner(&self) -> &IdbRequestFuture;

    /// Poll the inner future and format the ready response
    fn poll_with_formatting(&self, ctx: &Context<'_>) -> Poll<Result<T, DomException>> {
        self.inner().do_poll(ctx).map(Self::format_response)
    }
}
