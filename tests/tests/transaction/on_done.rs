use crate::prelude::*;
use indexed_db_futures::transaction::TransactionFinishKind;

#[wasm_bindgen_test]
pub async fn complete() {
    let db = random_db_with_store().await;
    open_tx!(db, Readonly > (tx, store));

    let on_done = tx.on_done().unwrap();
    drop(store);
    tx.commit().await.unwrap();

    assert_eq!(on_done.await, TransactionFinishKind::Ok);
}

#[wasm_bindgen_test]
pub async fn abort() {
    let db = random_db_with_store().await;
    open_tx!(db, Readonly > (tx, store));

    let on_done = tx.on_done().unwrap();
    drop(store);
    tx.abort().await.unwrap();

    assert_eq!(on_done.await, TransactionFinishKind::Abort);
}

#[cfg(feature = "async-upgrade")]
pub mod async_upgrade {
    use crate::prelude::*;
    use indexed_db_futures::database::Database;
    use indexed_db_futures::error::{Error, JSError, OpenDbError};
    use indexed_db_futures::transaction::TransactionMode;
    use std::sync::{Arc, Mutex};

    const STORE_NAME: &str = "foostore";

    pub async fn upgrade_needed_err_thrown() {
        const ERR_MSG: &str = "upgrade_needed_err_thrown";

        let err = Database::open(random_str())
            .with_version(2u8)
            .with_on_upgrade_needed_fut(move |_, db: Database| async move {
                // Create an object store and await its transaction
                db.create_object_store(STORE_NAME)
                    .with_auto_increment(true)
                    .build()?
                    .transaction()
                    .on_done()?
                    .await
                    .into_result()?;

                Err(js_sys::Error::new(ERR_MSG).into())
            })
            .await
            .unwrap_err();

        let expect = OpenDbError::Base(Error::Unknown(JSError::Error(js_sys::Error::new(ERR_MSG))));
        assert_eq!(err, expect);
    }

    #[wasm_bindgen_test]
    pub async fn upgrade_needed_ok() {
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        enum Event {
            CallbackStart,
            InitialTransaction,
            InnerTransaction,
        }

        let events = Arc::new(Mutex::new(Vec::with_capacity(3)));

        Database::open(random_str())
            .with_version(2u8)
            .with_on_upgrade_needed_fut({
                let events = Arc::clone(&events);
                move |_, db: Database| async move {
                    events.lock().unwrap().push(Event::CallbackStart);

                    // Create an object store and await its transaction
                    db.create_object_store(STORE_NAME)
                        .with_auto_increment(true)
                        .build()?
                        .transaction()
                        .on_done()?
                        .await
                        .into_result()?;

                    events.lock().unwrap().push(Event::InitialTransaction);

                    let tx = db
                        .transaction(STORE_NAME)
                        .with_mode(TransactionMode::Readwrite)
                        .build()?;
                    let store = tx.object_store(STORE_NAME)?;

                    dyn_await!(store.add("foo"))?;
                    dyn_await!(store.add("bar"))?;
                    tx.commit().await?;

                    events.lock().unwrap().push(Event::InnerTransaction);

                    Ok(())
                }
            })
            .await
            .unwrap();

        let events = events.lock().unwrap();

        assert_eq!(
            &*events,
            &[
                Event::CallbackStart,
                Event::InitialTransaction,
                Event::InnerTransaction
            ]
        );
    }
}
