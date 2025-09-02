use crate::prelude::*;
use idb_fut::database::Database;

#[wasm_bindgen_test]
pub async fn constraint_error() {
    let err = Database::open(random_str())
        .with_on_upgrade_needed(move |_, tx| {
            let db = tx.db();
            let name = db.name();
            let store = db
                .create_object_store(&name)
                .with_auto_increment(false)
                .with_key_path(Key::KEY_PATH)
                .build()?;

            if let Err(e) = store.create_index(&name, Key::KEY_PATH).build() {
                let e = js_sys::Error::new(&format!("1st idx creation errored: {e}"));
                return Err(e.into());
            }

            // This one should error
            store.create_index(&name, Key::KEY_PATH).build()?;

            Ok(())
        })
        .await
        .unwrap_err();

    assert_dom_exc!(open err, ConstraintError);
}
