use std::fmt::{Display, Formatter};

use wasm_bindgen::prelude::*;

use super::DomException;

/// A catch-all error.
#[derive(Debug, Clone, PartialEq, derive_more::From)]
pub enum JSError {
    /// A generic [error](js_sys::Error) error.
    #[from]
    Error(js_sys::Error),

    /// A non-[error](js_sys::Error) error.
    Unknown(JsValue),
}

fwd_from!(DomException, web_sys::DomException > JSError);

impl From<JsValue> for JSError {
    fn from(value: JsValue) -> Self {
        match value.dyn_into() {
            Ok(e) => Self::Error(e),
            Err(e) => Self::Unknown(e),
        }
    }
}

impl From<web_sys::DomException> for JSError {
    fn from(value: web_sys::DomException) -> Self {
        value.unchecked_into::<js_sys::Error>().into()
    }
}

impl Display for JSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(e) => Display::fmt(&e.message(), f),
            Self::Unknown(_) => f.write_str("Non-Error JSValue error"),
        }
    }
}

impl std::error::Error for JSError {}

impl AsRef<JsValue> for JSError {
    fn as_ref(&self) -> &JsValue {
        match self {
            Self::Error(e) => e.unchecked_ref(),
            Self::Unknown(e) => e,
        }
    }
}

impl From<JSError> for JsValue {
    fn from(value: JSError) -> Self {
        match value {
            JSError::Error(e) => e.unchecked_into(),
            JSError::Unknown(e) => e,
        }
    }
}

unsafe impl Send for JSError {}
