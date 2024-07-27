use super::random_db_versions;
use crate::prelude::*;
use idb_fut::database::Database;
use idb_fut::error::{Error, JSError, OpenDbError};
use tokio::sync::oneshot;
use wasm_bindgen_futures::spawn_local;

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
pub async fn forward_error_on_upgrade_needed() {
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

#[wasm_bindgen_test]
pub async fn on_blocked() {
    let [v1, v2] = random_db_versions();
    let db_name = random_str();
    let db1 = Database::open(&db_name).with_version(v1).await.unwrap();
    let (tx, rx) = oneshot::channel();
    spawn_local(async move {
        let _ = Database::open(&db_name)
            .with_version(v2)
            .with_on_blocked(move |evt| {
                let _ = tx.send(evt);
                Ok(())
            })
            .await;
    });

    let evt = rx.await.unwrap();
    assert_eq!(evt.old_version(), v1 as f64);
    assert_eq!(evt.new_version(), Some(v2 as f64));

    db1.close();
}
