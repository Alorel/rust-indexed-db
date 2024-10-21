//! Types for working with `wasm-bindgen` and `js-sys` primitive that
//! don't require [`serde`] for converting between Rust & JS.

mod from_js;

mod try_to_js;

pub use {from_js::*, try_to_js::*};

iffeat! {
    #[cfg(feature = "switch")]
    mod switch;
    pub use switch::*;
}

fwd_cast_js!(
    web_sys::DomException,
    js_sys::Error,
    js_sys::Date,
    js_sys::JsString,
    js_sys::Number,
    js_sys::ArrayBuffer,
    js_sys::Array,
    js_sys::Object,
);
