use crate::prelude::*;
use idb_fut::database::Database;

#[wasm_bindgen_test]
pub async fn invalid_state_error() {
    let db = random_db_with_store().await;
    let res = db.delete_object_store(&db.name()).unwrap_err();

    assert_dom_exc!(res, InvalidStateError);
}

#[wasm_bindgen_test]
pub async fn not_found_error() {
    let err = Database::open(random_str())
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            db.delete_object_store(&db.name())?;
            Ok(())
        })
        .await
        .unwrap_err();

    assert_dom_exc!(open err, NotFoundError);
}

#[wasm_bindgen_test]
pub async fn happy_path() {
    let n1 = random_str();
    let n1_clone = n1.clone();

    let db = Database::open(&n1)
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            db.create_object_store(&n1_clone).build()?;
            db.create_object_store(&random_str())
                .build()?
                .delete_object_store()
        })
        .await
        .unwrap();

    let names = db.object_store_names().collect::<Vec<_>>();
    assert_eq!(names, vec![n1]);
}
