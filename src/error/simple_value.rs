use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Write};
use wasm_bindgen::prelude::*;

/// Error converting a [`JsValue`] into a Rust equivalent.
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

    /// Error performing [`TryFromJs`](crate::primitive::TryFromJs) on a [`Switch2`](crate::primitive::Switch2) or its
    /// derivatives.
    #[error("Errors performing `TryFromJs` on `Switch`: {}", &FmtSimpleValueErrorArray(_0))]
    Switch(Vec<SimpleValueError>),

    /// Any other catch-all error.
    #[error(transparent)]
    Other(Box<dyn StdError>),
}

impl PartialEq for SimpleValueError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // JS
            (Self::NotAString(a), Self::NotAString(b))
            | (Self::NotANumber(a), Self::NotANumber(b))
            | (Self::NotABoolean(a), Self::NotABoolean(b))
            | (Self::DynCast(a), Self::DynCast(b)) => a.eq(b),

            // f64
            (Self::TooLarge(a), Self::TooLarge(b))
            | (Self::TooSmall(a), Self::TooSmall(b))
            | (Self::Signed(a), Self::Signed(b)) => a.eq(b),

            // Switch
            (Self::Switch(ref a), Self::Switch(ref b)) => a.eq(b),

            // Other
            (Self::Other(ref a), Self::Other(ref b)) => a.to_string() == b.to_string(),

            // Definitely not equal
            _ => false,
        }
    }
}

struct FmtSimpleValueErrorArray<'a>(&'a [SimpleValueError]);
impl Display for FmtSimpleValueErrorArray<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn fmt_one(item: &impl Display, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_char('`')?;
            Display::fmt(item, f)?;
            f.write_char('`')
        }

        let mut iter = self.0.iter();
        let Some(first) = iter.next() else {
            return f.write_str("[empty]");
        };

        fmt_one(first, f)?;

        for item in iter {
            f.write_str(", ")?;
            fmt_one(item, f)?;
        }

        Ok(())
    }
}

unsafe impl Send for SimpleValueError {}
