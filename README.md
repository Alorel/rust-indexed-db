Wraps the [web_sys](https://crates.io/crates/web_sys) Indexed DB API in a Future-based API and
removes the pain of dealing with JS callbacks or `JSValue` in Rust.

[![master CI badge](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml/badge.svg)](https://github.com/Alorel/rust-indexed-db/actions/workflows/test.yml)
[![crates.io badge](https://img.shields.io/crates/v/indexed_db_futures)](https://crates.io/crates/indexed_db_futures)
[![docs.rs badge](https://img.shields.io/docsrs/indexed_db_futures?label=docs.rs)](https://docs.rs/indexed_db_futures)
[![dependencies badge](https://img.shields.io/librariesio/release/cargo/indexed_db_futures)](https://libraries.io/cargo/indexed_db_futures)

Goals & features:

- **Shield you from having to interact with [`web_sys`](https://crates.io/crates/web-sys) or
  [`js_sys`](https://crates.io/crates/js-sys) APIs** - this should feel like a native Rust API.
- **Integrate with [`serde`](https://crates.io/crates/serde), but don't require it** - as a rule of thumb, you'll use
  `serde`-serialisable types when working with JS objects & bypass `serde` for Javascript primitives.
- **Implement [`Stream`](https://docs.rs/futures/0.3.31/futures/prelude/trait.Stream.html) where applicable** - cursors
  and key cursors have this at the time of writing.
- **Implement a more Rust-oriented API** - for example, transactions will roll back by default unless explicitly
  committed to allow you to use `?`s.

```rust
use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;

#[derive(Serialize, Deserialize)]
struct MySerdeType(u8, String);

#[derive(Deserialize)]
#[serde(untagged)]
enum ObjectOrString {
    Object(MySerdeType),
    String(String),
}

async fn main() -> indexed_db_futures::OpenDbResult<()> {
    let db = Database::open("my_db")
        .with_version(2u8)
        .with_on_upgrade_needed(|event, db| {
            // Convert versions from floats to integers to allow using them in match expressions
            let old_version = event.old_version() as u64;
            let new_version = event.new_version().map(|v| v as u64);

            match (event.old_version(), event.new_version()) {
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
```

Head over to the docs for a proper introduction!
