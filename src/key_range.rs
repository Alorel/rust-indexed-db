use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use wasm_bindgen::prelude::*;

use crate::primitive::TryToJs;

/// An [`IDBKeyRange`](https://developer.mozilla.org/en-US/docs/Web/API/IDBKeyRange) implementation.
///
/// This enum gets converted into a [`JsValue`] when communicating with the underlying `IndexedDB` implementation; this
/// conversion throws an error if one of the following conditions is met:
///
/// - The lower or upper parameters were not passed a valid key.
/// - The lower key is greater than the upper key.
/// - The lower key and upper key match and either of the bounds are open.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub enum KeyRange<T> {
    /// `(lower_bound_key, lower_bound_open)`
    LowerBound(T, bool),

    /// `(upper_bound_key, upper_bound_open)`
    UpperBound(T, bool),

    /// `(lower_bound_key, lower_bound_open, upper_bound_key, upper_bound_open)`
    Bound(T, bool, T, bool),

    /// Note: This defaults to passing the value on directly instead of calling
    /// [`IDBKeyRange.only`](https://developer.mozilla.org/en-US/docs/Web/API/IDBKeyRange/only).
    /// In order to use an explicit `only` key range, use the
    /// [`explicitly_only`](KeyRange::<JsValue>::explicitly_only) method.
    Only(T),
}

impl KeyRange<JsValue> {
    /// Construct a [`KeyRange`] with an explicit
    /// [`IDBKeyRange.only`](https://developer.mozilla.org/en-US/docs/Web/API/IDBKeyRange/only) call.
    ///
    /// See also [`KeyRange::Only`](KeyRange::Only) for more info.
    #[allow(clippy::missing_errors_doc)]
    pub fn explicitly_only<T: TryToJs>(value: T) -> crate::Result<Self> {
        value.try_to_js().map(Self::Only)
    }
}

impl<T> KeyRange<T> {
    fn try_to_js_any<F>(&self, key_fn: F) -> crate::Result<JsValue>
    where
        F: Fn(&T) -> crate::Result<JsValue>,
    {
        let res = match *self {
            Self::Only(ref key) => return key_fn(key),
            Self::LowerBound(ref key, open) => {
                web_sys::IdbKeyRange::lower_bound_with_open(&key_fn(key)?, open)
            }
            Self::UpperBound(ref key, open) => {
                web_sys::IdbKeyRange::upper_bound_with_open(&key_fn(key)?, open)
            }
            Self::Bound(ref lower, lower_open, ref upper, upper_open) => {
                web_sys::IdbKeyRange::bound_with_lower_open_and_upper_open(
                    &key_fn(lower)?,
                    &key_fn(upper)?,
                    lower_open,
                    upper_open,
                )
            }
        };

        match res {
            Ok(v) => Ok(v.unchecked_into()),
            Err(e) => Err(e.into()),
        }
    }
}

impl<T> From<Range<T>> for KeyRange<T> {
    #[inline]
    fn from(value: Range<T>) -> Self {
        Self::Bound(value.start, false, value.end, true)
    }
}

impl<T> From<RangeFrom<T>> for KeyRange<T> {
    #[inline]
    fn from(value: RangeFrom<T>) -> Self {
        Self::LowerBound(value.start, false)
    }
}

impl<T> From<RangeInclusive<T>> for KeyRange<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (start, end) = value.into_inner();

        Self::Bound(start, false, end, false)
    }
}

impl<T> From<RangeTo<T>> for KeyRange<T> {
    #[inline]
    fn from(value: RangeTo<T>) -> Self {
        Self::UpperBound(value.end, false)
    }
}

impl<T> From<RangeToInclusive<T>> for KeyRange<T> {
    #[inline]
    fn from(value: RangeToInclusive<T>) -> Self {
        Self::UpperBound(value.end, true)
    }
}

impl<T: TryToJs> From<T> for KeyRange<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Only(value)
    }
}

impl<T: TryToJs> TryToJs for KeyRange<T> {
    #[inline]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        self.try_to_js_any(TryToJs::try_to_js)
    }
}

#[cfg(feature = "serde")]
const _: () = {
    use crate::serde::SerialiseToJs;
    use serde::Serialize;

    impl<T: Serialize> SerialiseToJs for KeyRange<T> {
        #[inline]
        fn serialise_to_js(&self) -> crate::Result<JsValue> {
            self.try_to_js_any(SerialiseToJs::serialise_to_js)
        }
    }

    impl<T: Serialize> SerialiseToJs for &KeyRange<T> {
        #[inline]
        fn serialise_to_js(&self) -> crate::Result<JsValue> {
            <KeyRange<T>>::serialise_to_js(self)
        }
    }
};
