use wasm_bindgen::prelude::*;

use crate::idb_database::IdbVersionChangeEvent;

/// Common trait for `IdbOpenDbRequest`s
pub trait IdbOpenDbRequestLike {
    /// Set the callback for the `upgradeneeded` event
    fn set_on_upgrade_needed<F>(&mut self, callback: Option<F>)
    where
        F: Fn(&IdbVersionChangeEvent) -> Result<(), JsValue> + 'static;

    /// Set the callback for the `blocked` event
    fn set_on_blocked<F>(&mut self, callback: Option<F>)
    where
        F: Fn(&IdbVersionChangeEvent) -> Result<(), JsValue> + 'static;
}
