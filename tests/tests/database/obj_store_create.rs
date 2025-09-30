use crate::prelude::*;
use idb_fut::database::Database;

#[wasm_bindgen_test]
pub async fn happy_path() {
    let db = random_db_with_store().await;

    let stores = db.object_store_names().collect::<Vec<_>>();
    assert_eq!(stores, vec![db.name()]);
}

#[wasm_bindgen_test]
pub async fn constraint_error() {
    let err = Database::open(random_str())
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            let name = random_str();
            db.create_object_store(&name).build()?;
            db.create_object_store(&name).build()?;
            Ok(())
        })
        .await
        .unwrap_err();

    assert_dom_exc!(open err, ConstraintError);
}

#[wasm_bindgen_test]
pub async fn invalid_access_error() {
    let err = Database::open(random_str())
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            db.create_object_store(&db.name())
                .with_auto_increment(true)
                .with_key_path("".into())
                .build()?;
            Ok(())
        })
        .await
        .unwrap_err();

    assert_dom_exc!(open err, InvalidAccessError);
}

#[wasm_bindgen_test]
pub async fn invalid_state_error() {
    let name = random_str();
    let db = Database::open(&name).await.unwrap();
    let err = db.create_object_store(&name).build().unwrap_err();

    assert_dom_exc!(err, InvalidStateError);
}
