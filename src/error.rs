//! Crate errors

use wasm_bindgen::prelude::*;

pub use dom_exception::DomException;
pub use js_error::JSError;
pub use open_db::OpenDbError;
pub use open_db_op::OpenDbOpError;
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
mod open_db_op;
mod serde;
mod serialisation;
mod simple_value;

pub(crate) const CALLBACK_ERRORED: &str = "E_RS_IDB_CALLBACK";

/// Operation error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Most likely a driver error
    #[error(transparent)]
    DomException(#[from] DomException),

    /// (De)serialisation error
    #[error(transparent)]
    Serialisation(#[from] SerialisationError),

    /// Generic JS error
    #[error("Generic JS error: {0}")]
    Unknown(#[from] JSError),
}

impl From<SimpleValueError> for Error {
    fn from(value: SimpleValueError) -> Self {
        SerialisationError::from(value).into()
    }
}

impl From<web_sys::DomException> for Error {
    fn from(value: web_sys::DomException) -> Self {
        DomException::from(value).into()
    }
}

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
