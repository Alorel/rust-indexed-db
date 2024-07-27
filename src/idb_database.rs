//! Database-related code

use fancy_constructor::new;
pub(crate) use idb_version_change_event::IdbVersionChangeCallback;
pub use idb_version_change_event::IdbVersionChangeEvent;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{DomException, IdbTransactionMode};

use crate::dom_string_iterator::DomStringIterator;
use crate::idb_object_store::{IdbObjectStore, IdbObjectStoreParameters};
use crate::idb_transaction::IdbTransaction;
use crate::internal_utils::arrayify_slice;
use crate::request::{OpenDbRequest, VoidOpenDbRequest};

mod idb_version_change_event;

/// Wrapper for an [`IndexedDB`](web_sys::IdbDatabase)
#[derive(Debug, new)]
#[new(vis(pub(crate)))]
pub struct IdbDatabase {
    inner: web_sys::IdbDatabase,

    #[new(default)]
    on_version_change: Option<IdbVersionChangeCallback>,
}

type OpenDbResult = Result<OpenDbRequest, DomException>;

impl_display_for_named!(IdbDatabase);

#[cfg(test)]
pub mod test {
    use core::future::Future;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::{IdbKeyPath, IdbQuerySource};
    use crate::request::IdbOpenDbRequestLike;

    use super::*;

    fn db_name() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    async fn open_db(req: OpenDbRequest) -> IdbDatabase {
        req.await.expect("Future failed")
    }

    fn open_db_req(req: Result<OpenDbRequest, DomException>) -> impl Future<Output = IdbDatabase> {
        open_db(req.expect("Base open failed"))
    }

    pub mod object_store_names {
        test_mod_init!();

        test_case!(async empty_iter => {
            let db = open_db_req(IdbDatabase::open(&db_name())).await;
            let stores: Vec<String> = db.object_store_names().collect();
            assert_eq!(stores, Vec::<String>::new());
        });

        test_case!(async iter_with_two => {
            fn on_upgrade_needed(evt: &IdbVersionChangeEvent) -> Result<(), JsValue> {
                evt.db().create_object_store("store1")?;
                evt.db().create_object_store("store2")?;
                let _ = evt.transaction(); // make sure it doesn't panic
                Ok(())
            }

            let mut req = IdbDatabase::open(&db_name()).expect("Base open");
            req.set_on_upgrade_needed(Some(on_upgrade_needed));
            let db = open_db(req).await;
            let stores: Vec<String> = db.object_store_names().collect();

            assert_eq!(stores, vec![String::from("store1"), String::from("store2")]);
        });
    }

    pub mod deletions {
        test_mod_init!();

        test_case!(async delete_object_store => {
            let db_name = db_name();

            let mut req = IdbDatabase::open_u32(&db_name, 1).expect("open 1");
            req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
                evt.db().create_object_store("s1")?;
                evt.db().create_object_store("s2")?;
                Ok(())
            }));
            let db = req.await.expect("db await 1");
            db.close();

            let mut req = IdbDatabase::open_u32(&db_name, 2).expect("open 2");
            req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
                evt.db().delete_object_store("s1")?;
                Ok(())
            }));
            let db = req.await.expect("db await 2");
            let stores: Vec<String> = db.object_store_names().collect();
            let exp = vec![String::from("s2"); 1];

            assert_eq!(stores, exp);
        });

        test_case!(async delete_by_name => {
            async fn do_open(name: &str, v: u32, calls: Rc<RefCell<u8>>) -> IdbDatabase {
                let mut req = IdbDatabase::open_u32(name, v).expect("open");
                req.set_on_upgrade_needed(Some(move |_: &IdbVersionChangeEvent| {
                    let curr = *calls.borrow();
                    calls.replace(curr + 1);
                    Ok(())
                }));
                req.await.expect("db await")
            }

            let db_name = db_name();
            let calls = Rc::new(RefCell::new(0));

            let db = do_open(&db_name, 1, calls.clone()).await;
            db.delete().expect("Delete call").await.expect("delete promise");
            do_open(&db_name, 1, calls.clone()).await;

            assert_eq!(*calls.borrow(), 2);
        });
    }

    pub mod tx_open {
        test_mod_init!();

        #[allow(clippy::needless_pass_by_value)]
        fn check_transaction(
            res: Result<IdbTransaction, DomException>,
            mode: IdbTransactionMode,
            exp: Vec<String>,
        ) {
            let tx = res.expect("tx open failed");
            let mut stores: Vec<String> = tx.object_store_names().collect();
            stores.sort();

            assert_eq!(tx.mode(), mode, "Mode");
            assert_eq!(stores, exp, "Stores");
        }

        async fn open_db() -> IdbDatabase {
            let mut req = IdbDatabase::open(&db_name()).expect("open");
            req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
                evt.db().create_object_store("s1")?;
                evt.db().create_object_store("s2")?;
                Ok(())
            }));
            req.await.expect("db await 1")
        }

        test_case!(async transaction_on_one => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_one("s1"),
                IdbTransactionMode::Readonly,
                vec![String::from("s1")]
            );
        });

        test_case!(async transaction_on_multi_with_one => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_multi(&["s1"]),
                IdbTransactionMode::Readonly,
                vec![String::from("s1")]
            );
        });

        test_case!(async transaction_on_multi_with_multi => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_multi(&["s1", "s2"]),
                IdbTransactionMode::Readonly,
                vec![String::from("s1"), String::from("s2")]
            );
        });

        test_case!(async transaction_on_one_with_mode_r => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_one_with_mode("s2", IdbTransactionMode::Readonly),
                IdbTransactionMode::Readonly,
                vec![String::from("s2")]
            );
        });

        test_case!(async transaction_on_one_with_mode_rw => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_one_with_mode("s2", IdbTransactionMode::Readwrite),
                IdbTransactionMode::Readwrite,
                vec![String::from("s2")]
            );
        });
    }

    test_case!(async create_object_store_with_params => {
        let mut req = IdbDatabase::open(&db_name()).expect("req");
        req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
            evt.db().create_object_store_with_params(
                "s1",
                IdbObjectStoreParameters::new()
                .auto_increment(true)
                .key_path(Some(&IdbKeyPath::str("foo")))
            )?;
            Ok(())
        }));
        let db = req.await.expect("db");
        let tx = db.transaction_on_one("s1").expect("tx");
        let store = tx.object_store("s1").expect("store");

        assert_eq!(store.key_path(), Some(IdbKeyPath::str("foo")), "key path");
        assert!(store.auto_increment(), "auto_icrement");
    });
}
