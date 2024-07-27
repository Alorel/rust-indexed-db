use std::future;

use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::Database;

pub(crate) mod prelude {
    pub(crate) use {super::*, crate::error::*, wasm_bindgen_test::wasm_bindgen_test};
}

pub(crate) const MSG_DB_OPEN: &str = "Failed to open DB";

pub(crate) fn random_name() -> String {
    Uuid::new_v4().to_string()
}

pub(crate) async fn random_db() -> Database {
    Database::open(&random_name()).await.expect(MSG_DB_OPEN)
}

pub(crate) async fn random_store() -> (Database, String) {
    let name = random_name();
    let name_cloned = name.clone();

    let db = Database::open(&name)
        .with_on_upgrade_needed(move |evt| {
            evt.db().create_object_store(&name_cloned).unwrap();
            future::ready(Ok::<_, crate::error::Error>(()))
        })
        .await
        .expect(MSG_DB_OPEN);

    (db, name)
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console, js_name=log)]
    pub(crate) fn log_str(msg: &str);
}
