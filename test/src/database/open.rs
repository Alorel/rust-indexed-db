use crate::prelude::*;
use idb_fut::database::Database;
use idb_fut::error::{JSError, OpenDbError};

#[wasm_bindgen_test]
pub async fn happy_path() {
    let name = random_str();
    let version = rand::random::<u16>();
    let db = Database::open(&name)
        .with_version(version)
        .await
        .expect("Open");

    assert_eq!(db.name(), name);
    assert_eq!(db.version(), version as f64);
}

#[wasm_bindgen_test]
pub async fn forward_error_upgrade_needed() {
    let err_msg = random_str();

    let err = Database::open(random_str())
        .with_on_upgrade_needed({
            let err_msg = err_msg.clone();
            move |_, _| Err(JSError::Error(js_sys::Error::new(&err_msg)).into())
        })
        .await
        .unwrap_err();

    let js_err = match err {
        OpenDbError::Base(Error::Unknown(JSError::Error(e))) => e,
        other => panic!("Wrong error: {other}"),
    };

    assert_eq!(ToString::to_string(&js_err.message()), err_msg);
}
