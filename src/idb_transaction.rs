//! Transaction-related code

use std::fmt::Debug;
use std::future::{Future, IntoFuture};

use accessory::Accessors;
use fancy_constructor::new;
pub use idb_transaction_future::IdbTransactionFuture;
pub(crate) use idb_transaction_listeners::*;
pub use idb_transaction_result::*;
use web_sys::{DomException, IdbTransactionMode};

use crate::dom_string_iterator::DomStringIterator;
use crate::idb_database::IdbDatabase;
use crate::idb_object_store::IdbObjectStore;

mod idb_transaction_future;
mod idb_transaction_listeners;
mod idb_transaction_result;

#[cfg(test)]
pub mod test {
    pub mod future {
        use crate::internal_utils::open_any_db;
        use crate::prelude::{IdbTransactionMode, IdbTransactionResult};

        test_mod_init!();

        test_case!(async should_return_object_store_names => {
            let (db, store_name) = open_any_db().await;
            let tx = db.transaction_on_multi(&[store_name.as_str()]).expect("tx");
            let store_names: Vec<String> = tx.object_store_names().collect();

            assert_eq!(store_names, vec![store_name; 1]);
        });

        test_case!(async should_resolve_on_success => {
            let (db, store_name) = open_any_db().await;
            let tx = db.transaction_on_one_with_mode(&store_name, IdbTransactionMode::Readwrite).expect("tx");
            let store = tx.object_store(&store_name).expect("store");

            store.put_key_val_owned("foo", &JsValue::from("bar")).expect("put");
            assert!(tx.await.into_result().is_ok(), "result");
        });

        test_case!(async should_propagate_errors => {
            let (db, store_name) = open_any_db().await;
            let tx = db.transaction_on_one_with_mode(&store_name, IdbTransactionMode::Readwrite).expect("tx");
            let store = tx.object_store(&store_name).expect("store");

            store.add_key_val_owned("foo", &JsValue::from("bar")).expect("put 1");
            store.add_key_val_owned("foo", &JsValue::from("qux")).expect("put 2");
            match tx.await {
                IdbTransactionResult::Abort => panic!("Aborted"),
                IdbTransactionResult::Success => panic!("Didn't error"),
                IdbTransactionResult::Error(_) => {
                    // Pass; don't check error message as it differs across browsers
                }
            };
        });
    }
}
