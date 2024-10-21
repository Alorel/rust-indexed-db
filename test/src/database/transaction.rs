use crate::prelude::*;
use idb_fut::database::Database;

#[wasm_bindgen_test]
pub async fn not_found_error() {
    let name = random_str();
    let db = Database::open(&name).await.unwrap();
    let err = db.transaction(name).build().unwrap_err();

    assert_dom_exc!(err, NotFoundError);
}

#[wasm_bindgen_test]
pub async fn invalid_access_error() {
    let db = Database::open(random_str()).await.unwrap();
    let names: [String; 0] = [];
    let err = db.transaction(names).build().unwrap_err();

    assert_dom_exc!(err, InvalidAccessError);
}
