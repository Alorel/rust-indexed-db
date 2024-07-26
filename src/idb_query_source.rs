use wasm_bindgen::{prelude::*, JsCast};
use web_sys::DomException;

use crate::idb_key_path::IdbKeyPath;
use crate::request::{CountFuture, JsCastRequestFuture, OptionalJsValueFuture};
#[cfg(feature = "cursors")]
use crate::request::{IdbCursorFuture, IdbCursorWithValueFuture};

/// Code shared between [indices](crate::idb_index::IdbIndex) and
/// [object stores](crate::idb_object_store::IdbObjectStore)
pub trait IdbQuerySource: Sized {
    /// Get the index/object store name
    fn name(&self) -> String;

    /// Set the index/object store name
    fn set_name(&self, name: &str);

    /// Get the index/object store key path. Returns `None` if the index isn't auto-populated.
    fn key_path(&self) -> Option<IdbKeyPath>;

    /// Find either the value in the referenced object store that corresponds to the given key or
    /// the first corresponding value, if key is an [`IDBKeyRange`](web_sys::IdbKeyRange).
    fn get<K: JsCast>(&self, key: &K) -> Result<OptionalJsValueFuture, DomException>;

    /// Find either the value in the referenced object store that corresponds to the given key or
    /// the first corresponding value, if key is an [`IDBKeyRange`](web_sys::IdbKeyRange).
    #[inline]
    fn get_owned<K: Into<JsValue>>(&self, key: K) -> Result<OptionalJsValueFuture, DomException> {
        self.get(&key.into())
    }

    /// Get all values in the index/object store
    fn get_all(&self) -> Result<JsCastRequestFuture<js_sys::Array>, DomException>;

    /// Get all values in the index/object store that correspond to the given key or are in
    /// range, if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange).
    fn get_all_with_key<K: JsCast>(
        &self,
        key: &K,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException>;

    /// Get all values in the index/object store that correspond to the given key or are in
    /// range, if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange).
    #[inline]
    fn get_all_with_key_owned<K: Into<JsValue>>(
        &self,
        key: K,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException> {
        self.get_all_with_key(&key.into())
    }

    /// Get all values in the index/object store that correspond to the given key or are in
    /// range, if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange). `limit` controls the
    /// maximum number of results to return
    fn get_all_with_key_and_limit<K: JsCast>(
        &self,
        key: &K,
        limit: u32,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException>;

    /// Get all values in the index/object store that correspond to the given key or are in
    /// range, if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange). `limit` controls the
    /// maximum number of results to return
    fn get_all_with_key_and_limit_owned<K: Into<JsValue>>(
        &self,
        key: K,
        limit: u32,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException> {
        self.get_all_with_key_and_limit(&key.into(), limit)
    }

    /// Count the number of documents in the index/object store
    fn count(&self) -> Result<CountFuture, DomException>;

    /// Count the number of documents in the index/object store within the given key range
    fn count_with_key<K: JsCast>(&self, key: &K) -> Result<CountFuture, DomException>;

    /// Count the number of documents in the index/object store within the given key range
    #[inline]
    fn count_with_key_owned<K: Into<JsValue>>(&self, key: K) -> Result<CountFuture, DomException> {
        self.count_with_key(&key.into())
    }

    /// Find either the given key or the primary key, if key is an
    /// [`IDBKeyRange`](web_sys::IdbKeyRange).
    fn get_key<K: JsCast>(&self, key: &K) -> Result<OptionalJsValueFuture, DomException>;

    /// Find either the given key or the primary key, if key is an
    /// [`IDBKeyRange`](web_sys::IdbKeyRange).
    #[inline]
    fn get_key_owned<K>(&self, key: K) -> Result<OptionalJsValueFuture, DomException>
    where
        K: JsCast,
    {
        self.get_key(&key.into())
    }

    /// Get all the keys in the index/object store
    fn get_all_keys(&self) -> Result<JsCastRequestFuture<js_sys::Array>, DomException>;

    /// Get all the keys in the index/object store that correspond to the given key or are in range
    /// if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange).
    fn get_all_keys_with_key<K: JsCast>(
        &self,
        key: &K,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException>;

    /// Get all the keys in the index/object store that correspond to the given key or are in range
    /// if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange).
    #[inline]
    fn get_all_keys_with_key_owned<K: Into<JsValue>>(
        &self,
        key: K,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException> {
        self.get_all_keys_with_key(&key.into())
    }

    /// Get all the keys in the index/object store that correspond to the given key or are in range
    /// if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange), up to the given limit.
    fn get_all_keys_with_key_and_limit<K: JsCast>(
        &self,
        key: &K,
        limit: u32,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException>;

    /// Get all the keys in the index/object store that correspond to the given key or are in range
    /// if the key is an [`IDBKeyRange`](web_sys::IdbKeyRange), up to the given limit.
    fn get_all_keys_with_key_and_limit_owned<K: Into<JsValue>>(
        &self,
        key: K,
        limit: u32,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException> {
        self.get_all_keys_with_key_and_limit(&key.into(), limit)
    }

    /// Get all the keys in the index/object store, up to the given limit.
    #[inline]
    fn get_all_keys_with_limit(
        &self,
        limit: u32,
    ) -> Result<JsCastRequestFuture<js_sys::Array>, DomException> {
        self.get_all_keys_with_key_and_limit(&JsValue::undefined(), limit)
    }

    /// Open a cursor
    #[cfg(feature = "cursors")]
    fn open_cursor(&self) -> Result<IdbCursorWithValueFuture<Self>, DomException>;

    /// Open a cursor with the given key range
    #[cfg(feature = "cursors")]
    fn open_cursor_with_range<K: JsCast>(
        &self,
        range: &K,
    ) -> Result<IdbCursorWithValueFuture<Self>, DomException>;

    /// Open a cursor with the given key range
    #[cfg(feature = "cursors")]
    #[inline]
    fn open_cursor_with_range_owned<K: Into<JsValue>>(
        &self,
        range: K,
    ) -> Result<IdbCursorWithValueFuture<Self>, DomException> {
        self.open_cursor_with_range(&range.into())
    }

    /// Open a cursor with the given key range and direction
    #[cfg(feature = "cursors")]
    fn open_cursor_with_range_and_direction<K: JsCast>(
        &self,
        range: &K,
        direction: web_sys::IdbCursorDirection,
    ) -> Result<IdbCursorWithValueFuture<Self>, DomException>;

    /// Open a cursor with the given key range and direction
    #[cfg(feature = "cursors")]
    #[inline]
    fn open_cursor_with_range_and_direction_owned<K: Into<JsValue>>(
        &self,
        range: K,
        direction: web_sys::IdbCursorDirection,
    ) -> Result<IdbCursorWithValueFuture<Self>, DomException> {
        self.open_cursor_with_range_and_direction(&range.into(), direction)
    }

    /// Open a cursor with the given and direction
    #[cfg(feature = "cursors")]
    #[inline]
    fn open_cursor_with_direction(
        &self,
        direction: web_sys::IdbCursorDirection,
    ) -> Result<IdbCursorWithValueFuture<Self>, DomException> {
        self.open_cursor_with_range_and_direction(&JsValue::undefined(), direction)
    }

    /// Open a key cursor
    #[cfg(feature = "cursors")]
    fn open_key_cursor(&self) -> Result<IdbCursorFuture<Self>, DomException>;

    /// Open a key cursor with the given key range
    #[cfg(feature = "cursors")]
    fn open_key_cursor_with_range<K: JsCast>(
        &self,
        range: &K,
    ) -> Result<IdbCursorFuture<Self>, DomException>;

    /// Open a key cursor with the given key range
    #[cfg(feature = "cursors")]
    #[inline]
    fn open_key_cursor_with_range_owned<K: Into<JsValue>>(
        &self,
        range: K,
    ) -> Result<IdbCursorFuture<Self>, DomException> {
        self.open_key_cursor_with_range(&range.into())
    }

    /// Open a key cursor with the given key range and direction
    #[cfg(feature = "cursors")]
    fn open_key_cursor_with_range_and_direction<K: JsCast>(
        &self,
        range: &K,
        direction: web_sys::IdbCursorDirection,
    ) -> Result<IdbCursorFuture<Self>, DomException>;

    /// Open a key cursor with the given key range and direction
    #[cfg(feature = "cursors")]
    fn open_key_cursor_with_range_and_direction_owned<K: Into<JsValue>>(
        &self,
        range: K,
        direction: web_sys::IdbCursorDirection,
    ) -> Result<IdbCursorFuture<Self>, DomException> {
        self.open_key_cursor_with_range_and_direction(&range.into(), direction)
    }

    /// Open a key cursor with the given and direction
    #[cfg(feature = "cursors")]
    #[inline]
    fn open_key_cursor_with_direction(
        &self,
        direction: web_sys::IdbCursorDirection,
    ) -> Result<IdbCursorFuture<Self>, DomException> {
        self.open_key_cursor_with_range_and_direction(&JsValue::undefined(), direction)
    }
}
