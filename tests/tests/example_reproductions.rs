//! Rustdoc can't run example code so any and all examples found in `lib.rs` or `README.md` should be
//! recreated here

use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
pub async fn multi_threaded_executor() {
    use indexed_db_futures::database::Database;
    use indexed_db_futures::prelude::*;
    use indexed_db_futures::transaction::TransactionMode;

    async fn exec_example(db: Database) -> indexed_db_futures::Result<()> {
        let transaction = db
            .transaction("my_store")
            .with_mode(TransactionMode::Readwrite)
            .build()?;
        let object_store = transaction.object_store("my_store")?;

        let req1 = object_store.add("foo").primitive()?;
        let req2 = object_store.add("bar").primitive()?;

        transaction.commit().await?;

        req1.await?;
        req2.await?;

        Ok(())
    }

    let db = Database::open("my_db_multi_threaded_executor")
        .with_on_upgrade_needed(|_, tx| {
            let db = tx.db();
            db.create_object_store("my_store")
                .with_auto_increment(true)
                .build()?;
            Ok(())
        })
        .await
        .expect("DB open failed");

    exec_example(db).await.expect("Error running example")
}

#[wasm_bindgen_test]
#[cfg(all(feature = "tx-done", feature = "async-upgrade"))]
pub async fn opening_a_database_and_making_some_schema_changes() {
    use indexed_db_futures::database::{Database, VersionChangeEvent};
    use indexed_db_futures::prelude::*;
    use indexed_db_futures::transaction::TransactionMode;

    let _ = Database::open("opening_a_database_and_making_some_schema_changes")
        .with_version(2u8)
        .with_on_blocked(|_| Ok(()))
        .with_on_upgrade_needed_fut(|event: VersionChangeEvent, db: Database| {
            // Convert versions from floats to integers to allow using them in match expressions
            let old_version = event.old_version() as u64;
            let new_version = event.new_version().map(|v| v as u64);

            async move {
                match (old_version, new_version) {
                    (0, Some(1)) => {
                        db.create_object_store("my_store")
                            .with_auto_increment(true)
                            .build()?;
                    }
                    (prev, Some(2)) => {
                        if prev == 1 {
                            db.delete_object_store("my_store")?;
                        }

                        // Create an object store and await its transaction before inserting data.
                        db.create_object_store("my_other_store")
                            .with_auto_increment(true)
                            .build()?
                            .transaction()
                            .on_done()?
                            .await
                            .into_result()?;

                        //- Start a new transaction & add some data
                        let tx = db
                            .transaction("my_other_store")
                            .with_mode(TransactionMode::Readwrite)
                            .build()?;
                        let store = tx.object_store("my_other_store")?;
                        store.add("foo").await?;
                        store.add("bar").await?;
                        tx.commit().await?;
                    }
                    _ => {}
                }

                Ok(())
            }
        })
        .await
        .expect("Error opening DB");
}

#[wasm_bindgen_test]
#[cfg(feature = "serde")]
pub async fn rw_serde() {
    use indexed_db_futures::database::Database;
    use indexed_db_futures::prelude::*;
    use indexed_db_futures::transaction::TransactionMode;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct UserRef {
        id: u32,
        name: String,
    }

    let db = Database::open("example_rw_serde")
        .with_on_upgrade_needed(|_, tx| {
            let db = tx.db();
            db.create_object_store("users")
                .with_key_path("id".into())
                .build()?;

            Ok(())
        })
        .await
        .expect("DB open error");
    let tx = db
        .transaction("users")
        .with_mode(TransactionMode::Readwrite)
        .build()
        .expect("tx build error");
    let object_store = tx.object_store("users").expect("store open error");

    object_store
        .put(UserRef {
            id: 1,
            name: "Bobby Tables".into(),
        })
        .serde()
        .expect("serialisation error")
        .await
        .expect("put error");
    let user: UserRef = object_store
        .get(1u32)
        .serde()
        .expect("deserialisation error")
        .await
        .expect("get error")
        .expect("user not found");

    assert_eq!(user.id, 1);
    assert_eq!(user.name.as_str(), "Bobby Tables");
}

#[wasm_bindgen_test]
#[cfg(all(feature = "serde", feature = "cursors"))]
pub async fn readme_example() {
    use indexed_db_futures::database::Database;
    use indexed_db_futures::prelude::*;
    use indexed_db_futures::transaction::TransactionMode;
    use serde::{Deserialize, Serialize};

    async fn main() -> indexed_db_futures::OpenDbResult<()> {
        let db = Database::open("my_db_readme_example")
            .with_version(2u8)
            .with_on_upgrade_needed(|event, tx| {
                let db = tx.db();
                // Convert versions from floats to integers to allow using them in match expressions
                let old_version = event.old_version() as u64;
                let new_version = event.new_version().map(|v| v as u64);

                match (old_version, new_version) {
                    (0, Some(1)) => {
                        db.create_object_store("my_store")
                            .with_auto_increment(true)
                            .build()?;
                    }
                    (prev, Some(2)) => {
                        if prev == 1 {
                            let _ = db.delete_object_store("my_store");
                        }

                        db.create_object_store("my_other_store").build()?;
                    }
                    _ => {}
                }

                Ok(())
            })
            .await?;

        // Populate some data
        let transaction = db
            .transaction("my_other_store")
            .with_mode(TransactionMode::Readwrite)
            .build()?;

        let store = transaction.object_store("my_other_store")?;

        store
            .put("a primitive value that doesn't need serde")
            .with_key("my_key")
            .await?;

        // Awaiting individual requests is optional - they still go out
        store
            .put(MySerdeType(10, "foos".into()))
            .with_key("my_serde_key")
            .with_key_type::<String>() // `serde` keys must be deserialisable; String is, but the &str above isn't
            .serde()?;

        // Unlike JS, transactions ROLL BACK INSTEAD OF COMMITTING BY DEFAULT
        transaction.commit().await?;

        // Read some data
        let transaction = db.transaction("my_other_store").build()?;
        let store = transaction.object_store("my_other_store")?;

        // `None` is returned if the cursor is empty
        if let Some(mut cursor) = store.open_cursor().await? {
            // Use a limited loop in case we made a mistake and result in an infinite loop
            for _ in 0..5 {
                // We inserted a serde record and a primitive one so we need to deserialise as an enum that supports both
                match cursor.next_record_ser::<ObjectOrString>().await {
                    Ok(Some(record)) => match record {
                        ObjectOrString::Object(serde_record) => {
                            assert_eq!(serde_record.0, 10);
                            assert_eq!(serde_record.1, "foos");
                        }
                        ObjectOrString::String(string_record) => {
                            assert_eq!(
                                string_record.as_str(),
                                "a primitive value that doesn't need serde"
                            );
                        }
                    },
                    Err(e) => return Err(e.into()),
                    Ok(None) => return Ok(()), // reached cursor end
                }
            }

            panic!("Got an infinite loop!");
        }

        Ok(())
    }

    #[derive(Serialize, Deserialize)]
    struct MySerdeType(u8, String);

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ObjectOrString {
        Object(MySerdeType),
        String(String),
    }

    main().await.expect("Example errored");
}

#[wasm_bindgen_test]
#[cfg(feature = "cursors")]
pub async fn iterating_a_cursor() {
    use indexed_db_futures::database::Database;
    use indexed_db_futures::prelude::*;
    use indexed_db_futures::transaction::TransactionMode;

    let db = Database::open("example_iterating_a_cursor")
        .with_version(2u8)
        .with_on_upgrade_needed(|_, tx| {
            let db = tx.db();
            db.create_object_store("my_store").build()?;
            Ok(())
        })
        .await
        .expect("db create error");

    // Insert some data
    let tx = db
        .transaction("my_store")
        .with_mode(TransactionMode::Readwrite)
        .build()
        .expect("tx build error");
    let store = tx.object_store("my_store").expect("store open error");

    for i in 5u8..=8 {
        store
            .put(format!("num:{}", i))
            .with_key(i)
            .build()
            .expect("put error");
    }

    tx.commit().await.expect("put tx commit error");

    // Read the data
    let tx = db.transaction("my_store").build().expect("tx build error");
    let store = tx.object_store("my_store").expect("store open error");

    if let Some(mut cursor) = store.open_cursor().await.expect("error opening cursor") {
        while let Some(record) = cursor
            .next_record::<String>()
            .await
            .expect("Error fetching next record")
        {
            let num_part = record.strip_prefix("num:").expect("record un-pareseable");
            match num_part {
                "5" | "6" | "7" | "8" => {}
                _ => panic!("Unexpected record: {}", record),
            }
        }
    } else {
        panic!("Cursor empty");
    }
}

#[wasm_bindgen_test]
#[cfg(all(
    feature = "cursors",
    feature = "serde",
    feature = "streams",
    feature = "indices"
))]
pub async fn iterating_index_as_a_stream() {
    use futures::TryStreamExt;
    use indexed_db_futures::database::Database;
    use indexed_db_futures::prelude::*;
    use indexed_db_futures::transaction::TransactionMode;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Debug)]
    struct UserRef {
        id: u8,
        name: String,
    }

    let db = Database::open("example_iterating_index_as_a_stream")
        .with_on_upgrade_needed(|_, tx| {
            let db = tx.db();
            let store = db
                .create_object_store("my_store")
                .with_key_path("id".into())
                .build()?;
            store.create_index("my_index", "id".into()).build()?;
            Ok(())
        })
        .await
        .expect("db open err");

    // Insert some data
    let tx = db
        .transaction("my_store")
        .with_mode(TransactionMode::Readwrite)
        .build()
        .expect("tx write build error");
    let store = tx.object_store("my_store").expect("write store open error");

    for i in 0..=10 {
        store
            .put(UserRef {
                id: i,
                name: format!("user:{}", i),
            })
            .serde()
            .expect("serialisation error");
    }

    tx.commit().await.expect("put tx commit error");

    let tx = db
        .transaction("my_store")
        .build()
        .expect("tx read build error");
    let store = tx.object_store("my_store").expect("write store open error");
    let index = store.index("my_index").expect("index open error");

    let Some(cursor) = index
        .open_cursor()
        .with_query(5u8..=8)
        .serde()
        .expect("cursor open deserialisation error")
        .await
        .expect("cursor open error")
    else {
        panic!("Cursor empty");
    };

    let stream = cursor.stream_ser::<UserRef>();
    let mut records = stream
        .try_collect::<Vec<_>>()
        .await
        .expect("collect() error");
    records.sort_unstable();

    assert_eq!(
        records.as_slice(),
        &[
            UserRef {
                id: 5,
                name: "user:5".into()
            },
            UserRef {
                id: 6,
                name: "user:6".into()
            },
            UserRef {
                id: 7,
                name: "user:7".into()
            },
            UserRef {
                id: 8,
                name: "user:8".into()
            },
        ]
    );
}
