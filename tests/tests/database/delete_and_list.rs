use crate::prelude::*;
use idb_fut::database::Database;
use idb_fut::factory::{DBFactory, DatabaseDetails};
use rand::prelude::*;
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

    let expected_details =
        DatabaseDetails::new(random_str(), thread_rng().gen_range(1u8..=u8::MAX) as f64);

    let db = Database::open(expected_details.name())
        .with_factory(factory.clone())
        .with_version(expected_details.version())
        .await
        .expect("create db");

    {
        let dbs = collect_dbs(&factory).await;
        assert!(dbs.contains(&expected_details), "{:?}", dbs);
    }

    db.delete()
        .expect("delete()")
        .await
        .expect("delete().await");

    {
        let dbs = collect_dbs(&factory).await;
        assert!(
            !collect_dbs(&factory).await.contains(&expected_details),
            "{:?}",
            dbs
        );
    }
}
