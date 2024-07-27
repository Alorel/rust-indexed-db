//! Serde-specific utilities

use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A type that's convertible to [`JsValue`] using `serde`, but not necessarily
/// [`serialisable`](serde::Serialize) as a whole.
pub trait SerialiseToJs {
    /// Convert the type to a [`JsValue`], most likely using [`serde_wasm_bindgen`].
    #[allow(clippy::missing_errors_doc)]
    fn serialise_to_js(&self) -> crate::Result<JsValue>;
}

/// A type that's convertible from [`JsValue`] using `serde`, but not necessarily
/// [`deserialisable`](serde::Deserialize) as a whole.
pub trait DeserialiseFromJs {
    /// Deserialise a value from JS, most likely using [`serde_wasm_bindgen`].
    #[allow(clippy::missing_errors_doc)]
    fn deserialise_from_js(js: JsValue) -> crate::Result<Self>
    where
        Self: Sized;
}

impl<T: Serialize> SerialiseToJs for T {
    fn serialise_to_js(&self) -> crate::Result<JsValue> {
        serde_wasm_bindgen::to_value(self).map_err(Into::into)
    }
}

impl<T: DeserializeOwned> DeserialiseFromJs for T {
    fn deserialise_from_js(js: JsValue) -> crate::Result<Self> {
        serde_wasm_bindgen::from_value(js).map_err(Into::into)
    }
}
