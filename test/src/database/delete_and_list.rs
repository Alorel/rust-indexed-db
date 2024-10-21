use crate::prelude::*;
use idb_fut::database::Database;
use idb_fut::factory::{DBFactory, DatabaseDetails};
use std::collections::BTreeSet;

async fn collect_dbs(factory: &DBFactory) -> BTreeSet<DatabaseDetails> {
    factory
        .databases()
        .expect("databases()")
        .await
        .expect("databases().await")
        .collect::<Result<BTreeSet<_>, _>>()
        .expect("try_collect")
}

#[wasm_bindgen_test]
pub async fn delete_and_list_dbs() {
    let factory = DBFactory::new().expect("factory");

    let initial = collect_dbs(&factory).await;

    let db = Database::open(random_str())
        .with_factory(factory.clone())
        .await
        .expect("create db");

    {
        let mut expect = initial.clone();
        expect.insert(DatabaseDetails::new(db.name(), db.version()));

        let actual = collect_dbs(&factory).await;
        assert_eq!(actual, expect);
    }

    db.delete()
        .expect("delete()")
        .await
        .expect("delete().await");

    assert_eq!(collect_dbs(&factory).await, initial);
}
