# Indexed DB Futures

<!-- cargo-rdme start -->

Wraps the [web_sys](https://crates.io/crates/web_sys) Indexed DB API in a Future-based API and
removes the pain of dealing with Javascript callbacks in Rust.

[![master CI badge](https://github.com/Alorel/rust-indexed-db/actions/workflows/core.yml/badge.svg)](https://github.com/Alorel/rust-indexed-db/actions/workflows/core.yml)
[![crates.io badge](https://img.shields.io/crates/v/indexed_db_futures)](https://crates.io/crates/indexed_db_futures)
[![docs.rs badge](https://img.shields.io/docsrs/indexed_db_futures?label=docs.rs)](https://docs.rs/indexed_db_futures)
[![dependencies badge](https://img.shields.io/librariesio/release/cargo/indexed_db_futures)](https://libraries.io/cargo/indexed_db_futures)

### Overall API design

In most cases API methods will return a `Result` containing a wrapped
`IdbRequest` that implements `IntoFuture`, such as
`VoidRequest`, or, when more appropriate, the `Future`
directly, e.g. `CountFuture`.

The key difference between a wrapped Request and Future is that Requests don't have _any_ event
listeners attached, which aims to make quickfire operations such as inserting several records
into an `IdbObjectStore` a little bit more efficient.

### Features

The library can ship without cursor or index support for apps that just need a simple key-value
store akin to `localStorage`.

- `cursors` - Enable cursor support
- `indices` - Enable index support
- `nightly` - Use unsafe nightly features where appropriate, such as [`unwrap_unchecked`](Option::unwrap_unchecked).
- `default`:
   - `cursors`
   - `indices`

### Examples

#### Connecting to a DB and doing basic CRUD

Variable types included for clarity.

```rust
use indexed_db_futures::prelude::*;

pub async fn example() -> Result<(), DomException> {
    // Open my_db v1
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32("my_db", 1)?;
    db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
        // Check if the object store exists; create it if it doesn't
        if let None = evt.db().object_store_names().find(|n| n == "my_store") {
            evt.db().create_object_store("my_store")?;
        }
        Ok(())
    }));

    let db: IdbDatabase = db_req.await?;

    // Insert/overwrite a record
    let tx: IdbTransaction = db
      .transaction_on_one_with_mode("my_store", IdbTransactionMode::Readwrite)?;
    let store: IdbObjectStore = tx.object_store("my_store")?;

    let value_to_put: JsValue = get_some_js_value();
    store.put_key_val_owned("my_key", &value_to_put)?;

    // IDBTransactions can have an Error or an Abort event; into_result() turns both into a
    // DOMException
    tx.await.into_result()?;

    // Delete a record
    let tx = db.transaction_on_one_with_mode("my_store", IdbTransactionMode::Readwrite)?;
    let store = tx.object_store("my_store")?;
    store.delete_owned("my_key")?;
    tx.await.into_result()?;

    // Get a record
    let tx = db.transaction_on_one("my_store")?;
    let store = tx.object_store("my_store")?;

    let value: Option<JsValue> = store.get_owned("my_key")?.await?;
    use_value(value);

    Ok(())
}
```

<!-- cargo-rdme end -->
