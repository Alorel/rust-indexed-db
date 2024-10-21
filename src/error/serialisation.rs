use super::{SerdeError, SimpleValueError};

/// Error (de)serialising a value.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum SerialisationError {
    /// Error parsing a primitive value.
    #[error("Simple value parse error: {0}")]
    SimpleValue(#[from] SimpleValueError),

    /// Error (de)serialising a value. Holds an empty struct if the `serde` feature is not enabled.
    #[error(transparent)]
    Serde(#[from] SerdeError),
}
