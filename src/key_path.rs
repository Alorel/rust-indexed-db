use crate::internal_utils::slice_to_arr;
use internal_macros::generic_bounds;
use smallvec::SmallVec;
use wasm_bindgen::prelude::*;

/// Alias for the [`SmallVec`](SmallVec)s used for [`Sequence`](KeyPath::Sequence) key paths.
pub type KeyPathSeq<T = String> = SmallVec<[T; 3]>;

/// A [key path](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#gloss_keypath)
/// representation.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyPath<T = String> {
    /// A key path consisting of a single string.
    One(T),

    /// A key path consisting of a sequence of strings.
    Sequence(KeyPathSeq<T>),

    /// A raw key path.
    JsValue(JsValue),
}

#[generic_bounds(key_path(T))]
impl<T> KeyPath<T> {
    /// Convert the key path to a `JsValue`.
    pub fn to_js(&self) -> JsValue {
        match self {
            Self::One(v) => JsValue::from_str(v.as_ref()),
            Self::Sequence(seq) => slice_to_arr(seq).unchecked_into(),
            Self::JsValue(v) => v.clone(),
        }
    }
}

#[generic_bounds(key_path(T))]
impl<T> From<T> for KeyPath<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::One(value)
    }
}

#[generic_bounds(key_path(K))]
impl<K> FromIterator<K> for KeyPath<K> {
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        Self::Sequence(iter.into_iter().collect())
    }
}

#[generic_bounds(key_path(T))]
impl<T> From<KeyPathSeq<T>> for KeyPath<T> {
    #[inline]
    fn from(value: KeyPathSeq<T>) -> Self {
        Self::Sequence(value)
    }
}

#[generic_bounds(key_path(T))]
impl<T> From<Vec<T>> for KeyPath<T> {
    fn from(value: Vec<T>) -> Self {
        Self::Sequence(KeyPathSeq::from_vec(value))
    }
}

#[generic_bounds(key_path(T))]
impl<T> From<&[T]> for KeyPath<T>
where
    T: Clone,
{
    fn from(slice: &[T]) -> Self {
        slice.iter().cloned().collect()
    }
}

#[generic_bounds(key_path(T))]
impl<T, const N: usize> From<[T; N]> for KeyPath<T> {
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl From<JsValue> for KeyPath {
    #[inline]
    fn from(value: JsValue) -> Self {
        if let Some(str) = value.as_string() {
            KeyPath::One(str)
        } else {
            match value.dyn_into::<js_sys::Array>() {
                Ok(arr) => {
                    let iter = arr.into_iter().filter_map(move |v| v.as_string());
                    iter.collect()
                }
                Err(value) => KeyPath::JsValue(value),
            }
        }
    }
}

#[generic_bounds(key_path(T))]
impl<T> From<&KeyPath<T>> for JsValue {
    #[inline]
    fn from(value: &KeyPath<T>) -> Self {
        value.to_js()
    }
}

#[cfg(feature = "serde")]
const _: () = {
    use serde::de::{Deserializer, Error as DeError, SeqAccess, Visitor};
    use serde::ser::{Error as SerError, SerializeSeq, Serializer};
    use serde::{Deserialize, Serialize};
    use std::fmt::Formatter;

    #[generic_bounds(key_path(T))]
    impl<T> Serialize for KeyPath<T> {
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                Self::One(v) => ser.serialize_str(v.as_ref()),
                Self::Sequence(sequence) => {
                    let mut ser = ser.serialize_seq(Some(sequence.len()))?;
                    for el in sequence {
                        ser.serialize_element(el.as_ref())?;
                    }
                    ser.end()
                }
                Self::JsValue(_) => Err(S::Error::custom("`JSValue`s cannot be serialised")),
            }
        }
    }

    impl<'de> Deserialize<'de> for KeyPath {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Vis;
            impl<'de> Visitor<'de> for Vis {
                type Value = KeyPath;

                fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                    formatter.write_str("str or str sequence")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: DeError,
                {
                    self.visit_string(v.into())
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: DeError,
                {
                    Ok(KeyPath::One(v))
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut out = KeyPathSeq::new();
                    while let Some(next) = seq.next_element()? {
                        out.push(next);
                    }

                    Ok(KeyPath::Sequence(out))
                }
            }

            deserializer.deserialize_any(Vis)
        }
    }
};
