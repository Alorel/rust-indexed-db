use std::ops::Deref;

use wasm_bindgen::prelude::*;
use web_sys::DomException;

use crate::idb_query_source::IdbQuerySource;

use super::IdbCursor;

/// A key value pair returned by [IdbCursorWithValue::into_vec]
#[derive(Clone, Debug, PartialEq)]
pub struct KeyVal(JsValue, JsValue);

impl KeyVal {
    #[inline]
    pub(crate) fn new(key: JsValue, value: JsValue) -> Self {
        Self(key, value)
    }

    #[inline]
    pub fn key(&self) -> &JsValue {
        &self.0
    }

    #[inline]
    pub fn value(&self) -> &JsValue {
        &self.1
    }
}

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

    /// Consume the remainder of the cursor, collecting each key-value pair into a vector.
    ///
    /// ### Arguments
    ///
    /// - **skip** - how many times to advance the cursor before starting to collect key-value
    /// pairs. Setting this to 0 will include the current key and value in the output; setting it to
    /// 5 will skip the current key + value and 4 more.
    pub async fn into_vec(self, skip: u32) -> Result<Vec<KeyVal>, DomException> {
        self.handle_into_vec(skip, |k| KeyVal::new(k, self.value()))
            .await
    }

    /// Get the cursor's current value
    pub fn value(&self) -> JsValue {
        self.inner_as_cursor_with_value().value().unwrap()
    }
}

impl<'a, T: IdbQuerySource> Deref for IdbCursorWithValue<'a, T> {
    type Target = IdbCursor<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T: IdbQuerySource> AsRef<IdbCursor<'a, T>> for IdbCursorWithValue<'a, T> {
    #[inline]
    fn as_ref(&self) -> &IdbCursor<'a, T> {
        &self.0
    }
}
