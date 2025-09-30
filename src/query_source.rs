//! Common functionality for making queries.

use sealed::sealed;
use wasm_bindgen::prelude::*;

pub use get_all::{GetAll, GetAllKeys, GetAllRecords};
use internal_macros::errdoc;

use crate::internal_utils::SystemRepr;
use crate::{KeyPath, KeyRange};
pub use get::Get;
pub use get_key::GetKey;
use internal::QuerySourceInternal;

pub use count::Count;

iffeat! {
    #[cfg(feature = "cursors")]
    pub(crate) mod cursor;
    pub use cursor::{AnyCursorBuilder, CursorBuilder, KeyCursorBuilder};
}

mod count;
mod get;
pub(crate) mod get_all;
mod get_key;

/// Common functionality for making queries.
#[sealed]
pub trait QuerySource {
    /// Count the number of documents in the index/object store.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, DataError))]
    fn count(&self) -> Count<'_, Self>
    where
        Self: Sized;

    /// Get one record from the object store or index. Returns the first match if a non-[only](KeyRange::Only) key is
    /// provided and multiple records match.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, DataError))]
    fn get<V, K, I>(&self, key: I) -> Get<'_, Self, K, V>
    where
        Self: Sized,
        I: Into<KeyRange<K>>;

    /// Return the first matching key selected by the specified query.
    #[errdoc(QuerySource(TransactionInactiveError, InvalidStateError, DataError))]
    fn get_key<K, I>(&self, key_range: I) -> GetKey<'_, Self, K>
    where
        Self: Sized,
        I: Into<KeyRange<K>>;

    /// Get all records in the object store or index.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, DataError))]
    fn get_all<V>(&self) -> GetAllRecords<'_, Self, V>
    where
        Self: Sized;

    /// Get all keys in the object store or index.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, DataError))]
    fn get_all_keys<K>(&self) -> GetAllKeys<'_, Self, K>
    where
        Self: Sized;

    /// Get the index/object store key path.
    fn key_path(&self) -> Option<KeyPath>;

    /// Get the index/object store name.
    fn name(&self) -> String;

    /// Set the index/object store name.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, ConstraintError))]
    fn set_name(&self, name: &str);

    iffeat! {
        #[cfg(feature = "cursors")]

        /// Open a cursor that iterates over the records in the index or object store.
        /// Resolves to `None` if the cursor is empty.
        #[errdoc(Cursor(TransactionInactiveError, DataErrorOpen, InvalidStateErrorOpen))]
        fn open_cursor(&self) -> CursorBuilder<'_, Self> where Self: Sized;

        /// Open a cursor that iterates over the keys in the index or object store.
        /// Resolves to `None` if the cursor is empty.
        #[errdoc(Cursor(TransactionInactiveError, DataErrorOpen, InvalidStateErrorOpen))]
        fn open_key_cursor(&self) -> KeyCursorBuilder<'_, Self> where Self: Sized;
    }
}

#[sealed]
impl<T: SystemRepr<Repr = R>, R: QuerySourceInternal> QuerySource for T {
    #[inline]
    fn name(&self) -> String {
        self.as_sys().name()
    }

    #[inline]
    fn set_name(&self, name: &str) {
        self.as_sys().set_name(name);
    }

    #[inline]
    fn count(&self) -> Count<'_, Self> {
        Count::new(self)
    }

    fn key_path(&self) -> Option<KeyPath> {
        match self.as_sys().key_path() {
            Ok(path) if !path.is_null() => Some(path.into()),
            _ => None,
        }
    }

    fn get<V, K, I>(&self, key: I) -> Get<'_, Self, K, V>
    where
        I: Into<KeyRange<K>>,
    {
        Get::new(self, key.into())
    }

    fn get_key<K, I>(&self, key_range: I) -> GetKey<'_, Self, K>
    where
        I: Into<KeyRange<K>>,
    {
        GetKey::new(self, key_range.into())
    }

    #[inline]
    fn get_all<V>(&self) -> GetAllRecords<'_, Self, V> {
        GetAllRecords::new(self)
    }

    #[inline]
    fn get_all_keys<K>(&self) -> GetAllKeys<'_, Self, K> {
        GetAllKeys::new(self)
    }

    iffeat! {
        #[cfg(feature = "cursors")]
        #[inline]
        fn open_cursor(&self) -> CursorBuilder<'_, Self> {
            CursorBuilder::new(self)
        }

        #[inline]
        fn open_key_cursor(&self) -> KeyCursorBuilder<'_, Self> {
            KeyCursorBuilder::new(self)
        }
    }
}

pub(crate) mod internal {
    use wasm_bindgen::prelude::*;

    /// Internal representation of a [`QuerySource`](super::QuerySource).
    #[::sealed::sealed(pub(super))]
    pub trait QuerySourceInternal {
        #[doc(hidden)]
        fn name(&self) -> String;

        #[doc(hidden)]
        fn set_name(&self, name: &str);

        #[doc(hidden)]
        fn count(&self) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn count_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn key_path(&self) -> Result<JsValue, JsValue>;

        #[doc(hidden)]
        fn get(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn get_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn get_all(&self) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn get_all_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn get_all_with_key_and_limit(
            &self,
            key: &JsValue,
            limit: u32,
        ) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn get_all_keys(&self) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn get_all_keys_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn get_all_keys_with_key_and_limit(
            &self,
            key: &JsValue,
            limit: u32,
        ) -> Result<web_sys::IdbRequest, JsValue>;

        iffeat! {
            #[cfg(feature = "cursors")]
            #[doc(hidden)]
            fn open_cursor(&self) -> Result<web_sys::IdbRequest, JsValue>;

            #[doc(hidden)]
            fn open_cursor_with_range(&self, range: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;

            #[doc(hidden)]
            fn open_cursor_with_range_and_direction(
                &self,
                range: &JsValue,
                direction: web_sys::IdbCursorDirection,
            ) -> Result<web_sys::IdbRequest, JsValue>;

            #[doc(hidden)]
            fn open_key_cursor(&self) -> Result<web_sys::IdbRequest, JsValue>;

            #[doc(hidden)]
            fn open_key_cursor_with_range(&self, range: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;

            #[doc(hidden)]
            fn open_key_cursor_with_range_and_direction(
                &self,
                range: &JsValue,
                direction: web_sys::IdbCursorDirection,
            ) -> Result<web_sys::IdbRequest, JsValue>;
        }
    }
}

macro_rules! impl_internal {
    ($for: ty) => {
        #[::sealed::sealed]
        impl internal::QuerySourceInternal for $for {
            #[inline]
            fn name(&self) -> String {
                <$for>::name(self)
            }

            #[inline]
            fn set_name(&self, name: &str) {
                <$for>::set_name(self, name);
            }

            #[inline]
            fn count(&self) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::count(self)
            }

            #[inline]
            fn count_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::count_with_key(self, key)
            }

            #[inline]
            fn key_path(&self) -> Result<JsValue, JsValue> {
                <$for>::key_path(self)
            }

            #[inline]
            fn get(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get(self, key)
            }

            #[inline]
            fn get_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_key(self, key)
            }

            #[inline]
            fn get_all(&self) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_all(self)
            }

            #[inline]
            fn get_all_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_all_with_key(self, key)
            }

            #[inline]
            fn get_all_with_key_and_limit(
                &self,
                key: &JsValue,
                limit: u32,
            ) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_all_with_key_and_limit(self, key, limit)
            }

            #[inline]
            fn get_all_keys(&self) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_all_keys(self)
            }

            #[inline]
            fn get_all_keys_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_all_keys_with_key(self, key)
            }

            #[inline]
            fn get_all_keys_with_key_and_limit(
                &self,
                key: &JsValue,
                limit: u32,
            ) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_all_keys_with_key_and_limit(self, key, limit)
            }

            iffeat! {
                #[cfg(feature = "cursors")]
                #[inline]
                fn open_cursor(&self) -> Result<web_sys::IdbRequest, JsValue> {
                    <$for>::open_cursor(self)
                }

                #[inline]
                fn open_cursor_with_range(&self, range: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                    <$for>::open_cursor_with_range(self, range)
                }

                #[inline]
                fn open_cursor_with_range_and_direction(
                    &self,
                    range: &JsValue,
                    direction: web_sys::IdbCursorDirection,
                ) -> Result<web_sys::IdbRequest, JsValue> {
                    <$for>::open_cursor_with_range_and_direction(self, range, direction)
                }

                #[inline]
                fn open_key_cursor(&self) -> Result<web_sys::IdbRequest, JsValue> {
                    <$for>::open_key_cursor(self)
                }

                #[inline]
                 fn open_key_cursor_with_range(&self, range: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                    <$for>::open_key_cursor_with_range(self, range)
                }

                #[inline]
                fn open_key_cursor_with_range_and_direction(
                    &self,
                    range: &JsValue,
                    direction: web_sys::IdbCursorDirection,
                ) -> Result<web_sys::IdbRequest, JsValue> {
                    <$for>::open_key_cursor_with_range_and_direction(self, range, direction)
                }
            }
        }
    };
}

impl_internal!(web_sys::IdbObjectStore);

#[cfg(feature = "indices")]
impl_internal!(web_sys::IdbIndex);
