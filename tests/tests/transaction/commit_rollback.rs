use crate::prelude::*;

#[wasm_bindgen_test]
pub async fn explicit_commit() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_tx!(db, Readonly > (tx, store));
    assert_eq!(dyn_await!(store.count()), Ok(KeyVal::RANGE_LEN));
}

#[wasm_bindgen_test]
pub async fn explicit_rollback() {
    let db = random_db_keyval().await;

    open_tx!(db, Readwrite > (tx, store));
    store.put(KeyVal::new(0, 0)).build_dyn().unwrap();
    drop(store);
    assert_eq!(tx.abort().await, Ok(()));

    open_tx!(db, Readonly > (tx, store));
    assert_eq!(dyn_await!(store.count()), Ok(0));
}

#[wasm_bindgen_test]
pub async fn implicit_rollback() {
    let db = random_db_keyval().await;

    {
        open_tx!(db, Readwrite > (tx, store));
        store.put(KeyVal::default()).build_dyn().unwrap();
    }

    open_tx!(db, Readonly > (tx, store));
    assert_eq!(dyn_await!(store.count()), Ok(0));
}
