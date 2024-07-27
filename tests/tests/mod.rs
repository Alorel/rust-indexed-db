macro_rules! assert_dom_exc {
    ($err: expr, $dom_exc: ident) => {{
        let err = $err;
        let m = matches!(
            err,
            ::indexed_db_futures::error::Error::DomException(
                ::indexed_db_futures::error::DomException::$dom_exc(_)
            )
        );
        assert!(m, "{err:?}")
    }};
    (open $err: expr, $dom_exc: ident) => {{
        let err = $err;
        let m = matches!(
            err,
            ::indexed_db_futures::error::OpenDbError::Base(
                ::indexed_db_futures::error::Error::DomException(
                    ::indexed_db_futures::error::DomException::$dom_exc(_)
                )
            )
        );
        assert!(m, "{err:?}")
    }};
}

macro_rules! open_tx {
    ($db: expr, $mode: ident > ($tx: ident, $store: ident named $store_name: expr)) => {
        let $tx = ::indexed_db_futures::Build::build(
            $db.transaction(&$db.name()).with_mode(::indexed_db_futures::transaction::TransactionMode::$mode)
        ).expect("tx build");
        let $store = $tx.object_store($store_name).expect("object_store()");
    };
    ($db: expr, $mode: ident > ($tx: ident, $store: ident)) => {
        open_tx!($db, $mode > ($tx, $store named &$db.name()));
    };
}

macro_rules! dyn_await {
    ($builder: expr) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                idb_fut::BuildSerde::serde($builder).expect("build()").await
            } else {
                $builder.await
            }
        }
    }};
}

macro_rules! collect {
    ($req: expr) => {
        dyn_await!($req)
            .expect("req")
            .collect::<idb_fut::Result<Vec<_>>>()
            .expect("collect")
    };
}

#[cfg(feature = "cursors")]
macro_rules! next_key {
    ($ty: ty, $cursor: expr) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                $cursor.next_key_ser::<$ty>()
            } else {
                $cursor.next_key::<$ty>()
            }
        }
    }
    .await};
}

#[cfg(feature = "cursors")]
macro_rules! primary_key {
    ($ty: ty, $cursor: expr) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                $cursor.primary_key_ser::<$ty>()
            } else {
                $cursor.primary_key::<$ty>()
            }
        }
    }};
}

#[cfg(feature = "cursors")]
macro_rules! continue_to_key {
    ($key: expr, $cursor: expr) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                $cursor.continue_to_key_ser($key)
            } else {
                $cursor.continue_to_key($key)
            }
        }
    }
    .await};
}

#[cfg(all(feature = "cursors", feature = "indices"))]
macro_rules! continue_to_pkey {
    ($key: expr, $pkey: expr, $cursor: expr) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                $cursor.continue_to_primary_key_ser($key, $pkey)
            } else {
                $cursor.continue_to_primary_key($key, $pkey)
            }
        }
    }
    .await};
}

#[cfg(feature = "cursors")]
macro_rules! next_record {
    ($ty: ty, $cursor: expr) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                $cursor.next_record_ser::<$ty>()
            } else {
                $cursor.next_record::<$ty>()
            }
        }
    }
    .await};
}

#[cfg(all(feature = "streams", feature = "cursors"))]
macro_rules! open_kstream {
    ($cursor: ident, $ty: ty) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                $cursor.key_stream_ser::<$ty>()
            } else {
                $cursor.key_stream::<$ty>()
            }
        }
    }};
}

#[cfg(all(feature = "streams", feature = "cursors"))]
macro_rules! open_stream {
    ($cursor: ident, $ty: ty) => {{
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "serde")] {
                $cursor.stream_ser::<$ty>()
            } else {
                $cursor.stream::<$ty>()
            }
        }
    }};
}

pub mod utils;

pub mod database;
#[cfg(feature = "dates")]
pub mod date;
#[cfg(feature = "indices")]
pub mod index;
pub mod key_path;
pub mod object_store;
pub mod primitive;
pub mod transaction;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::utils::dummy_data::*;
    pub use super::utils::init::*;
    pub use super::utils::{random_str, BuildDyn};
    pub use idb_fut::database::VersionChangeEvent;
    pub use idb_fut::prelude::*;
    pub use idb_fut::primitive::{TryFromJs, TryToJs};
    pub use indexed_db_futures as idb_fut;
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen_test::wasm_bindgen_test;

    #[cfg(feature = "_serialise-deserialise-dyn")]
    pub use super::utils::{DeserialiseDyn, SerialiseDyn};

    pub use futures::{Stream, StreamExt, TryFuture, TryFutureExt, TryStream, TryStreamExt};
}
