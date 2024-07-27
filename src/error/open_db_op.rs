use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use wasm_bindgen::prelude::*;

use super::{DomException, Error, OpenDbError};

/// Error running user operation within an open DB transaction
#[derive(Debug, derive_more::From)]
pub enum OpenDbOpError<B = Error, U = Error> {
    /// Error during an [`upgradeneeded`](crate::OpenDbRequestBuilder::with_on_upgrade_needed) callback
    UpgradeNeeded(U),

    /// Error during a [`blocked`](crate::OpenDbRequestBuilder::with_on_blocked) callback
    Blocked(B),

    /// Driver error
    #[from]
    System(OpenDbError),
}

fwd_from!(Error, OpenDbError > OpenDbOpError);

fwd_from!(DomException, OpenDbError > OpenDbOpError);
fwd_from!(web_sys::DomException, OpenDbError > OpenDbOpError);
fwd_from!(js_sys::Error, OpenDbError > OpenDbOpError);
fwd_from!(JsValue, OpenDbError > OpenDbOpError);

impl<B: Display, U: Display> Display for OpenDbOpError<B, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System(e) => Display::fmt(e, f),
            Self::Blocked(e) => Display::fmt(e, f),
            Self::UpgradeNeeded(e) => Display::fmt(e, f),
        }
    }
}

impl<B: StdError + 'static, U: StdError + 'static> StdError for OpenDbOpError<B, U> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::System(e) => e,
            Self::Blocked(e) => e,
            Self::UpgradeNeeded(e) => e,
        })
    }
}
