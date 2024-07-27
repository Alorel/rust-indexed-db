use crate::prelude::*;
use idb_fut::KeyRange;

#[wasm_bindgen_test]
pub async fn readonly_error() {
    let db = random_db_with_store().await;
    open_tx!(db, Readonly > (tx, store));
    let err = store.delete(0u8).build_dyn().unwrap_err();

    assert_dom_exc!(err, ReadOnlyError);
}

#[wasm_bindgen_test]
pub async fn data_error() {
    let db = random_db_with_store().await;
    open_tx!(db, Readwrite > (tx, store));

    let key = KeyRange::Only(js_sys::Symbol::for_(".").unchecked_into::<JsValue>());
    let err = store.delete::<JsValue, _>(key).build().unwrap_err();

    assert_dom_exc!(err, DataError);
}

#[wasm_bindgen_test]
pub async fn delete_one() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_tx!(db, Readwrite > (tx, store));

    dyn_await!(store.delete(-1i8)).unwrap();

    let mut actual = dyn_await!(store.get_all::<KeyVal>())
        .expect("get_all")
        .collect::<idb_fut::Result<Vec<_>>>()
        .expect("collect");
    actual.sort_unstable();

    let mut expected = KeyVal::iter_range()
        .filter(|kv| kv.key() != -1)
        .collect::<Vec<_>>();
    expected.sort_unstable();

    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
pub async fn delete_multi() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_tx!(db, Readwrite > (tx, store));
    store.delete(i8::MIN..*Key::MAX).await.unwrap();

    let records = dyn_await!(store.get_all::<KeyVal>())
        .expect("get_all")
        .collect::<idb_fut::Result<Vec<_>>>()
        .expect("collect");

    assert_eq!(records, vec![KeyVal::new(Key::MAX, Value::MIN)]);
}
