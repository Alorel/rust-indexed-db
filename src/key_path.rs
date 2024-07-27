use std::borrow::Cow;

use wasm_bindgen::prelude::*;

use crate::internal_utils::slice_to_arr;

/// A [key path](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#gloss_keypath)
/// representation.
#[derive(Clone, PartialEq, Debug, derive_more::From)]
pub enum KeyPath<'a, S = Vec<String>> {
    /// A key path consisting of a single string
    #[from]
    One(Cow<'a, str>),

    /// A key path consisting of a sequence of strings
    Sequence(S),

    /// A raw key path
    JsValue(JsValue),
}

impl<'a, S: 'a> KeyPath<'a, S> {
    /// Convert to a [`JsValue`]
    pub fn to_js_value<I>(&'a self) -> JsValue
    where
        &'a S: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        match self {
            KeyPath::One(s) => JsValue::from_str(s.as_ref()),
            KeyPath::Sequence(s) => slice_to_arr(s).unchecked_into(),
            KeyPath::JsValue(v) => v.clone(),
        }
    }

    /// Convert to a [`JsValue`]
    pub fn into_js_value<I>(self) -> JsValue
    where
        S: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        match self {
            KeyPath::One(s) => JsValue::from_str(s.as_ref()),
            KeyPath::Sequence(s) => slice_to_arr(s).unchecked_into(),
            KeyPath::JsValue(v) => v,
        }
    }
}

impl KeyPath<'static> {
    /// Convert from a [`JsValue`]. Resolves to [`One`](KeyPath::One) if the value is a string,
    /// [`Sequence`](KeyPath::Sequence) if the value is an array, and
    /// [`JsValue`](KeyPath::JsValue) (invalid) otherwise.
    pub fn from_js_value(value: &JsValue) -> Self {
        if let Some(str) = value.as_string() {
            KeyPath::One(Cow::Owned(str))
        } else {
            match value.dyn_ref::<js_sys::Array>() {
                Some(arr) => {
                    let vec = arr.iter().filter_map(move |v| v.as_string()).collect();
                    KeyPath::Sequence(vec)
                }
                None => KeyPath::JsValue(value.clone()),
            }
        }
    }
}

impl<'a, 'r, I, S: 'a> From<&'r KeyPath<'a, S>> for JsValue
where
    &'r S: IntoIterator<Item = I>,
    I: AsRef<str>,
{
    #[inline]
    fn from(value: &'r KeyPath<'r, S>) -> Self {
        value.to_js_value()
    }
}

impl<'a, I, S: 'a> From<KeyPath<'a, S>> for JsValue
where
    S: IntoIterator<Item = I>,
    I: AsRef<str>,
{
    #[inline]
    fn from(value: KeyPath<'a, S>) -> Self {
        value.into_js_value()
    }
}

impl From<JsValue> for KeyPath<'static> {
    fn from(value: JsValue) -> Self {
        if let Some(str) = value.as_string() {
            KeyPath::One(Cow::Owned(str))
        } else {
            match value.dyn_into::<js_sys::Array>() {
                Ok(arr) => {
                    let vec = arr.into_iter().filter_map(move |v| v.as_string()).collect();
                    KeyPath::Sequence(vec)
                }
                Err(value) => KeyPath::JsValue(value),
            }
        }
    }
}

impl<'a, S> From<&'a str> for KeyPath<'a, S> {
    #[inline]
    fn from(value: &'a str) -> Self {
        KeyPath::One(Cow::Borrowed(value))
    }
}

impl<'a, S> From<&'a String> for KeyPath<'a, S> {
    #[inline]
    fn from(value: &'a String) -> Self {
        value.as_str().into()
    }
}

impl<S> From<String> for KeyPath<'static, S> {
    #[inline]
    fn from(value: String) -> Self {
        KeyPath::One(Cow::Owned(value))
    }
}

impl<'a, T: AsRef<str>> From<&'a [T]> for KeyPath<'a, &'a [T]> {
    #[inline]
    fn from(value: &'a [T]) -> Self {
        KeyPath::Sequence(value)
    }
}

impl<'a, T: AsRef<str>> From<Vec<T>> for KeyPath<'a, Vec<T>> {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        KeyPath::Sequence(value)
    }
}

impl<'a, T: AsRef<str> + 'a, const N: usize> From<[T; N]> for KeyPath<'a, [T; N]> {
    #[inline]
    fn from(value: [T; N]) -> Self {
        KeyPath::Sequence(value)
    }
}
