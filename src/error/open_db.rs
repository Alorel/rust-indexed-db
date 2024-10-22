use wasm_bindgen::prelude::*;

use super::{DomException, Error, UnexpectedDataError};

/// Error opening a database.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum OpenDbError {
    /// Version cannot be zero.
    #[error("Version cannot be zero")]
    VersionZero,

    /// Unsupported environment.
    #[error("Unsupported environment")]
    UnsupportedEnvironment,

    /// The `indexedDB` getter returned `null` or `undefined`.
    #[error("The `indexedDB` getter returned `null` or `undefined`")]
    NullFactory,

    /// A forwarded base error.
    #[error(transparent)]
    Base(#[from] Error),
}

fwd_from!(DomException, Error > OpenDbError);
fwd_from!(web_sys::DomException, Error > OpenDbError);
fwd_from!(js_sys::Error, Error > OpenDbError);
fwd_from!(JsValue, Error > OpenDbError);
fwd_from!(UnexpectedDataError, Error > OpenDbError);
