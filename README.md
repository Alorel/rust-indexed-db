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

#[derive(serde::Serialize, serde::Deserialize)]
struct MySerdeType(u8, String);

async fn main() -> indexed_db_futures::OpenDbResult<()> {
    let db = Database::open("my_db")
        .with_version(2u8)
        .with_on_upgrade_needed(|event, db| {
            match (event.old_version(), event.new_version()) {
                (0.0, Some(1.0)) => {
                    db.create_object_store("my_store")
                        .with_auto_increment(true)
                        .build()?;
                }
                (prev, Some(2.0)) => {
                    if prev == 1.0 {
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
        .await?;

    // awaiting individual requests is optional - they still go out
    store.put(MySerdeType(10, "foos".into())).serde()?;

    // Unlike JS, transactions ROLL BACK INSTEAD OF COMMITTING BY DEFAULT
    transaction.commit().await?;

    // Read some data
    let transaction = db.transaction("my_other_store").build()?;
    let store = transaction.object_store("my_other_store")?;
    let Some(mut cursor) = store.open_cursor().await? else {
        // `None` is returned if the cursor is empty
        return Ok(());
    };

    loop {
        match cursor.next_record_ser::<MySerdeType>().await {
            Ok(Some(record)) => handle_record(record),
            Ok(None) => break,
            Err(e) => handle_error(e),
        }
    }

    Ok(())
}
```

Head over to the docs for a proper introduction!
