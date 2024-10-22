pub mod key_path;

#[cfg(feature = "cursors")]
pub mod cursor;

use crate::prelude::*;
use idb_fut::database::Database;
use idb_fut::KeyRange;

#[wasm_bindgen_test]
pub async fn count() {
    async fn count_all(db: &Database) -> idb_fut::Result<u32> {
        open_tx!(db, Readonly > (tx, store));
        dyn_await!(store.count())
    }

    let db = random_db_keyval().await;
    assert_eq!(count_all(&db).await, Ok(0), "Initial");

    KeyVal::insert_keyval_docs(&db).await;
    assert_eq!(count_all(&db).await, Ok(KeyVal::RANGE_LEN), "all");

    let filtered = {
        open_tx!(db, Readonly > (tx, store));
        dyn_await!(store.count().with_query(Key::new(-5)..Key::new(-2)))
    };
    assert_eq!(filtered, Ok(3), "filtered");
}

#[wasm_bindgen_test]
pub async fn get() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_tx!(db, Readonly > (tx, store));

    let none = dyn_await!(store.get::<KeyVal, _, _>(random_str()));
    assert_eq!(none, Ok(None));

    let one = dyn_await!(store.get::<KeyVal, Key, _>(KeyRange::Only(Key::MAX)));
    assert_eq!(one, Ok(Some(KeyVal::new(Key::MAX, Value::MIN))));

    let ranged = dyn_await!(store.get::<KeyVal, _, _>(Key::new(-3)..=Key::new(3)));
    assert_eq!(ranged, Ok(Some(KeyVal::new(Key::new(-3), Value::new(3)))));
}

#[wasm_bindgen_test]
pub async fn get_key() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_tx!(db, Readonly > (tx, store));

    let none = dyn_await!(store.get_key(random_str()));
    assert_eq!(none, Ok(None));

    let one = dyn_await!(store.get_key(KeyRange::Only(Key::MAX)));
    assert_eq!(one, Ok(Some(Key::MAX)));

    let retype = {
        let req = store
            .get_key::<Key, _>(KeyRange::Only(Key::MAX))
            .with_key_type::<i8>();
        dyn_await!(req)
    };
    assert_eq!(retype, Ok(Some(*Key::MAX)));

    let range = dyn_await!(store.get_key(Key::new(-3)..=Key::new(3)));
    assert_eq!(range, Ok(Some(Key::new(-3))));
}

#[wasm_bindgen_test]
pub async fn get_all() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    let all = {
        open_tx!(db, Readonly > (tx, store));
        let req = store.get_all::<KeyVal>();
        collect!(req)
    };
    assert_eq!(all, KeyVal::iter_range().collect::<Vec<_>>());

    let limited = {
        open_tx!(db, Readonly > (tx, store));
        let req = store.get_all::<KeyVal>().with_limit(3);
        collect!(req)
    };
    assert_eq!(limited, KeyVal::iter_range().take(3).collect::<Vec<_>>());

    let filtered = {
        open_tx!(db, Readonly > (tx, store));
        collect!(store
            .get_all::<KeyVal>()
            .with_query(Key::new(-7)..Key::new(-2)))
    };
    assert_eq!(
        filtered,
        KeyVal::iter_range().skip(3).take(5).collect::<Vec<_>>()
    );

    let filtered_limited = {
        open_tx!(db, Readonly > (tx, store));
        collect!(store
            .get_all::<KeyVal>()
            .with_query(Key::new(-7)..)
            .with_limit(2))
    };
    assert_eq!(
        filtered_limited,
        KeyVal::iter_range().skip(3).take(2).collect::<Vec<_>>()
    );

    let none = {
        open_tx!(db, Readonly > (tx, store));
        collect!(store.get_all::<KeyVal>().with_query(random_str()))
    };
    assert_eq!(none, Vec::new());
}

#[wasm_bindgen_test]
pub async fn get_all_keys() {
    let db = random_db_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    let all = {
        open_tx!(db, Readonly > (tx, store));
        let req = store.get_all_keys::<Key>();
        collect!(req)
    };
    assert_eq!(all, Key::iter_range().collect::<Vec<_>>());

    let limited = {
        open_tx!(db, Readonly > (tx, store));
        let req = store.get_all_keys::<Key>().with_limit(3);
        collect!(req)
    };
    assert_eq!(limited, Key::iter_range().take(3).collect::<Vec<_>>());

    let filtered = {
        open_tx!(db, Readonly > (tx, store));
        collect!(store
            .get_all_keys::<Key>()
            .with_query(Key::new(-7)..Key::new(-2)))
    };
    assert_eq!(
        filtered,
        Key::iter_range().skip(3).take(5).collect::<Vec<_>>()
    );

    let filtered_limited = {
        open_tx!(db, Readonly > (tx, store));
        collect!(store
            .get_all_keys::<Key>()
            .with_query(Key::new(-7)..)
            .with_limit(2))
    };
    assert_eq!(
        filtered_limited,
        Key::iter_range().skip(3).take(2).collect::<Vec<_>>()
    );

    let none = {
        open_tx!(db, Readonly > (tx, store));
        collect!(store.get_all_keys::<Key>().with_query(random_str()))
    };
    assert_eq!(none, Vec::<Key>::new());
}
