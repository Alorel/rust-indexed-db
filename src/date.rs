//! Date handling. Re-exports the [`web_time`](::web_time) crate.
//!
//! Note that your code should try to consistently use either `serde` or primitives for date serialisation;
//! `serde` serialises [`SystemTime`] to `{secs_since_epoch: u64, nanos_since_epoch: u32}` while the primitives
//! serialise it to a [`js_sys::Date`].
//!
//! The [`SerialiseToJs`](crate::serde::SerialiseToJs) wrapper trait is able to offer its own serialisation to
//! [typed arrays](crate::typed_array::TypedArray) because `js_sys` typed arrays do not implement
//! [`Serialize`](serde::Serialize); `SystemTime`, however, does, therefore implementing `SerialiseToJs` for
//! `SystemTime` would create a trait implementation conflict.

#![allow(rustdoc::redundant_explicit_links)]

use js_sys::Reflect;
use wasm_bindgen::prelude::*;
pub use web_time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

use crate::error::SimpleValueError;
use crate::primitive::{TryFromJs, TryToJs};

const K_SECS: &str = "secs_since_epoch";
const K_NANOS: &str = "nanos_since_epoch";

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub(crate) fn systime_from_date(js_date: &js_sys::Date) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_millis(js_date.get_time() as u64)
}

/// Try to deserialise a date that's been serialised with `serde`
fn try_primitive_js_deserialise(js: &JsValue) -> Result<Option<SystemTime>, SimpleValueError> {
    let secs = match Reflect::get(js, &JsValue::from_str(K_SECS)) {
        Ok(s) if !s.is_undefined() => s,
        _ => return Ok(None),
    };
    let nanos = match Reflect::get(js, &JsValue::from_str(K_NANOS)) {
        Ok(n) if !n.is_undefined() => n,
        _ => return Ok(None),
    };

    let dur = Duration::new(u64::from_js(secs)?, u32::from_js(nanos)?);
    Ok(Some(SystemTime::UNIX_EPOCH + dur))
}

impl TryFromJs for SystemTime {
    fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
        match js.dyn_into::<js_sys::Date>() {
            Ok(js_date) => Ok(systime_from_date(&js_date)),
            Err(e) => match try_primitive_js_deserialise(&e)? {
                Some(st) => Ok(st),
                None => Err(SimpleValueError::DynCast(e)),
            },
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
