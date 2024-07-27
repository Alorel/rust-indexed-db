use crate::error::SimpleValueError;
use crate::iter::ArrayMapIter;
use cfg_if::cfg_if;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::{BuildHasher, Hash};
use wasm_bindgen::prelude::*;

/// Convert from a JS value.
pub trait TryFromJs {
    /// Convert from a JS value.
    #[allow(clippy::missing_errors_doc)]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError>
    where
        Self: Sized;
}

pub(crate) trait TryFromJsExt: TryFromJs {
    fn from_js_base(js: JsValue) -> crate::Result<Self>
    where
        Self: Sized;
}

impl<T: TryFromJs> TryFromJsExt for T {
    fn from_js_base(js: JsValue) -> crate::Result<Self> {
        Self::from_js(js).map_err(Into::into)
    }
}

impl<T: TryFromJs> TryFromJs for Option<T> {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        if js.is_undefined() || js.is_null() {
            Ok(None)
        } else {
            T::from_js(js).map(Some)
        }
    }
}

impl TryFromJs for f32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        const MAX: f64 = f32::MAX as f64;
        let big = <f64 as TryFromJs>::from_js(js)?;

        if big > MAX {
            Err(SimpleValueError::TooLarge(big))
        } else {
            Ok(big as f32)
        }
    }
}

impl TryFromJs for String {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        match js.as_string() {
            Some(v) => Ok(v),
            None => Err(SimpleValueError::NotAString(js)),
        }
    }
}

impl TryFromJs for char {
    #[allow(clippy::cast_precision_loss)]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        let string = String::from_js(js)?;
        let mut chars = string.chars();

        if let Some(ch) = chars.next() {
            if chars.next().is_some() {
                Err(SimpleValueError::TooLarge(string.len() as f64))
            } else {
                Ok(ch)
            }
        } else {
            Err(SimpleValueError::TooSmall(0.0))
        }
    }
}

impl TryFromJs for JsValue {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        Ok(js)
    }
}

fn js_to_list<L, T>(js: JsValue) -> Result<L, SimpleValueError>
where
    T: TryFromJs,
    L: FromIterator<T>,
{
    if js.is_null() || js.is_undefined() {
        return Ok(None::<T>.into_iter().collect());
    }

    match js.dyn_into::<js_sys::Array>() {
        Ok(arr) => ArrayMapIter::new(arr.into_iter(), T::from_js).collect(),
        Err(js) => Err(SimpleValueError::DynCast(js)),
    }
}

pub(crate) fn js_to_map<M, T>(js: JsValue) -> Result<M, SimpleValueError>
where
    M: FromIterator<(String, T)>,
    T: TryFromJs,
{
    fn parse_tuple<T: TryFromJs>(tuple: JsValue) -> Result<(String, T), SimpleValueError> {
        match tuple.dyn_into::<js_sys::Array>() {
            Ok(tuple) => {
                let second = T::from_js(tuple.get(1))?;
                let first = tuple.get(0);

                match first.as_string() {
                    Some(first) => Ok((first, second)),
                    None => Err(SimpleValueError::NotAString(first)),
                }
            }
            Err(e) => Err(SimpleValueError::DynCast(e)),
        }
    }

    if js.is_null() || js.is_undefined() {
        return Ok(None::<(String, T)>.into_iter().collect());
    }

    match js.dyn_into::<js_sys::Object>() {
        Ok(obj) => js_sys::Object::entries(&obj)
            .into_iter()
            .map(parse_tuple)
            .collect(),
        Err(js) => Err(SimpleValueError::DynCast(js)),
    }
}

impl<T: TryFromJs> TryFromJs for BTreeMap<String, T> {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        js_to_map(js)
    }
}

impl<T: TryFromJs + Hash, H: BuildHasher + Default> TryFromJs for HashMap<String, T, H> {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        js_to_map(js)
    }
}

impl<T: TryFromJs> TryFromJs for Vec<T> {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        js_to_list(js)
    }
}

impl<T: TryFromJs> TryFromJs for VecDeque<T> {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        js_to_list(js)
    }
}

impl<T: TryFromJs + Hash + Eq, H: BuildHasher + Default> TryFromJs for HashSet<T, H> {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        js_to_list(js)
    }
}

impl<T: TryFromJs> TryFromJs for BTreeSet<T>
where
    Self: FromIterator<T>,
{
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        js_to_list(js)
    }
}

impl<T: TryFromJs> TryFromJs for LinkedList<T> {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        js_to_list(js)
    }
}

impl TryFromJs for usize {
    #[allow(clippy::cast_possible_truncation)]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        cfg_if! {
            if #[cfg(target_pointer_width = "64")] {
                Ok(u64::from_js(js)? as usize)
            } else if #[cfg(target_pointer_width = "32")] {
                Ok(u32::from_js(js)? as usize)
            } else if #[cfg(target_pointer_width = "16")] {
                Ok(u16::from_js(js)? as usize)
            } else {
                let big = u128::from_js(js)?;
                if let Ok(out) = big.try_into() {
                    Ok(out)
                } else {
                    Err(SimpleValueError::TooLarge(f64::MAX))
                }
            }
        }
    }
}

impl TryFromJs for isize {
    #[allow(clippy::cast_possible_truncation)]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        cfg_if! {
            if #[cfg(target_pointer_width = "64")] {
                Ok(i64::from_js(js)? as isize)
            } else if #[cfg(target_pointer_width = "32")] {
                Ok(i32::from_js(js)? as isize)
            } else if #[cfg(target_pointer_width = "16")] {
                Ok(i16::from_js(js)? as isize)
            } else {
                let big = i128::from_js(js)?;
                if let Ok(out) = big.try_into() {
                    Ok(out)
                } else if big < 0 {
                    Err(SimpleValueError::TooSmall(f64::MIN))
                } else  {
                    Err(SimpleValueError::TooLarge(f64::MAX))
                }
            }
        }
    }
}

macro_rules! minmax {
    ($min: expr, $max: expr, $js: ident, $as: ty, $err_ident: ident) => {{
        let float = <f64 as TryFromJs>::from_js($js)?;

        if float > $max {
            Err($crate::error::SimpleValueError::TooLarge(float))
        } else if float < $min {
            Err($crate::error::SimpleValueError::$err_ident(float))
        } else {
            Ok(float as $as)
        }
    }};
}

macro_rules! impl_from_js {
    (direct > $for: ty, $method: ident, $err: ident) => {
        impl TryFromJs for $for {
            fn from_js(js: wasm_bindgen::JsValue) -> Result<Self, $crate::error::SimpleValueError> {
                match js.$method() {
                    Some(v) => Ok(v),
                    None => Err($crate::error::SimpleValueError::$err(js)),
                }
            }
        }
    };
    (signed, $err_variant: ident > $($ty: ty),+) => {
        $(
            impl TryFromJs for $ty {
                #[allow(clippy::cast_possible_truncation)]
                fn from_js(js: wasm_bindgen::JsValue) -> Result<Self, $crate::error::SimpleValueError> {
                    const MAX: f64 = <$ty>::MAX as f64;
                    const MIN: f64 = <$ty>::MIN as f64;
                    minmax!(MIN, MAX, js, $ty, $err_variant)
                }
            }
        )+
    };
    (unsigned, $err_variant: ident > $($ty: ty),+) => {
        $(
            impl TryFromJs for $ty {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                fn from_js(js: wasm_bindgen::JsValue) -> Result<Self, $crate::error::SimpleValueError> {
                    const MAX: f64 = <$ty>::MAX as f64;
                    minmax!(0.0, MAX, js, $ty, $err_variant)
                }
            }
        )+
    };
    (big > $($ty: ty),+ $(,)?) => {
        $(impl TryFromJs for $ty {
            fn from_js(js: wasm_bindgen::JsValue) -> Result<Self, $crate::error::SimpleValueError> {
                match js.try_into() {
                    Ok(v) => Ok(v),
                    Err(js) => Err($crate::error::SimpleValueError::DynCast(js)),
                }
            }
        })+
    };
}

impl_from_js!(direct > f64, as_f64, NotANumber);
impl_from_js!(direct > bool, as_bool, NotABoolean);

impl_from_js!(unsigned, Signed > u8, u16, u32);
impl_from_js!(signed, TooSmall > i8, i16, i32);
impl_from_js!(big > i64, u64, i128, u128);
