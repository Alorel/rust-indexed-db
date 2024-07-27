use crate::prelude::*;

#[wasm_bindgen_test]
pub async fn readonly_error() {
    let db = random_db_with_store().await;
    open_tx!(db, Readonly > (tx, store));
    assert_dom_exc!(store.clear().unwrap_err(), ReadOnlyError);
}

#[wasm_bindgen_test]
pub async fn happy_path() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_tx!(db, Readwrite > (tx, store));
    assert_ne!(dyn_await!(store.count()).unwrap(), 0);

    store.clear().unwrap().await.unwrap();
    assert_eq!(dyn_await!(store.count()), Ok(0));
}
