use std::ops::Deref;

use wasm_bindgen::prelude::*;

use crate::idb_query_source::IdbQuerySource;

use super::IdbCursor;

/// Like [IdbCursor], but iterates values as well as keys
///
/// Features required: `cursors`
#[derive(Debug)]
pub struct IdbCursorWithValue<'a, T: IdbQuerySource>(IdbCursor<'a, T>);

impl<'a, T: IdbQuerySource> IdbCursorWithValue<'a, T> {
    #[inline]
    pub(crate) fn new(inner: IdbCursor<'a, T>) -> Self {
        Self(inner)
    }

    /// Get the cursor's current value
    pub fn value(&self) -> JsValue {
        self.0.inner_as_cursor_with_value().value().unwrap()
    }
}

impl<'a, T: IdbQuerySource> Deref for IdbCursorWithValue<'a, T> {
    type Target = IdbCursor<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
