use std::error::Error as StdError;

use wasm_bindgen::prelude::*;

/// Error converting a [`JsValue`] into [`SimpleValue`](crate::value::JSPrimitive).
#[derive(Debug, thiserror::Error)]
pub enum SimpleValueError {
    /// Expected the value to be a string, but it isn't.
    #[error("Not a string")]
    NotAString(JsValue),

    /// Expected the value to be a number, but it isn't.
    #[error("Not a number")]
    NotANumber(JsValue),

    /// Expected the value to be a boolean, but it isn't.
    #[error("Not a boolean")]
    NotABoolean(JsValue),

    /// [Dynamic cast](JsCast::dyn_into) failed.
    #[error("Dynamic cast failed")]
    DynCast(JsValue),

    /// The value is too large.
    #[error("The value is too large: {0}")]
    TooLarge(f64),

    /// The value is too small.
    #[error("The value is too small: {0}")]
    TooSmall(f64),

    /// Expected an unsigned value.
    #[error("The value is signed: {0}")]
    Signed(f64),

    /// Any other catch-all error
    #[error(transparent)]
    Other(Box<dyn StdError>),
}

impl SimpleValueError {
    pub(crate) fn cast_err<E: StdError + 'static>(e: E) -> Self {
        Self::Other(Box::new(e))
    }
}
