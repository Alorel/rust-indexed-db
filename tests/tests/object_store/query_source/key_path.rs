use crate::prelude::*;
use idb_fut::database::Database;
use idb_fut::KeyPath;

#[wasm_bindgen_test]
pub async fn auto_incremented() {
    let db = Database::open(random_str())
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            db.create_object_store(&db.name())
                .with_auto_increment(true)
                .build()?;
            Ok(())
        })
        .await
        .unwrap();

    open_tx!(db, Readonly > (tx, store));
    assert_eq!(store.key_path(), None);
}

#[wasm_bindgen_test]
pub async fn none() {
    let db = Database::open(random_str())
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            db.create_object_store(&db.name()).build()?;
            Ok(())
        })
        .await
        .unwrap();

    open_tx!(db, Readonly > (tx, store));
    assert_eq!(store.key_path(), None);
}

#[wasm_bindgen_test]
pub async fn explicit() {
    let db = Database::open(random_str())
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            db.create_object_store(&db.name())
                .with_key_path(Key::KEY_PATH)
                .build()?;
            Ok(())
        })
        .await
        .unwrap();

    open_tx!(db, Readonly > (tx, store));
    assert_eq!(
        store.key_path(),
        Some(KeyPath::<String>::One(Key::PATH.into()))
    );
}
