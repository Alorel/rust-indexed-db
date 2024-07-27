use sealed::sealed;
use wasm_bindgen::prelude::*;

use crate::error::SimpleValueError;

/// Convert to a JS value
#[sealed(pub(crate))]
pub trait ToJs {
    /// Convert to a JS value
    #[must_use]
    fn to_js(&self) -> JsValue;
}

/// Convert to a JS value
#[sealed(pub(crate))]
pub trait TryToJs {
    /// Convert to a JS value
    fn try_to_js(&self) -> crate::Result<JsValue>;
}

/// Convert from a JS value.
#[sealed(pub(crate))]
pub trait FromJs {
    /// Convert from a JS value.
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError>
    where
        Self: Sized;
}

#[sealed]
impl<T: ToJs> ToJs for &T {
    #[inline]
    fn to_js(&self) -> JsValue {
        <T as ToJs>::to_js(self)
    }
}

#[sealed]
impl<T: ToJs> ToJs for Option<T> {
    fn to_js(&self) -> JsValue {
        match self {
            Some(v) => v.to_js(),
            None => JsValue::UNDEFINED,
        }
    }
}

#[sealed]
impl<T: FromJs> FromJs for Option<T> {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        if js.is_undefined() || js.is_null() {
            Ok(None)
        } else {
            T::from_js(js).map(Some)
        }
    }
}

#[sealed]
impl<T: ToJs> TryToJs for T {
    #[inline]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        Ok(self.to_js())
    }
}

#[sealed]
impl FromJs for f32 {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        const MAX: f64 = f32::MAX as f64;
        let big = <f64 as FromJs>::from_js(js)?;

        if big > MAX {
            Err(SimpleValueError::TooLarge(big))
        } else {
            Ok(big as f32)
        }
    }
}

#[sealed]
impl FromJs for i128 {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        Ok(<f64 as FromJs>::from_js(js)? as i128)
    }
}

#[sealed]
impl FromJs for u128 {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        let float = <f64 as FromJs>::from_js(js)?;

        if float < 0.0 {
            Err(SimpleValueError::Signed(float))
        } else {
            Ok(float as u128)
        }
    }
}

#[sealed]
impl ToJs for &str {
    #[inline]
    fn to_js(&self) -> JsValue {
        JsValue::from_str(self)
    }
}

#[sealed]
impl ToJs for String {
    #[inline]
    fn to_js(&self) -> JsValue {
        self.as_str().to_js()
    }
}

#[sealed]
impl FromJs for String {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        match js.as_string() {
            Some(v) => Ok(v),
            None => Err(SimpleValueError::NotAString(js)),
        }
    }
}

#[sealed]
impl FromJs for JsValue {
    #[inline]
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        Ok(js)
    }
}

#[sealed]
impl ToJs for JsValue {
    #[inline]
    fn to_js(&self) -> JsValue {
        self.clone()
    }
}

macro_rules! minmax {
    ($min: expr, $max: expr, $js: ident, $as: ty, $err_ident: ident) => {{
        let float = <f64 as FromJs>::from_js($js)?;

        if float > $max {
            Err(SimpleValueError::TooLarge(float))
        } else if float < $min {
            Err(SimpleValueError::$err_ident(float))
        } else {
            Ok(float as $as)
        }
    }};
}

macro_rules! impl_to_js {
    (into > $($ty: ident),+ $(,)?) => {
        $(
            #[sealed]
            impl ToJs for $ty {
                #[inline]
                fn to_js(&self) -> JsValue {
                    (*self).into()
                }
            }
        )+
    };
}

macro_rules! impl_from_js {
    (direct > $for: ty, $method: ident, $err: ident) => {
        #[sealed]
        impl FromJs for $for {
            fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
                match js.$method() {
                    Some(v) => Ok(v),
                    None => Err(SimpleValueError::$err(js)),
                }
            }
        }
    };
    (signed, $err_variant: ident > $($ty: ty),+) => {
        $(
            #[sealed]
            impl FromJs for $ty {
                fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
                    const MAX: f64 = <$ty>::MAX as f64;
                    const MIN: f64 = <$ty>::MIN as f64;
                    minmax!(MIN, MAX, js, $ty, $err_variant)
                }
            }
        )+
    };
    (unsigned, $err_variant: ident > $($ty: ty),+) => {
        $(
            #[sealed]
            impl FromJs for $ty {
                fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
                    const MAX: f64 = <$ty>::MAX as f64;
                    minmax!(0.0, MAX, js, $ty, $err_variant)
                }
            }
        )+
    };
}

impl_to_js!(
    into > f32,
    f64,
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
    i128,
    u128
);

impl_from_js!(direct > f64, as_f64, NotANumber);
impl_from_js!(direct > bool, as_bool, NotABoolean);

impl_from_js!(unsigned, Signed > u8, u16, u32, u64, usize);
impl_from_js!(signed, TooSmall > i8, i16, i32, i64, isize);

fwd_cast_js!(
    web_sys::DomException,
    js_sys::Error,
    js_sys::JsString,
    js_sys::Number,
    js_sys::ArrayBuffer,
    js_sys::Array,
    js_sys::Object,
);
