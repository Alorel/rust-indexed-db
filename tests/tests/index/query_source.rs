#[cfg(feature = "cursors")]
pub mod cursor;

use crate::prelude::*;
use idb_fut::database::Database;
use idb_fut::{KeyPath, KeyRange};

#[wasm_bindgen_test]
pub async fn count() {
    async fn count_all(db: &Database) -> idb_fut::Result<u32> {
        open_idx!(db, Readonly > idx);
        dyn_await!(idx.count())
    }

    let db = random_db_idx_keyval().await;
    assert_eq!(count_all(&db).await, Ok(0), "Initial");

    KeyVal::insert_keyval_docs(&db).await;
    assert_eq!(count_all(&db).await, Ok(KeyVal::RANGE_LEN), "all");

    let filtered = {
        open_idx!(db, Readonly > idx);
        dyn_await!(idx.count().with_query(Value::new(5)..=Value::new(8)))
    };
    assert_eq!(filtered, Ok(4), "filtered");
}

#[wasm_bindgen_test]
pub async fn get() {
    let db = random_db_idx_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_idx!(db, Readonly > idx);

    let none = dyn_await!(idx.get::<KeyVal, _, _>(random_str()));
    assert_eq!(none, Ok(None));

    let one = dyn_await!(idx.get::<KeyVal, Value, _>(KeyRange::Only(Value::MAX)));
    assert_eq!(one, Ok(Some(KeyVal::new(Key::MIN, Value::MAX))));

    let ranged = dyn_await!(idx.get::<KeyVal, _, _>(Value::new(3)..=Value::new(5)));
    assert_eq!(ranged, Ok(Some(KeyVal::new(Key::new(-3), Value::new(3)))));
}

#[wasm_bindgen_test]
pub async fn get_key() {
    let db = random_db_idx_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    open_idx!(db, Readonly > idx);

    let none = dyn_await!(idx.get_key(random_str()));
    assert_eq!(none, Ok(None));

    let one = {
        let req = idx
            .get_key::<Value, _>(KeyRange::Only(Value::MAX))
            .with_key_type::<Key>();
        dyn_await!(req)
    };
    assert_eq!(one, Ok(Some(Key::MIN)));

    let range = {
        let req = idx
            .get_key(Value::new(3)..=Value::new(6))
            .with_key_type::<Key>();
        dyn_await!(req)
    };
    assert_eq!(range, Ok(Some(Key::new(-3))));
}

#[wasm_bindgen_test]
pub async fn get_all() {
    let db = random_db_idx_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    let all = {
        open_idx!(db, Readonly > idx);
        let req = idx.get_all::<KeyVal>();
        collect!(req)
    };
    assert_eq!(all, KeyVal::iter_range().rev().collect::<Vec<_>>());

    let limited = {
        open_idx!(db, Readonly > idx);
        let req = idx.get_all::<KeyVal>().with_limit(3);
        collect!(req)
    };
    assert_eq!(
        limited,
        KeyVal::iter_range().rev().take(3).collect::<Vec<_>>()
    );

    let filtered = {
        open_idx!(db, Readonly > idx);
        collect!(idx
            .get_all::<KeyVal>()
            .with_query(Value::new(2)..Value::new(7)))
    };
    assert_eq!(
        filtered,
        KeyVal::iter_range()
            .rev()
            .skip(1)
            .take(5)
            .collect::<Vec<_>>()
    );

    let filtered_limited = {
        open_idx!(db, Readonly > idx);
        collect!(idx
            .get_all::<KeyVal>()
            .with_query(Value::new(4)..)
            .with_limit(2))
    };
    assert_eq!(
        filtered_limited,
        KeyVal::iter_range()
            .rev()
            .skip(3)
            .take(2)
            .collect::<Vec<_>>()
    );

    let none = {
        open_idx!(db, Readonly > idx);
        collect!(idx.get_all::<KeyVal>().with_query(random_str()))
    };
    assert_eq!(none, Vec::new());
}

#[wasm_bindgen_test]
pub async fn get_all_keys() {
    let db = random_db_idx_keyval().await;
    KeyVal::insert_keyval_docs(&db).await;

    let all = {
        open_idx!(db, Readonly > idx);
        let req = idx.get_all_keys::<Key>();
        collect!(req)
    };
    assert_eq!(all, Key::iter_range().rev().collect::<Vec<_>>());

    let limited = {
        open_idx!(db, Readonly > idx);
        let req = idx.get_all_keys::<Key>().with_limit(3);
        collect!(req)
    };
    assert_eq!(limited, Key::iter_range().rev().take(3).collect::<Vec<_>>());

    let filtered = {
        open_idx!(db, Readonly > idx);
        collect!(idx
            .get_all_keys::<Key>()
            .with_query(Value::new(2)..Value::new(5)))
    };
    assert_eq!(
        filtered,
        Key::iter_range().rev().skip(1).take(3).collect::<Vec<_>>()
    );

    let filtered_limited = {
        open_idx!(db, Readonly > idx);
        collect!(idx
            .get_all_keys::<Key>()
            .with_query(Value::new(3)..Value::new(7))
            .with_limit(2))
    };
    assert_eq!(
        filtered_limited,
        Key::iter_range().rev().skip(2).take(2).collect::<Vec<_>>()
    );

    let none = {
        open_idx!(db, Readonly > idx);
        collect!(idx.get_all_keys::<Key>().with_query(random_str()))
    };
    assert_eq!(none, Vec::<Key>::new());
}

#[wasm_bindgen_test]
pub async fn key_path() {
    let db = random_db_idx_keyval().await;
    open_idx!(db, Readonly > idx);

    assert_eq!(
        idx.key_path(),
        Some(KeyPath::<String>::One(Value::PATH.into()))
    );
}
