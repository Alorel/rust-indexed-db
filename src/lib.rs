//! Wraps the [web_sys](https://docs.rs/web_sys) Indexed DB API in a Future-based API and
//! removes the pain of dealing with JS callbacks or `JSValue` in Rust.
//!
//! [![master CI badge](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml/badge.svg)](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml)
//! [![crates.io badge](https://img.shields.io/crates/v/indexed_db_futures)](https://crates.io/crates/indexed_db_futures)
//! [![docs.rs badge](https://img.shields.io/docsrs/indexed_db_futures?label=docs.rs)](https://docs.rs/indexed_db_futures)
//! [![dependencies badge](https://img.shields.io/librariesio/release/cargo/indexed_db_futures)](https://libraries.io/cargo/indexed_db_futures)
//!

#![deny(clippy::correctness, clippy::suspicious)]
#![warn(
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    missing_docs
)]
#![allow(clippy::module_name_repetitions, clippy::wildcard_imports)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]

pub use build::*;
pub use key_path::{KeyPath, KeyPathSeq};
pub use key_range::KeyRange;

/// `cfg_if` doesn't add appropriate tags in docs.rs
macro_rules! iffeat {
    (#[cfg(feature = $feat: literal)] $($stmt: item)+) => {
        $(
            #[cfg(feature = $feat)]
            $stmt
        )+
    };
}

macro_rules! fwd_cast_js {
    (TryFromJs: $($ty: ty),+ $(,)?) => {
        $(
            impl $crate::primitive::TryFromJs for $ty {
                fn from_js(js: ::wasm_bindgen::JsValue) -> Result<Self, $crate::error::SimpleValueError> {
                    ::wasm_bindgen::JsCast::dyn_into(js).map_err($crate::error::SimpleValueError::DynCast)
                }
            }
        )+
    };
    (TryToJs: $($ty: ty),+ $(,)?) => {
        $(
            #[allow(unused_qualifications)]
            impl $crate::primitive::try_to_js::TryToJs for $ty {
                fn try_to_js(&self) -> $crate::Result<::wasm_bindgen::JsValue> {
                    Ok(::wasm_bindgen::JsCast::unchecked_ref::<::wasm_bindgen::JsValue>(self).clone())
                }
            }
        )+
    };
    ($($ty: ty),+ $(,)?) => {
        fwd_cast_js!(TryFromJs: $($ty),+);
        fwd_cast_js!(TryToJs: $($ty),+);
    };
}

#[allow(unused_macros)]
macro_rules! log {
    ($level: ident, $($arg:tt)+) => {
        #[cfg(feature = "__unit-tests")]
        ::log::log!(::log::Level::$level, $($arg)+);
    };
}

pub mod database;
pub mod error;
pub mod future;
pub mod iter;

pub mod query_source;

/// A [`Result`](std::result::Result) with an [`Error`](error::Error) as the error type.
pub type Result<T> = std::result::Result<T, error::Error>;

/// A [`Result`](std::result::Result) with an [`OpenDbError`](error::OpenDbError) as the error type.
pub type OpenDbResult<T> = std::result::Result<T, error::OpenDbError>;

mod internal_utils;
mod key_path;
mod key_range;
pub mod object_store;
pub mod prelude;
pub mod primitive;
pub mod transaction;

#[cfg(feature = "dates")]
pub mod date;
pub mod factory;

mod build;
#[cfg(feature = "typed-arrays")]
pub mod typed_array;

#[cfg(feature = "cursors")]
pub mod cursor;
#[cfg(feature = "indices")]
pub mod index;

pub mod internals;

iffeat! {
    #[cfg(feature = "serde")]
    mod serde;
    pub use serde::{DeserialiseFromJs, SerialiseToJs};
}
