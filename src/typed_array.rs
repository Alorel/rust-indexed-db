//! Partial
//! [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray)
//! implementations. Currently only aims to offer convenient conversion to/from [`Vec`].

use fancy_constructor::new;
use std::iter::Copied;
use std::ops::Deref;
use wasm_bindgen::prelude::*;

/// A partial
/// [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray)
/// implementation.
///
/// Currently only aims to offer convenient conversion to/from [`Vec`].
#[derive(
    new,
    Debug,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Clone,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::Into,
)]
pub struct TypedArray<T>(#[new(name(source), into)] Vec<T>);

/// A [`TypedArray`] slice. Exists for performing [`TryToJs`](crate::primitive::TryToJs) conversions on borrowed data.
#[derive(
    new, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, derive_more::Into, derive_more::From,
)]
pub struct TypedArraySlice<'a, T>(#[new(name(source))] &'a [T]);

impl<T> TypedArray<T> {
    /// Convert this [`TypedArray`] into a [`TypedArraySlice`].
    #[must_use]
    pub fn as_slice(&self) -> TypedArraySlice<'_, T> {
        TypedArraySlice::new(&self.0)
    }
}

impl<T> Clone for TypedArraySlice<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TypedArraySlice<'_, T> {}

impl<T> Default for TypedArray<T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T> AsRef<[T]> for TypedArray<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for TypedArray<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut *self
    }
}

impl<T> IntoIterator for TypedArray<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: Copy> IntoIterator for &'a TypedArray<T> {
    type Item = T;
    type IntoIter = Copied<std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl<T> Deref for TypedArraySlice<'_, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> AsRef<[T]> for TypedArraySlice<'_, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<T: PartialEq> PartialEq<TypedArraySlice<'_, T>> for TypedArray<T> {
    fn eq(&self, other: &TypedArraySlice<'_, T>) -> bool {
        self.as_slice().eq(other)
    }
}

impl<T: PartialEq> PartialEq<TypedArray<T>> for TypedArraySlice<'_, T> {
    fn eq(&self, other: &TypedArray<T>) -> bool {
        other.as_slice().eq(self)
    }
}

macro_rules! impl_array {
    ($num: ty, $alias_name: ident, $slice_name: ident) => {
        #[doc = concat!(" A partial [`", stringify!($alias_name), "`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/", stringify!($alias_name) ,") implementation.")]
        pub type $alias_name = TypedArray<$num>;

        #[doc = concat!(" A [`", stringify!($alias_name), "`] slice.")]
        pub type $slice_name<'a> = TypedArraySlice<'a, $num>;

        impl From<Vec<$num>> for $alias_name {
            #[inline]
            fn from(source: Vec<$num>) -> Self {
                Self(source)
            }
        }

        impl FromIterator<$num> for $alias_name {
            fn from_iter<T: IntoIterator<Item = $num>>(iter: T) -> Self {
                Self(iter.into_iter().collect())
            }
        }

        impl $crate::primitive::TryToJs for $slice_name<'_> {
            fn try_to_js(&self) -> crate::Result<JsValue> {
                Ok(js_sys::$alias_name::from(self.as_ref()).unchecked_into())
            }
        }

        impl $crate::primitive::TryToJs for $alias_name {
            fn try_to_js(&self) -> crate::Result<JsValue> {
                Ok(js_sys::$alias_name::from(self.as_ref()).unchecked_into())
            }
        }

        impl $crate::primitive::TryFromJs for $alias_name {
            fn from_js(js: JsValue) -> Result<Self, $crate::error::SimpleValueError> {
                match js.dyn_into::<js_sys::$alias_name>() {
                    Ok(js) => Ok(Self(js.to_vec())),
                    Err(e) => Err($crate::error::SimpleValueError::DynCast(e)),
                }
            }
        }
    };
}

impl_array!(i8, Int8Array, Int8ArraySlice);
impl_array!(u8, Uint8Array, Uint8ArraySlice);
impl_array!(i16, Int16Array, Int16ArraySlice);
impl_array!(u16, Uint16Array, Uint16ArraySlice);
impl_array!(i32, Int32Array, Int32ArraySlice);
impl_array!(u32, Uint32Array, Uint32ArraySlice);
impl_array!(f32, Float32Array, Float32ArraySlice);
impl_array!(f64, Float64Array, Float64ArraySlice);
impl_array!(i64, BigInt64Array, BigInt64ArraySlice);
impl_array!(u64, BigUint64Array, BigUint64ArraySlice);

#[cfg(feature = "serde")]
const _: () = {
    use crate::primitive::{TryFromJs, TryToJs};
    use crate::serde::{DeserialiseFromJs, SerialiseToJs};

    impl<T> SerialiseToJs for TypedArray<T>
    where
        Self: TryToJs,
    {
        #[inline]
        fn serialise_to_js(&self) -> crate::Result<JsValue> {
            self.try_to_js()
        }
    }

    impl<T> SerialiseToJs for TypedArraySlice<'_, T>
    where
        Self: TryToJs,
    {
        #[inline]
        fn serialise_to_js(&self) -> crate::Result<JsValue> {
            self.try_to_js()
        }
    }

    impl<T> DeserialiseFromJs for TypedArray<T>
    where
        Self: TryFromJs,
    {
        fn deserialise_from_js(js: JsValue) -> crate::Result<Self> {
            Self::from_js(js).map_err(Into::into)
        }
    }
};
