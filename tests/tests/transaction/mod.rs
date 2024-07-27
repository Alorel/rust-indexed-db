use crate::prelude::*;

pub mod commit_rollback;

#[wasm_bindgen_test]
pub async fn multi_store() {
    let db = random_db_with_init(move |_, db| {
        db.create_object_store("s1").build()?;
        db.create_object_store("s2").build()?;
        Ok(())
    })
    .await;

    let tx = db.transaction(["s1", "s2"]).build().unwrap();
    tx.object_store("s1").expect("s1");
    tx.object_store("s2").expect("s2");
    tx.object_store("s3").expect_err("s3");
}
