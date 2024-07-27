//! Wraps the [web_sys](https://crates.io/crates/web_sys) Indexed DB API in a Future-based API and
//! removes the pain of dealing with Javascript callbacks in Rust.
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
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]

pub use web_sys::IdbTransactionMode as TransactionMode;

pub use database::{Database, ObjectStoreParameters, VersionChangeEvent};
pub use factory::{DBFactory, DBVersion, OpenDbRequestBuilder};
pub use key_path::KeyPath;
pub use key_range::KeyRange;
pub use object_store::ObjectStore;
pub use primitive::{FromJs, ToJs, TryToJs};
pub use query_source::QuerySource;
pub use transaction::{Transaction, TransactionDurability, TransactionOptions};

macro_rules! iffeat {
    (#[cfg(feature = $feat: literal)] $($stmt: item)+) => {
        $(
            #[cfg(feature = $feat)]
            $stmt
        )+
    };
}

macro_rules! maybe_errored_dom {
    ($expr: expr, |$req: ident| $then: expr, |$e: ident| $catch: expr) => {
        match $expr {
            Ok($req) => $crate::future::MaybeErrored::running($then),
            Err($e) => {
                let $e: $crate::error::Error = $e.into();
                $crate::future::MaybeErrored::errored($catch)
            }
        }
    };
    (into, $expr: expr, |$req: ident| $then: expr) => {
        maybe_errored_dom!($expr, |$req| $then, |e| e.into())
    };
    ($expr: expr, |$req: ident| $then: expr) => {
        maybe_errored_dom!($expr, |$req| $then, |e| e)
    };
}

macro_rules! maybe_errored {
    ($expr: expr, |$req: ident| $then: expr) => {
        match $expr {
            Ok($req) => $crate::future::MaybeErrored::running($then),
            Err(e) => $crate::future::MaybeErrored::errored(e),
        }
    };
}

macro_rules! fwd_cast_js {
    (FromJs: $($ty: ty),+ $(,)?) => {
        $(
            #[sealed]
            impl $crate::primitive::FromJs for $ty {
                fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
                    js.dyn_into().map_err(SimpleValueError::DynCast)
                }
            }
        )+
    };
    (ToJs: $($ty: ty),+ $(,)?) => {
        $(
            #[sealed]
            impl $crate::primitive::ToJs for $ty {
                fn to_js(&self) -> JsValue {
                    self.unchecked_ref::<JsValue>().clone()
                }
            }
        )+
    };
    ($($ty: ty),+ $(,)?) => {
        fwd_cast_js!(FromJs: $($ty),+);
        fwd_cast_js!(ToJs: $($ty),+);
    };
}

#[cfg(test)]
mod test_util;

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[cfg(test)]
macro_rules! random_obj_store {
    (r => $db: ident, $tx: ident, $store: ident $block: block) => {
        let ($db, name) = $crate::test_util::random_store().await;
        let $tx = $db.transaction(&name).expect("Error starting transaction");
        let $store = $tx.object_store(&name).expect("Error getting object store");
        drop(name);
        $block
    };
}

mod database;
pub mod error;
pub mod future;
pub mod iter;

mod query_source;

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, error::Error>;

#[allow(missing_docs)]
pub type OpenDbResult<T> = std::result::Result<T, error::OpenDbError>;

#[allow(missing_docs)]
pub type OpenDbOpResult<T, B = error::Error, U = error::Error> =
    std::result::Result<T, error::OpenDbOpError<B, U>>;

mod internal_utils;
mod key_path;
mod key_range;
mod object_store;
pub mod prelude;
mod primitive;
mod transaction;

#[cfg(feature = "dates")]
pub mod date;
mod factory;

iffeat! {
    #[cfg(feature = "cursors")]
    mod cursor;
    pub use cursor::Cursor;
}

iffeat! {
    #[cfg(feature = "indices")]
    mod index;
    pub use index::Index;
}
