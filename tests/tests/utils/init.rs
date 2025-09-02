use crate::prelude::*;
use idb_fut::database::{Database, VersionChangeEvent};
use indexed_db_futures::transaction::Transaction;

pub async fn random_db_with_init<F>(on_upgrade_needed: F) -> Database
where
    F: Fn(VersionChangeEvent, &Transaction<'_>) -> idb_fut::Result<()> + 'static,
{
    Database::open(random_str())
        .with_on_upgrade_needed(on_upgrade_needed)
        .await
        .expect("random_db()")
}

/// Crate a DB with and an object store with a matching name and default params.
pub async fn random_db_with_store() -> Database {
    random_db_with_init(move |_, tx| {
        let db = tx.db();
        db.create_object_store(&db.name()).build()?;
        Ok(())
    })
    .await
}

/// Create a random DB and a store with a matching name that expect [`KeyVal`] inputs.
pub async fn random_db_keyval() -> Database {
    random_db_with_init(move |_, tx| {
        let db = tx.db();
        db.create_object_store(&db.name())
            .with_auto_increment(false)
            .with_key_path(Key::KEY_PATH)
            .build()?;
        Ok(())
    })
    .await
}

/// [`random_db_keyval`] + an index with default params.
#[cfg(feature = "indices")]
pub async fn random_db_idx_keyval() -> Database {
    random_db_with_init(move |_, tx| {
        let db = tx.db();
        let name = db.name();
        let store = db
            .create_object_store(&name)
            .with_auto_increment(false)
            .with_key_path(Key::KEY_PATH)
            .build()?;
        store.create_index(&name, Value::KEY_PATH).build()?;

        Ok(())
    })
    .await
}
