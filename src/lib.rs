//! Wraps the [web_sys](https://docs.rs/web_sys) Indexed DB API in a Future-based API and
//! removes the pain of dealing with JS callbacks or `JSValue` in Rust.
//!
//! [![master CI badge](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml/badge.svg)](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml)
//! [![crates.io badge](https://img.shields.io/crates/v/indexed_db_futures)](https://crates.io/crates/indexed_db_futures)
//! [![docs.rs badge](https://img.shields.io/docsrs/indexed_db_futures?label=docs.rs)](https://docs.rs/indexed_db_futures)
//! [![dependencies badge](https://img.shields.io/librariesio/release/cargo/indexed_db_futures)](https://libraries.io/cargo/indexed_db_futures)
//!

//! # Overall API design
//!
//! This library implements the same structs and methods as the JavaScript API - there should be no learning curve
//! involved if you're familiar with it.
//!
//! ## Primitives
//!
//! In the context of this library, primitives refer to types that would be considered scalar primitives in JavaScript
//! (bar some feature-flagged exceptions) and are converted using the [`TryToJs`](primitive::TryToJs) &
//! [`TryFromJs`](primitive::TryFromJs) traits. They are meant to be quickly derivable from
//! [`JsValue`](wasm_bindgen::JsValue), e.g. `String` is easily derivable via
//! [`JsValue::as_string`](wasm_bindgen::JsValue::as_string).
//!
//! ## Builders
//!
//! Most API calls are constructed using builders which, in turn, get built using one of the following traits:
//!
//! - [`BuildPrimitive`] - implemented for requests use primitive serialisation.
//! - [`BuildSerde`] - implemented for requests that use [`serde`](::serde) serialisation.
//! - [`Build`] - implemented for requests that aren't `serde` _or_ primitive-serialisable (e.g. creating an index).
//!   Implemented automatically for any type that implements [`BuildPrimitive`]. As a convenience method, types that
//!   implement [`Build`] or [`BuildPrimitive`] also implement [`IntoFuture`](std::future::IntoFuture).
//!
//! Note that API requests go out immediately after being built, not after being `await`ed.

//! # Transactions default to rolling back
//!
//! ❗ Unlike Javascript, transactions will roll back by default instead of committing - this design choice was made to
//! allow code to use `?`s. There is one browser compatibility-related caveat, however - see comment on
//! [`Transaction::abort`](transaction::Transaction::abort) for more details.
//!

//! # Multi-threaded executor
//!
//! You will likely run into issues if your app is compiled with `#[cfg(target_feature = "atomics")]` as reported in
//! [#33](https://github.com/Alorel/rust-indexed-db/issues/33).
//!
//! Transactions auto-commit on `JavasScript`'s end on the next tick of the event loop if there are no outstanding
//! requests active; this isn't a problem in the default single-threaded executor, but, in a multi-threaded environment,
//! `wasm-bindgen-futures` needs to schedule our closures on the next tick as well which causes transactions to
//! prematurely auto-commit.
//!
//! As a workaround, you can try only `awaiting` individual requests after committing your transaction (requests go out
//! after being built, not after being polled).
//!
//! ```
//! # use indexed_db_futures::prelude::*;
//! # use indexed_db_futures::transaction::TransactionMode;
//! #
//! # async fn example(db: indexed_db_futures::database::Database) -> indexed_db_futures::Result<()> {
//! let transaction = db.transaction("my_store").with_mode(TransactionMode::Readwrite).build()?;
//! let object_store = transaction.object_store("my_store")?;
//!
//! let req1 = object_store.add("foo").primitive()?;
//! let req2 = object_store.add("bar").primitive()?;
//!
//! transaction.commit().await?;
//!
//! req1.await?;
//! req2.await?;
//! # Ok(())
//! # }
//! ```
//!
//! Alternatively, you can check out the [`indexed_db`](https://crates.io/crates/indexed_db) crate which explicitly
//! focuses on multi-threaded support at the cost of ergonomics.
//!

//! # Examples
//!
//! ## Opening a database & making some schema changes
//!
//! ```
//! use indexed_db_futures::database::{Database, VersionChangeEvent};
//! use indexed_db_futures::prelude::*;
//! use indexed_db_futures::transaction::{Transaction, TransactionMode};
//!
//! # async fn example() -> indexed_db_futures::OpenDbResult<()> {
//! # #[allow(dead_code)]
//! # #[cfg(all(feature = "async-upgrade", feature = "tx-done"))]
//! let db = Database::open("my_db")
//!     .with_version(2u8)
//!     .with_on_blocked(|event| {
//!       log::debug!("DB upgrade blocked: {:?}", event);
//!       Ok(())
//!     })
//!     .with_on_upgrade_needed_fut(async |event: VersionChangeEvent, tx: &Transaction<'_>| {
//!         let db = tx.db();
//!         // Convert versions from floats to integers to allow using them in match expressions
//!         let old_version = event.old_version() as u64;
//!         let new_version = event.new_version().map(|v| v as u64);
//!
//!         match (old_version, new_version) {
//!             (0, Some(1)) => {
//!                 db.create_object_store("my_store")
//!                     .with_auto_increment(true)
//!                     .build()?;
//!             }
//!             (prev, Some(2)) => {
//!                 if prev == 1 {
//!                     if let Err(e) = db.delete_object_store("my_store") {
//!                       log::error!("Error deleting v1 object store: {}", e);
//!                     }
//!                 }
//!
//!                 // Create an object store and await its transaction before inserting data.
//!                 db.create_object_store("my_other_store")
//!                   .with_auto_increment(true)
//!                   .build()?
//!                   .transaction()
//!                   .on_done()?
//!                   .await
//!                   .into_result()?;
//!
//!                 //- Start a new transaction & add some data
//!                 let tx = db.transaction("my_other_store")
//!                   .with_mode(TransactionMode::Readwrite)
//!                   .build()?;
//!                 let store = tx.object_store("my_other_store")?;
//!                 store.add("foo").await?;
//!                 store.add("bar").await?;
//!                 tx.commit().await?;
//!             }
//!             _ => {}
//!         }
//!
//!         Ok(())
//!     })
//!     .await?;
//! #    Ok(())
//! # }
//! ```
//!
//! ## Reading/writing with `serde`
//!
//! ```
//! # #[cfg(feature = "serde")]
//! # mod wrapper {
//! # use indexed_db_futures::object_store::ObjectStore;
//! # use indexed_db_futures::prelude::*;
//! # use serde::{Deserialize, Serialize};
//! #
//! #[derive(Serialize, Deserialize)]
//! struct UserRef {
//!   id: u32,
//!   name: String,
//! }
//!
//! # #[allow(dead_code)]
//! # async fn example(object_store: ObjectStore<'_>) -> indexed_db_futures::Result<()> {
//! object_store.put(UserRef { id: 1, name: "Bobby Tables".into() }).serde()?.await?;
//! let user: Option<UserRef> = object_store.get(1u32).serde()?.await?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! # Iterating a cursor
//!
//! ```
//! # use indexed_db_futures::object_store::ObjectStore;
//! # use indexed_db_futures::prelude::*;
//! #
//! # #[allow(dead_code)]
//! # #[cfg(feature = "cursors")]
//! # async fn example(object_store: ObjectStore<'_>) -> indexed_db_futures::Result<()> {
//! let Some(mut cursor) = object_store.open_cursor().await? else {
//!   log::debug!("Cursor empty");
//!   return Ok(());
//! };
//!
//! // Retrieve the next record in the stream, expecting a String
//! let next: Option<String> = cursor.next_record().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Iterating an index as a stream
//!
//! ```
//! # #[cfg(all(feature = "serde", feature = "indices", feature = "cursors", feature = "streams"))]
//! # mod wrapper {
//! # use indexed_db_futures::object_store::ObjectStore;
//! # use indexed_db_futures::prelude::*;
//! # use serde::{Deserialize, Serialize};
//! use futures::TryStreamExt;
//! #
//! # #[derive(Serialize, Deserialize)]
//! # struct UserRef {
//! #  id: u32,
//! #  name: String,
//! # }
//!
//! # #[allow(dead_code)]
//! # async fn example(object_store: ObjectStore<'_>) -> indexed_db_futures::Result<()> {
//! let index = object_store.index("my_index")?;
//! let Some(cursor) = index.open_cursor().with_query(10u32..=100u32).serde()?.await? else {
//!   log::debug!("Cursor empty");
//!   return Ok(());
//! };
//! let stream = cursor.stream_ser::<UserRef>();
//! let records = stream.try_collect::<Vec<_>>().await?;
//! # Ok(())
//! # }
//! # }
//! ```
//!

//! # Environment support
//!
//! The following table is populated **as a best effort attempt** based on the crate's unit tests succeeding/failing
//! under different configurations.
//!
//! | Environment      | Chrome | Firefox | Safari |
//! |------------------|---------|---------|--------|
//! | Browser          |   ✅    |   ✅    |   ✅   |
//! | Dedicated Worker |  ✅     |   ✅    |   ✅   |
//! | Shared Worker    | ✅      |   ✅    |   ✅   |
//! | Worker           |  ✅    |   ✅    |   ✅   |
//! | Service worker   | ✅    |   ❌    |   ✅   |
//!

//! # Feature table
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `async-upgrade` | Enable async closures in [`upgradeneeded`](https://developer.mozilla.org/en-US/docs/Web/API/IDBOpenDBRequest/upgradeneeded_event) event listeners. |
//! | `cursors` | Enable opening IndexedDB [cursors](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor). |
//! | `dates` | Enable [`SystemTime`](std::time::SystemTime) & [`Date`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date) handling. |
//! | `indices` | Enable IndexedDB [indices](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex). |
//! | `list-databases` | Enable getting a list of defined databases. |
//! | `serde` | Enable [`serde`](::serde) integration. |
//! | `streams` | Implement [`Stream`](::futures_core::Stream) where applicable. |
//! | `switch` | Enable [switches](primitive::Switch2). |
//! | `tx-done` | Enable waiting for transactions to complete without consuming them. |
//! | `typed-arrays` | Enable [typed array](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray) handling. |
//! | `version-change` | Enable listening for [`versionchange`](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/versionchange_event) events. |

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
