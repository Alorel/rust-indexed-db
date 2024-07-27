use serde::Serialize;

use internal_macros::serrdoc;

use crate::future::{MaybeErrored, VoidRequest};

use super::ObjectStore;

#[cfg(feature = "serde")]
impl<'a> ObjectStore<'a> {
    /// Add the value to the object store. Throws if the computed key already exists - use the
    /// [`put`](Self::put) method if you want to update the value.
    ///
    /// # Errors
    /// See [`add_primitive`](Self::add_primitive).
    #[serrdoc]
    pub fn add<V: Serialize>(&self, value: &V) -> MaybeErrored<VoidRequest> {
        match serde_wasm_bindgen::to_value(value) {
            Ok(v) => self.add_primitive(v),
            Err(e) => MaybeErrored::errored(e.into()),
        }
    }
}
