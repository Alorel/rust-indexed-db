use cfg_if::cfg_if;
use std::fmt;

use super::Error;
use super::SerialisationError;

cfg_if! {
    if #[cfg(feature = "serde")] {
        use serde_wasm_bindgen::Error as BaseError;
    } else {
        use crate::internal_utils::StructName;
    }
}

/// Wrapper around
/// [`serde_wasm_bindgen::Error`](https://docs.rs/serde-wasm-bindgen/0.6.5/serde_wasm_bindgen/struct.Error.html).
///
/// Is an empty struct if the `serde` feature is not enabled.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(derive_more::From), repr(transparent))]
#[cfg_attr(not(feature = "serde"), derive(StructName))]
pub struct SerdeError(#[cfg(feature = "serde")] BaseError);

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                fmt::Display::fmt(&self.0, f)
            } else {
                f.write_str(<Self as StructName>::TYPE_NAME)
            }
        }
    }
}

impl ::std::error::Error for SerdeError {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        cfg_if! {
            if #[cfg(feature = "serde")] {
                Some(&self.0)
            } else {
                None
            }
        }
    }
}

impl From<SerdeError> for Error {
    fn from(value: SerdeError) -> Self {
        SerialisationError::from(value).into()
    }
}

#[cfg(feature = "serde")]
impl From<BaseError> for Error {
    fn from(value: BaseError) -> Self {
        SerdeError::from(value).into()
    }
}

impl PartialEq for SerdeError {
    #[cfg_attr(not(feature = "serde"), allow(unused_variables))]
    fn eq(&self, other: &Self) -> bool {
        cfg_if! {
            if #[cfg(feature = "serde")] {
                self.0.to_string() == other.0.to_string()
            } else {
                true
            }
        }
    }
}

unsafe impl Send for SerdeError {}
