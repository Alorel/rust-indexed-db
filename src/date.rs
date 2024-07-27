//! Date handling. Re-exports the [`web_time`] crate.

use sealed::sealed;
use wasm_bindgen::prelude::*;
pub use web_time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

use crate::error::SimpleValueError;

fwd_cast_js!(js_sys::Date);

#[sealed]
#[allow(unused_qualifications)]
impl crate::primitive::FromJs for SystemTime {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        match js.dyn_into::<js_sys::Date>() {
            Ok(js_date) => {
                Ok(SystemTime::UNIX_EPOCH + Duration::from_millis(js_date.get_time() as u64))
            }
            Err(e) => Err(SimpleValueError::DynCast(e)),
        }
    }
}

#[sealed]
#[allow(unused_qualifications)]
impl crate::primitive::TryToJs for SystemTime {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        match self.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let duration = JsValue::from_f64(duration.as_millis() as f64);
                Ok(js_sys::Date::new(&duration).unchecked_into())
            }
            Err(e) => Err(SimpleValueError::cast_err(e).into()),
        }
    }
}

#[sealed]
#[allow(unused_qualifications)]
impl crate::primitive::TryToJs for &SystemTime {
    #[inline]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        <SystemTime as crate::primitive::TryToJs>::try_to_js(self)
    }
}

macro_rules! impl_common {
    (opt>$($ty: ty),+ $(,)?) => {
        $(
            #[sealed]
            #[allow(unused_qualifications)]
            impl crate::primitive::TryToJs for $ty {
                fn try_to_js(&self) -> crate::Result<JsValue> {
                    match self {
                        Some(v) => v.try_to_js(),
                        None => Ok(JsValue::UNDEFINED),
                    }
                }
            }
        )+
    };
}

impl_common!(opt > Option<SystemTime>, Option<&SystemTime>, &Option<SystemTime>,  &Option<&SystemTime>);
