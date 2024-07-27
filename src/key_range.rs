use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

use sealed::sealed;
use wasm_bindgen::prelude::*;

use crate::primitive::TryToJs;
use crate::ToJs;

/// An [`IDBKeyRange`](https://developer.mozilla.org/en-US/docs/Web/API/IDBKeyRange) implementation.
///
/// This enum gets converted into a [`JsValue`] when communicating with the underlying IndexedDB implementation; this
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
    /// [`explicitly_only`](KeyRange::<JsValue>::explicitly_only) method.1
    Only(T),
}

impl KeyRange<JsValue> {
    /// Construct a [`KeyRange`] with an explicit
    /// [`IDBKeyRange.only`](https://developer.mozilla.org/en-US/docs/Web/API/IDBKeyRange/only) call.
    ///
    /// See also [`KeyRange::Only`](KeyRange::Only) for more info.
    pub fn explicitly_only<T: ToJs>(value: T) -> crate::Result<Self> {
        Ok(Self::Only(value.to_js()))
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

#[sealed]
#[allow(unused_qualifications)]
impl<T: TryToJs> crate::primitive::TryToJs for KeyRange<T> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        let res = match *self {
            Self::Only(ref key) => return key.try_to_js(),
            Self::LowerBound(ref key, open) => {
                web_sys::IdbKeyRange::lower_bound_with_open(&key.try_to_js()?, open)
            }
            Self::UpperBound(ref key, open) => {
                web_sys::IdbKeyRange::upper_bound_with_open(&key.try_to_js()?, open)
            }
            Self::Bound(ref lower, lower_open, ref upper, upper_open) => {
                web_sys::IdbKeyRange::bound_with_lower_open_and_upper_open(
                    &lower.try_to_js()?,
                    &upper.try_to_js()?,
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

#[sealed]
#[allow(unused_qualifications)]
impl<T: TryToJs> crate::primitive::TryToJs for &KeyRange<T> {
    #[inline]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        <KeyRange<T> as TryToJs>::try_to_js(self)
    }
}
