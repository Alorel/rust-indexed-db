use wasm_bindgen::prelude::*;

use internal_macros::dom_exception_err;

use crate::internal_utils::SystemRepr;

/// Wrapper around a [web-sys `DomException`](web_sys::DomException).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, dom_exception_err)]
pub enum DomException {
    /// The object is in an invalid state.
    InvalidStateError(web_sys::DomException),

    /// A request was placed against a transaction that is currently not active or is finished.
    TransactionInactiveError(web_sys::DomException),

    /// The object cannot be found here.
    NotFoundError(web_sys::DomException),

    /// A mutation operation in a transaction failed because a constraint was not satisfied.
    ConstraintError(web_sys::DomException),

    /// The object does not support the operation or argument.
    InvalidAccessError(web_sys::DomException),

    /// Provided data is inadequate.
    DataError(web_sys::DomException),

    /// The operation was aborted.
    AbortError(web_sys::DomException),

    /// The mutating operation was attempted in a [`Readonly`](crate::transaction::TransactionMode::Readonly)
    /// transaction.
    ReadOnlyError(web_sys::DomException),

    /// The object can not be cloned.
    DataCloneError(web_sys::DomException),

    /// The string did not match the expected pattern.
    SyntaxError(web_sys::DomException),

    /// An different, unhandled DOM exception.
    #[from_dom_exception(default)]
    Other(web_sys::DomException),
}

impl DomException {
    /// Javascript error message
    #[must_use]
    pub fn message(&self) -> String {
        self.as_sys().message()
    }
}

impl AsRef<JsValue> for DomException {
    fn as_ref(&self) -> &JsValue {
        self.as_sys().unchecked_ref()
    }
}

impl From<DomException> for JsValue {
    fn from(value: DomException) -> Self {
        value.into_sys().unchecked_into()
    }
}

impl From<DomException> for web_sys::DomException {
    #[inline]
    fn from(value: DomException) -> Self {
        value.into_sys()
    }
}

impl TryFrom<JsValue> for DomException {
    type Error = JsValue;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        value.dyn_into::<web_sys::DomException>().map(Into::into)
    }
}

unsafe impl Send for DomException {}
