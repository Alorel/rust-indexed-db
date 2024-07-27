use super::{SerdeError, SimpleValueError};

/// Error (de)serialising a value
#[derive(Debug, thiserror::Error)]
pub enum SerialisationError {
    /// Error parsing a [`SimpleValue`](crate::value::JSPrimitive)
    #[error("Simple value parse error: {0}")]
    SimpleValue(#[from] SimpleValueError),

    /// Error (de)serialising a value. Holds an empty struct if the `serde` feature is not enabled.
    #[error(transparent)]
    Serde(#[from] SerdeError),
}
