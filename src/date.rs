//! Date handling. Re-exports the [`web_time`] crate.
//!
//! Note that your code should consistently use either `serde` or primitives for date serialisation; `serde` serialises
//! [`SystemTime`] to `{secs_since_epoch: u64, nanos_since_epoch: u32}` while the primitives serialise it to a
//! [`js_sys::Date`]. The [`SerialiseToJs`](crate::serde::SerialiseToJs) wrapper trait is able to offer its own
//! serialisation to [typed arrays](crate::typed_array::TypedArray) because `js_sys` typed arrays do not implement
//! [`Serialize`](serde::Serialize); `SystemTime`, however, does, therefore implementing `SerialiseToJs` for
//! `SystemTime` would create a trait implementation conflict.

use wasm_bindgen::prelude::*;
pub use web_time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

use crate::error::SimpleValueError;
use crate::primitive::{TryFromJs, TryToJs};

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub(crate) fn systime_from_date(js_date: &js_sys::Date) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_millis(js_date.get_time() as u64)
}

impl TryFromJs for SystemTime {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        match js.dyn_into::<js_sys::Date>() {
            Ok(js_date) => Ok(systime_from_date(&js_date)),
            Err(e) => Err(SimpleValueError::DynCast(e)),
        }
    }
}

impl TryToJs for SystemTime {
    #[allow(clippy::cast_precision_loss)]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        match self.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let duration = JsValue::from_f64(duration.as_millis() as f64);
                Ok(js_sys::Date::new(&duration).unchecked_into())
            }
            Err(e) => Err(SimpleValueError::Other(Box::new(e)).into()),
        }
    }
}
