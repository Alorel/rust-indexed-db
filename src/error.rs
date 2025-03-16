//! Crate errors.

use std::sync::PoisonError;
use wasm_bindgen::prelude::*;

pub use dom_exception::DomException;
pub use js_error::JSError;
pub use open_db::OpenDbError;
pub use serde::SerdeError;
pub use serialisation::SerialisationError;
pub use simple_value::SimpleValueError;

macro_rules! fwd_from {
    ($from: ty, $tmp: ty > $target: ty) => {
        impl From<$from> for $target {
            fn from(value: $from) -> Self {
                <$tmp>::from(value).into()
            }
        }
    };
}

mod dom_exception;
mod js_error;
mod open_db;
mod serde;
mod serialisation;
mod simple_value;
mod unexpected_data;

pub use unexpected_data::UnexpectedDataError;

/// Operation error
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// Most likely a driver error.
    #[error(transparent)]
    DomException(#[from] DomException),

    /// (De)serialisation error.
    #[error(transparent)]
    Serialisation(#[from] SerialisationError),

    /// Missing data.
    #[error("Missing data: {0}")]
    MissingData(#[from] UnexpectedDataError),

    /// Generic JS error.
    #[error("Generic JS error: {0}")]
    Unknown(#[from] JSError),
}

fwd_from!(SimpleValueError, SerialisationError > Error);
fwd_from!(web_sys::DomException, DomException > Error);

impl From<js_sys::Error> for Error {
    fn from(e: js_sys::Error) -> Self {
        match e.dyn_into::<web_sys::DomException>() {
            Ok(e) => e.into(),
            Err(e) => Self::Unknown(e.into()),
        }
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        match value.dyn_into::<web_sys::DomException>() {
            Ok(v) => v.into(),
            Err(v) => Self::Unknown(v.into()),
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    #[inline]
    fn from(value: PoisonError<T>) -> Self {
        UnexpectedDataError::from(value).into()
    }
}
