use super::random_db_versions;
use crate::prelude::*;
use idb_fut::database::Database;

#[wasm_bindgen_test]
pub async fn basic() {
    let [v1, v2] = random_db_versions();
    let db_name = random_str();
    let db1 = Database::open(&db_name).with_version(v1).await.unwrap();
    let mut v_change = db1.version_changes().unwrap();
    helpers::open_v2(db_name, v2);

    let evt = v_change.recv().await.unwrap();
    assert_eq!(evt.old_version(), v1 as f64);
    assert_eq!(evt.new_version(), Some(v2 as f64));

    db1.close();
}

#[wasm_bindgen_test]
pub async fn stream() {
    use futures::StreamExt;

    let [v1, v2] = random_db_versions();
    let db_name = random_str();
    let db1 = Database::open(&db_name).with_version(v1).await.unwrap();
    let mut v_change = db1.version_changes().unwrap();
    helpers::open_v2(db_name, v2);

    let evt = v_change.next().await.unwrap();
    assert_eq!(evt.old_version(), v1 as f64);
    assert_eq!(evt.new_version(), Some(v2 as f64));

    db1.close();
}

mod helpers {
    use indexed_db_futures::database::Database;
    use wasm_bindgen_futures::spawn_local;

    pub fn open_v2(db_name: String, version: u8) {
        spawn_local(async move {
            let _ = Database::open(db_name).with_version(version).await;
        });
    }
}
