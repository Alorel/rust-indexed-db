//! Wraps the [web_sys](https://crates.io/crates/web_sys) Indexed DB API in a Future-based API and
//! removes the pain of dealing with Javascript callbacks in Rust.
//!
//! [![master CI badge](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml/badge.svg)](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml)
//! [![crates.io badge](https://img.shields.io/crates/v/indexed_db_futures)](https://crates.io/crates/indexed_db_futures)
//! [![docs.rs badge](https://img.shields.io/docsrs/indexed_db_futures?label=docs.rs)](https://docs.rs/indexed_db_futures)
//! [![dependencies badge](https://img.shields.io/librariesio/release/cargo/indexed_db_futures)](https://libraries.io/cargo/indexed_db_futures)
//!
//! ## Overall API design
//!
//! In most cases API methods will return a [`Result`](::std::result::Result) containing a wrapped
//! [`IdbRequest`](crate::web_sys::IdbRequest) that implements [`IntoFuture`](::std::future::IntoFuture), such as
//! [`VoidRequest`](crate::request::VoidRequest), or, when more appropriate, the [`Future`](::std::future::Future)
//! directly, e.g. [`CountFuture`](crate::request::CountFuture).
//!
//! The key difference between a wrapped Request and Future is that Requests don't have _any_ event
//! listeners attached, which aims to make quickfire operations such as inserting several records
//! into an [`IdbObjectStore`](crate::idb_object_store::IdbObjectStore) a little bit more efficient.
//!
//! ## Features
//!
//! The library can ship without cursor or index support for apps that just need a simple key-value
//! store akin to `localStorage`.
//!
//! - `cursors` - Enable cursor support
//! - `indices` - Enable index support
//! - `nightly` - Use unsafe nightly features where appropriate, such as [`unwrap_unchecked`](Option::unwrap_unchecked).
//! - `default`:
//!    - `cursors`
//!    - `indices`
//!
//! ## Examples
//!
//! ### Connecting to a DB and doing basic CRUD
//!
//! Variable types included for clarity.
//!
//! ```rust
//! use indexed_db_futures::prelude::*;
//!# use web_sys::DomException;
//!# use wasm_bindgen::prelude::*;
//!#
//!# fn use_value(_v: Option<JsValue>) {}
//!# fn get_some_js_value() -> JsValue { JsValue::UNDEFINED }
//!
//! pub async fn example() -> Result<(), DomException> {
//!     // Open my_db v1
//!     let mut db_req: OpenDbRequest = IdbDatabase::open_u32("my_db", 1)?;
//!     db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
//!         // Check if the object store exists; create it if it doesn't
//!         if let None = evt.db().object_store_names().find(|n| n == "my_store") {
//!             evt.db().create_object_store("my_store")?;
//!         }
//!         Ok(())
//!     }));
//!
//!     let db: IdbDatabase = db_req.await?;
//!
//!     // Insert/overwrite a record
//!     let tx: IdbTransaction = db
//!       .transaction_on_one_with_mode("my_store", IdbTransactionMode::Readwrite)?;
//!     let store: IdbObjectStore = tx.object_store("my_store")?;
//!
//!     let value_to_put: JsValue = get_some_js_value();
//!     store.put_key_val_owned("my_key", &value_to_put)?;
//!
//!     // IDBTransactions can have an Error or an Abort event; into_result() turns both into a
//!     // DOMException
//!     tx.await.into_result()?;
//!
//!     // Delete a record
//!     let tx = db.transaction_on_one_with_mode("my_store", IdbTransactionMode::Readwrite)?;
//!     let store = tx.object_store("my_store")?;
//!     store.delete_owned("my_key")?;
//!     tx.await.into_result()?;
//!
//!     // Get a record
//!     let tx = db.transaction_on_one("my_store")?;
//!     let store = tx.object_store("my_store")?;
//!
//!     let value: Option<JsValue> = store.get_owned("my_key")?.await?;
//!     use_value(value);
//!
//!     Ok(())
//! }
//! ```

#![deny(clippy::correctness, clippy::suspicious)]
#![warn(clippy::complexity, clippy::perf, clippy::style, clippy::pedantic)]
#![warn(missing_docs)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::uninlined_format_args
)]
#![allow(rustdoc::redundant_explicit_links)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]

pub use js_sys;
pub use web_sys;

pub use idb_database::*;
pub use idb_key_path::*;
pub use idb_query_source::*;

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[cfg(test)]
macro_rules! test_mod_init {
    () => {
        #[allow(unused_imports)]
        use {
            super::*,
            wasm_bindgen::{prelude::*, JsCast},
            wasm_bindgen_test::*,
        };
    };
}

#[cfg(test)]
macro_rules! test_case {
    ($name: ident => $body: block) => {
        #[wasm_bindgen_test::wasm_bindgen_test]
        fn $name() {
            $body
        }
    };
    (async $name: ident => $body: block) => {
        #[wasm_bindgen_test::wasm_bindgen_test]
        async fn $name() {
            $body
        }
    };
}

macro_rules! impl_display_for_named {
    ($for: ty) => {
        impl std::fmt::Display for $for {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unused_imports)]
                use $crate::idb_query_source::IdbQuerySource;
                f.write_str(&self.name())
            }
        }
    };
}

macro_rules! impl_query_source {
    ($for: ty) => {
        impl_display_for_named!($for);

        impl $crate::idb_query_source::IdbQuerySource for $for {
            // Cursors
            cfg_if::cfg_if! {
                if #[cfg(feature = "cursors")] {
                    fn open_cursor(
                        &self,
                    ) -> Result<$crate::request::IdbCursorWithValueFuture<Self>, web_sys::DomException>
                    {
                        let base = $crate::request::IdbCursorFuture::new(self.inner.open_cursor(), self)?;
                        Ok($crate::request::IdbCursorWithValueFuture::new(base))
                    }

                    fn open_cursor_with_range_and_direction<K: wasm_bindgen::JsCast>(&self, range: &K, direction: web_sys::IdbCursorDirection) -> Result<$crate::request::IdbCursorWithValueFuture<Self>, web_sys::DomException> {
                        #[allow(unused_imports)]
                        use wasm_bindgen::JsCast;
                        let base = self.inner.open_cursor_with_range_and_direction(range.unchecked_ref(), direction);
                        let base = $crate::request::IdbCursorFuture::new(base, self)?;
                        Ok($crate::request::IdbCursorWithValueFuture::new(base))
                    }

                    fn open_cursor_with_range<K: wasm_bindgen::JsCast>(&self, range: &K) -> Result<$crate::request::IdbCursorWithValueFuture<Self>, web_sys::DomException> {
                        #[allow(unused_imports)]
                        use wasm_bindgen::JsCast;
                        let base = self.inner.open_cursor_with_range(range.unchecked_ref());
                        let base = $crate::request::IdbCursorFuture::new(base, self)?;
                        Ok($crate::request::IdbCursorWithValueFuture::new(base))
                    }

                    fn open_key_cursor(
                        &self,
                    ) -> Result<$crate::request::IdbCursorFuture<Self>, web_sys::DomException> {
                        $crate::request::IdbCursorFuture::new(self.inner.open_key_cursor(), self)
                    }

                    fn open_key_cursor_with_range<K: wasm_bindgen::JsCast>(&self, range: &K) -> Result<$crate::request::IdbCursorFuture<Self>, web_sys::DomException> {
                        let base = self.inner.open_key_cursor_with_range(range.unchecked_ref());
                        $crate::request::IdbCursorFuture::new(base, self)
                    }

                    fn open_key_cursor_with_range_and_direction<K: wasm_bindgen::JsCast>(&self, range: &K, direction: web_sys::IdbCursorDirection) -> Result<$crate::request::IdbCursorFuture<Self>, web_sys::DomException> {
                        let base = self.inner.open_key_cursor_with_range_and_direction(range.unchecked_ref(), direction);
                        $crate::request::IdbCursorFuture::new(base, self)
                    }
                }
            }

            #[inline]
            fn get<K: wasm_bindgen::JsCast>(
                &self,
                key: &K,
            ) -> Result<$crate::request::OptionalJsValueFuture, web_sys::DomException> {
                #[allow(unused_imports)]
                use wasm_bindgen::JsCast;
                $crate::request::OptionalJsValueFuture::new(self.inner.get(key.unchecked_ref()))
            }

            #[inline]
            fn get_all(
                &self,
            ) -> Result<$crate::request::JsCastRequestFuture<js_sys::Array>, web_sys::DomException>
            {
                $crate::request::JsCastRequestFuture::new(self.inner.get_all())
            }

            #[inline]
            fn get_all_with_key<K: wasm_bindgen::JsCast>(
                &self,
                key: &K,
            ) -> Result<$crate::request::JsCastRequestFuture<js_sys::Array>, web_sys::DomException>
            {
                #[allow(unused_imports)]
                use wasm_bindgen::JsCast;
                $crate::request::JsCastRequestFuture::new(self.inner.get_all_with_key(key.unchecked_ref()))
            }

            #[inline]
            fn count(&self) -> Result<$crate::request::CountFuture, web_sys::DomException> {
                $crate::request::CountFuture::new(self.inner.count())
            }

            #[inline]
            fn key_path(&self) -> Option<$crate::idb_key_path::IdbKeyPath> {
                $crate::idb_key_path::IdbKeyPath::try_from_js(self.inner.key_path())
            }

            #[inline]
            fn count_with_key<K: wasm_bindgen::JsCast>(
                &self,
                key: &K,
            ) -> Result<$crate::request::CountFuture, web_sys::DomException> {
                #[allow(unused_imports)]
                use wasm_bindgen::JsCast;
                $crate::request::CountFuture::new(self.inner.count_with_key(key.unchecked_ref()))
            }

            #[inline]
            fn name(&self) -> String {
                self.inner.name()
            }

            #[inline]
            fn set_name(&self, name: &str) {
                self.inner.set_name(name);
            }

            #[inline]
            fn get_key<K: wasm_bindgen::JsCast>(
                &self,
                key: &K,
            ) -> Result<$crate::request::OptionalJsValueFuture, web_sys::DomException> {
                #[allow(unused_imports)]
                use wasm_bindgen::JsCast;
                $crate::request::OptionalJsValueFuture::new(self.inner.get_key(key.unchecked_ref()))
            }

            #[inline]
            fn get_all_keys(
                &self,
            ) -> Result<$crate::request::JsCastRequestFuture<js_sys::Array>, web_sys::DomException>
            {
                $crate::request::JsCastRequestFuture::new(self.inner.get_all_keys())
            }

            #[inline]
            fn get_all_keys_with_key<K: wasm_bindgen::JsCast>(
                &self,
                key: &K,
            ) -> Result<$crate::request::JsCastRequestFuture<js_sys::Array>, web_sys::DomException>
            {
                #[allow(unused_imports)]
                use wasm_bindgen::JsCast;
                $crate::request::JsCastRequestFuture::new(self.inner.get_all_keys_with_key(key.unchecked_ref()))
            }

            #[inline]
            fn get_all_keys_with_key_and_limit<K: wasm_bindgen::JsCast>(
                &self,
                key: &K,
                limit: u32,
            ) -> Result<$crate::request::JsCastRequestFuture<js_sys::Array>, web_sys::DomException>
            {
                #[allow(unused_imports)]
                use wasm_bindgen::JsCast;
                $crate::request::JsCastRequestFuture::new(
                    self.inner.get_all_keys_with_key_and_limit(key.unchecked_ref(), limit),
                )
            }
        }
    };
}

mod idb_database;
pub mod idb_object_store;
mod idb_query_source;
pub mod idb_transaction;
mod internal_utils;
pub mod prelude;
pub mod request;

pub(crate) mod dom_string_iterator;

#[cfg(feature = "indices")]
mod idb_index;

#[cfg(feature = "indices")]
pub use idb_index::*;

#[cfg(feature = "cursors")]
pub mod idb_cursor;
mod idb_key_path;
