//! Cursor-related code
//!
//! ## Examples
//!
// use super::{*, prelude::*};
// use wasm_bindgen::prelude::*;
// use web_sys::DomException;
//
// #[allow(unused_variables)]
// async fn example() -> Result<(), DomException> {
//     let db = IdbDatabase::open("foo_db")?.into_future().await?;
//     let tx = db.transaction_on_one("foo_store")?;
//     let object_store = tx.object_store("foo_store")?;
//
//!     match object_store.open_cursor()?.await? {
//!         Some(cursor) => {
//!             let first_key: JsValue = cursor.key().unwrap();
//!             let first_value: JsValue = cursor.value();
//!
//!             // Iterate one by one
//!             while cursor.continue_cursor()?.await? {
//!                 let subsequent_key: JsValue = cursor.key().unwrap();
//!                 let subsequent_value: JsValue = cursor.value();
//!             }
//!
//!             // Or collect the remainder into a vector
//!             let cursor_contents: Vec<KeyVal> = cursor.into_vec(0).await?;
//!         },
//!         None => {
//!             // No elements matched
//!         }
//!     };
//
//         Ok(())
//     }

use accessory::Accessors;
use fancy_constructor::new;
use std::future::Future;
use std::rc::Rc;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DomException, IdbCursorDirection};

pub use idb_cursor_with_value::*;

use crate::idb_query_source::IdbQuerySource;
use crate::internal_utils::optional_jsvalue_undefined;
use crate::request::{
    IdbCursorAdvancementFuture, IdbRequestFuture, IdbRequestRef, JsCastRequestFuture, VoidRequest,
};

mod idb_cursor_with_value;

/// An interface for an [`IndexedDB` cursor](web_sys::IdbCursor)
#[derive(Debug, new, Accessors)]
#[new(vis(pub(crate)))]
pub struct IdbCursor<'a, T: IdbQuerySource> {
    inner: web_sys::IdbCursor,

    #[access(get(cp))]
    /// What spawned this cursor
    source: &'a T,
    req: Rc<IdbRequestRef>,
}

impl<'a, T: IdbQuerySource> IdbCursor<'a, T> {
    #[inline]
    pub(crate) fn inner_as_cursor_with_value(&self) -> &web_sys::IdbCursorWithValue {
        self.inner.unchecked_ref()
    }

    /// Get the cursor direction
    #[inline]
    #[must_use]
    pub fn direction(&self) -> IdbCursorDirection {
        self.inner.direction()
    }

    /// Get the key at the cursor's current position. Returns `None` if the cursor is outside its
    /// range.
    #[must_use]
    pub fn key(&self) -> Option<JsValue> {
        optional_jsvalue_undefined(self.inner.key().unwrap())
    }

    /// Get the cursor's current effective primary key. Returns `None` if the cursor is currently
    /// being iterated or has iterated outside its range.
    #[inline]
    #[must_use]
    pub fn primary_key(&self) -> Option<JsValue> {
        optional_jsvalue_undefined(self.inner.primary_key().unwrap())
    }

    /// Common code for continue methods
    fn continue_common(&self) -> IdbCursorAdvancementFuture {
        let fut = IdbRequestFuture::new_with_rc(self.req.clone(), true);
        IdbCursorAdvancementFuture::new(fut)
    }

    /// Advances the cursor to the next position along its direction
    pub fn continue_cursor(
        &self,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner.continue_()?;
        Ok(self.continue_common())
    }

    /// Advances the cursor to the next position along its direction, to the item whose key matches
    /// the given key parameter
    pub fn continue_cursor_with_key<K: JsCast>(
        &self,
        key: &K,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner.continue_with_key(key.unchecked_ref())?;
        Ok(self.continue_common())
    }

    /// Sets the cursor to the given index key and primary key given as arguments.
    pub fn continue_primary_key<K: JsCast, PK: JsCast>(
        &self,
        key: &K,
        primary_key: &PK,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner
            .continue_primary_key(key.unchecked_ref(), primary_key.unchecked_ref())?;
        Ok(self.continue_common())
    }

    /// Sets the number of times a cursor should move its position forward.
    pub fn advance(
        &self,
        count: u32,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner.advance(count)?;
        Ok(self.continue_common())
    }

    /// Internal [`IdbCursor::into_vec`] handler
    async fn handle_into_vec<F, O>(&self, skip: u32, mapper: F) -> Result<Vec<O>, DomException>
    where
        F: Fn(JsValue) -> O,
    {
        if skip != 0 && !self.advance(skip)?.await? {
            return Ok(Vec::new());
        }

        let mut out = match self.key() {
            Some(v) => vec![mapper(v)],
            None => {
                return Ok(Vec::new());
            }
        };

        while self.continue_cursor()?.await? {
            match self.key() {
                Some(key) => {
                    out.push(mapper(key));
                }
                None => {
                    break;
                }
            }
        }

        Ok(out)
    }

    /// Consume the remainder of the cursor, collecting each key into a vector.
    ///
    /// ### Arguments
    ///
    /// - **skip** - how many times to advance the cursor before starting to collect keys. Setting
    ///   this to 0 will include the current key and value in the output; setting it to 5 will skip
    ///   the current key + value and 4 more.
    pub async fn into_vec(self, skip: u32) -> Result<Vec<JsValue>, DomException> {
        fn passthrough(v: JsValue) -> JsValue {
            v
        }
        self.handle_into_vec(skip, passthrough).await
    }

    /// Delete the record at the cursor's position, without changing the cursor's position
    pub fn delete(&self) -> Result<VoidRequest, DomException> {
        Ok(VoidRequest::new(self.inner.delete()?))
    }

    /// Update the value at the current position of the cursor in the object store
    pub fn update<V: JsCast>(
        &self,
        value: &V,
    ) -> Result<impl Future<Output = Result<JsValue, DomException>>, DomException> {
        JsCastRequestFuture::new(self.inner.update(value.unchecked_ref()))
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::needless_pass_by_value)]
pub mod idb_cursor_test {
    use wasm_bindgen::prelude::*;
    use web_sys::DomException;

    use crate::internal_utils::open_any_db;
    use crate::prelude::*;

    test_mod_init!();

    async fn insert_dummy_data(db: &IdbDatabase, store_name: &str) {
        fn add_values(store: &IdbObjectStore) -> Result<(), DomException> {
            store.add_key_val_owned("k1", &JsValue::from(1u8))?;
            store.add_key_val_owned("k2", &JsValue::from(2u8))?;
            store.add_key_val_owned("k3", &JsValue::from(3u8))?;
            store.add_key_val_owned("k4", &JsValue::from(4u8))?;
            Ok(())
        }

        let tx = db
            .transaction_on_one_with_mode(store_name, IdbTransactionMode::Readwrite)
            .expect("Start insert tx open");
        let store = tx.object_store(store_name).expect("Start insert store");

        add_values(&store).expect("Start insert add_values");
        tx.await.into_result().expect("Start insert tx await");
    }

    async fn open_dummy_db() -> (IdbDatabase, String) {
        let ret = open_any_db().await;
        insert_dummy_data(&ret.0, &ret.1).await;
        ret
    }

    fn map_key(k: Option<JsValue>) -> Option<String> {
        k?.as_string()
    }

    #[allow(clippy::cast_sign_loss)]
    fn map_value(v: JsValue) -> u8 {
        v.as_f64().expect("failed to unwrap value as f64") as u8
    }

    async fn do_continue_cursor<'a>(cur: &'a IdbCursor<'a, IdbObjectStore<'a>>) -> bool {
        cur.continue_cursor()
            .expect("continue_cursor")
            .await
            .expect("continue_cursor_await")
    }

    async fn open_cur<'a>(
        store: &'a IdbObjectStore<'a>,
    ) -> IdbCursorWithValue<'a, IdbObjectStore<'a>> {
        store
            .open_cursor()
            .expect("open_cursor")
            .await
            .expect("open_cursor await")
            .expect("initial cursor empty")
    }

    test_case!(async idb_cursor_to_vec => {
        let (db, store_name) = open_dummy_db().await;
        let tx = db.transaction_on_one(&store_name).unwrap();
        let store = tx.object_store(&store_name).unwrap();
        let cur = store.open_key_cursor().unwrap().await.unwrap().unwrap();
        let cur: Vec<String> = cur.into_vec(2).await.unwrap()
          .into_iter()
          .map(|v| v.as_string().unwrap())
          .collect();
        let exp: Vec<String> = vec!["k3".into(), "k4".into()];

        assert_eq!(cur, exp);
    });

    test_case!(async delete_and_update => {
        async fn process<'a>(cur: &'a IdbCursorWithValue<'a, IdbObjectStore<'a>>) {
            let key = map_key(cur.key()).expect("unwrap_key");
            if key.as_str() == "k3" {
                cur.delete().expect("delete").await.expect("delete await");
            } else if key.as_str() == "k4" {
                cur.update(&JsValue::from(100u8)).expect("update").await.expect("update await");
            }
        }

        let (db, store_name) = open_dummy_db().await;
        let tx = db.transaction_on_one_with_mode(&store_name, IdbTransactionMode::Readwrite)
            .unwrap();
        let store = tx.object_store(&store_name).unwrap();
        let cur = open_cur(&store).await;

        process(&cur).await;
        while do_continue_cursor(&cur).await {
            process(&cur).await;
        }

        tx.await.into_result().expect("first tx await");

        let tx = db.transaction_on_one(&store_name).unwrap();
        let store = tx.object_store(&store_name).unwrap();
        let mut cur = open_cur(&store).await.into_vec(1).await.unwrap();
        cur.sort_by_key(|a| a.key().as_string());

        let expected = vec![
            KeyVal::new("k2".into(), JsValue::from(2u8)),
            KeyVal::new("k4".into(), JsValue::from(100u8))
        ];

        assert_eq!(cur, expected);
    });

    pub mod iteration {
        test_mod_init!();

        test_case!(async cursor => {
            let (db, store_name) = open_dummy_db().await;
            let tx = db.transaction_on_one(&store_name).unwrap();
            let store = tx.object_store(&store_name).unwrap();
            let cur = open_cur(&store).await;

            let mut result: Vec<(String, u8)> = Vec::with_capacity(4);
            result.push((map_key(cur.key()).unwrap(), map_value(cur.value())));
            while do_continue_cursor(&cur).await {
                result.push((map_key(cur.key()).unwrap(), map_value(cur.value())));
            }

            let exp = vec![
                ("k1".into(), 1),
                ("k2".into(), 2),
                ("k3".into(), 3),
                ("k4".into(), 4)
            ];

            assert_eq!(result, exp);
        });

        test_case!(async key_cursor => {
            let (db, store_name) = open_dummy_db().await;
            let tx = db.transaction_on_one(&store_name).unwrap();
            let store = tx.object_store(&store_name).unwrap();
            let cur = store
                .open_key_cursor()
                .expect("open_key_cursor")
                .await
                .expect("open_key_cursor await")
                .expect("initial cursor empty");

            let mut result: Vec<Option<String>> = Vec::with_capacity(3);
            result.push(map_key(cur.key()));
            {
                let r = cur.advance(2).expect("advance").await.expect("advance await");
                assert!(r, "advance result");
            }
            result.push(map_key(cur.key()));
            while do_continue_cursor(&cur).await {
                result.push(map_key(cur.key()));
            }
            result.sort();

            let exp = vec![Some("k1".into()), Some("k3".into()), Some("k4".into())];

            assert_eq!(result, exp);
        });
    }
}
