pub mod key_cursor {
    use crate::prelude::*;
    use idb_fut::cursor::CursorDirection;

    #[wasm_bindgen_test]
    pub async fn empty() {
        let db = random_db_idx_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;
        open_idx!(db, Readonly > idx);

        let cur = {
            let req = idx.open_key_cursor().with_query(random_str());
            dyn_await!(req).unwrap()
        };
        assert!(cur.is_none());
    }

    #[wasm_bindgen_test]
    pub async fn next() {
        let db = random_db_idx_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_key_cursor()).unwrap().unwrap();
            next_key!(Value, cursor)
        };

        assert_eq!(next, Ok(Some(Value::MIN)));
    }

    #[wasm_bindgen_test]
    pub async fn direction() {
        let db = random_db_idx_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_idx!(db, Readonly > idx);
            let req = idx.open_key_cursor().with_direction(CursorDirection::Prev);
            let mut cursor = dyn_await!(req).unwrap().unwrap();
            next_key!(Value, cursor)
        };

        assert_eq!(next, Ok(Some(Value::MAX)));
    }

    pub mod base_cursor {
        use crate::prelude::*;

        #[wasm_bindgen_test]
        pub async fn primary_key() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_key_cursor()).unwrap().unwrap();

            let initial = primary_key!(Key, cursor);
            assert_eq!(initial, Ok(Some(Key::MAX)));

            // First next() shouldn't actually advance the cursor, but, rather, return the 1st element without advancing
            let next_key = next_key!(Value, cursor);
            assert_eq!(next_key, Ok(Some(Value::MIN)));

            // Should remain the same
            assert_eq!(primary_key!(Key, cursor), initial);

            let next_key = next_key!(Value, cursor);
            assert_eq!(next_key, Ok(Some(Value::MIN + 1)));

            assert_eq!(primary_key!(Key, cursor), Ok(Some(Key::MAX - 1)));
        }

        #[wasm_bindgen_test]
        pub async fn advance_by() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_key_cursor()).unwrap().unwrap();

            cursor.advance_by(5).await.unwrap();

            let pk1 = primary_key!(Key, cursor);
            let key1 = next_key!(Value, cursor);
            let pk2 = primary_key!(Key, cursor);
            let key2 = next_key!(Value, cursor);
            let pk3 = primary_key!(Key, cursor);

            assert_eq!(pk1, Ok(Some(Key::MAX - 5)));
            assert_eq!(key1, Ok(Some(Value::MIN + 5)));
            assert_eq!(pk2, Ok(Some(Key::MAX - 5)));
            assert_eq!(key2, Ok(Some(Value::MIN + 6)));
            assert_eq!(pk3, Ok(Some(Key::MAX - 6)));

            cursor.advance_by(u8::MAX as u32).await.unwrap();

            assert_dom_exc!(next_key!(Key, cursor).unwrap_err(), InvalidStateError);
            assert_dom_exc!(cursor.advance_by(1).await.unwrap_err(), InvalidStateError);
        }

        #[wasm_bindgen_test]
        pub async fn continue_to_primary_key() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_key_cursor()).unwrap().unwrap();

            let invalid = {
                let key = js_sys::Symbol::for_(&random_str()).unchecked_into::<JsValue>();
                cursor.continue_to_primary_key(&key, &key).await
            }
            .unwrap_err();
            assert_dom_exc!(invalid, DataError);

            assert_eq!(
                continue_to_pkey!(Value::new(5), Key::new(-5), cursor),
                Ok(())
            );

            let pk = primary_key!(Key, cursor);
            let key = next_key!(Value, cursor);
            let pk2 = primary_key!(Key, cursor);

            assert_eq!(pk, Ok(Some(Key::new(-5))));
            assert_eq!(key, Ok(Some(Value::new(5))));
            assert_eq!(pk2, Ok(Some(Key::new(-5))));
        }

        #[wasm_bindgen_test]
        pub async fn continue_to_key() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_key_cursor()).unwrap().unwrap();

            let invalid = {
                let key = js_sys::Symbol::for_(&random_str()).unchecked_into::<JsValue>();
                cursor.continue_to_key(key)
            }
            .await
            .unwrap_err();
            assert_dom_exc!(invalid, DataError);

            assert_eq!(continue_to_key!(Value::new(5), cursor), Ok(()));

            let pk = primary_key!(Key, cursor);
            let key = next_key!(Value, cursor);
            let pk2 = primary_key!(Key, cursor);

            assert_eq!(pk, Ok(Some(Key::new(-5))));
            assert_eq!(key, Ok(Some(Value::new(5))));
            assert_eq!(pk2, Ok(Some(Key::new(-5))));
        }
    }

    #[cfg(feature = "streams")]
    pub mod key_stream {
        use crate::prelude::*;
        use futures::TryStreamExt;
        use idb_fut::cursor::CursorDirection;

        #[wasm_bindgen_test]
        pub async fn all() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let cursor = dyn_await!(idx.open_key_cursor()).unwrap().unwrap();
            let stream = open_kstream!(cursor, Value);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = Value::iter_range().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_query() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let cursor = {
                let req = idx
                    .open_key_cursor()
                    .with_query(Value::new(4)..=Value::new(7));
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Value);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (4u8..=7).map(Value::new).collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_direction() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let cursor = {
                let req = idx.open_key_cursor().with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Value);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = Value::iter_range().rev().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_query_and_direction() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let cursor = {
                let req = idx
                    .open_key_cursor()
                    .with_query(Value::new(4)..=Value::new(7))
                    .with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Value);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (4u8..=7).map(Value::new).rev().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }
    }
}

#[allow(clippy::module_inception)]
pub mod cursor {
    use crate::prelude::*;
    use idb_fut::cursor::CursorDirection;

    #[wasm_bindgen_test]
    pub async fn empty() {
        let db = random_db_idx_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;
        open_idx!(db, Readonly > idx);

        let cur = {
            let req = idx.open_cursor().with_query(random_str());
            dyn_await!(req).unwrap()
        };
        assert!(cur.is_none());
    }

    #[wasm_bindgen_test]
    pub async fn next() {
        let db = random_db_idx_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_cursor()).unwrap().unwrap();
            next_record!(KeyVal, cursor)
        };

        assert_eq!(next, Ok(Some(KeyVal::new(Key::MAX, Value::MIN))));
    }

    #[wasm_bindgen_test]
    pub async fn iterate_whole_db() {
        let db = random_db_idx_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        open_idx!(db, Readonly > idx);
        let mut all = Vec::with_capacity(KeyVal::RANGE_LEN as usize);
        let mut cursor = dyn_await!(idx.open_cursor()).unwrap().unwrap();

        while let Some(record) = next_record!(KeyVal, cursor).unwrap() {
            all.push(record);
        }

        let expect = KeyVal::iter_range().rev().collect::<Vec<_>>();
        assert_eq!(all, expect);
    }

    #[wasm_bindgen_test]
    pub async fn direction() {
        let db = random_db_idx_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_idx!(db, Readonly > idx);
            let req = idx.open_cursor().with_direction(CursorDirection::Prev);
            let mut cursor = dyn_await!(req).unwrap().unwrap();
            next_record!(KeyVal, cursor)
        };

        assert_eq!(next, Ok(Some(KeyVal::new(Key::MIN, Value::MAX))));
    }

    pub mod base_cursor {
        pub use crate::prelude::*;

        #[wasm_bindgen_test]
        pub async fn primary_key() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_cursor()).unwrap().unwrap();

            let initial = primary_key!(Key, cursor);
            assert_eq!(initial, Ok(Some(Key::MAX)));

            // First next() shouldn't actually advance the cursor, but, rather, return the 1st element without advancing
            let next_record = next_record!(KeyVal, cursor);
            assert_eq!(next_record, Ok(Some(KeyVal::new(Key::MAX, Value::MIN))));

            // Should remain the same
            assert_eq!(primary_key!(Key, cursor), initial);

            let next_record = next_record!(KeyVal, cursor);
            assert_eq!(
                next_record,
                Ok(Some(KeyVal::new(Key::MAX - 1, Value::MIN + 1)))
            );

            assert_eq!(primary_key!(Key, cursor), Ok(Some(Key::MAX - 1)));
        }

        #[wasm_bindgen_test]
        pub async fn advance_by() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_cursor()).unwrap().unwrap();

            cursor.advance_by(5).await.unwrap();

            let pk1 = primary_key!(Key, cursor);
            let record1 = next_record!(KeyVal, cursor);
            let pk2 = primary_key!(Key, cursor);
            let record2 = next_record!(KeyVal, cursor);
            let pk3 = primary_key!(Key, cursor);

            assert_eq!(pk1, Ok(Some(Key::MAX - 5)));
            assert_eq!(record1, Ok(Some(KeyVal::new(Key::MAX - 5, Value::MIN + 5))));
            assert_eq!(pk2, Ok(Some(Key::MAX - 5)));
            assert_eq!(record2, Ok(Some(KeyVal::new(Key::MAX - 6, Value::MIN + 6))));
            assert_eq!(pk3, Ok(Some(Key::MAX - 6)));

            cursor.advance_by(u8::MAX as u32).await.unwrap();

            assert_dom_exc!(next_record!(KeyVal, cursor).unwrap_err(), InvalidStateError);
            assert_dom_exc!(cursor.advance_by(1).await.unwrap_err(), InvalidStateError);
        }

        #[wasm_bindgen_test]
        pub async fn continue_to_key() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_cursor()).unwrap().unwrap();

            let invalid = {
                let key = js_sys::Symbol::for_(&random_str()).unchecked_into::<JsValue>();
                cursor.continue_to_key(key)
            }
            .await
            .unwrap_err();
            assert_dom_exc!(invalid, DataError);

            assert_eq!(continue_to_key!(Value::new(5), cursor), Ok(()));

            let pk = primary_key!(Key, cursor);
            let key = next_record!(KeyVal, cursor);
            let pk2 = primary_key!(Key, cursor);

            assert_eq!(pk, Ok(Some(Key::new(-5))));
            assert_eq!(key, Ok(Some(KeyVal::new(Key::new(-5), Value::new(5)))));
            assert_eq!(pk2, Ok(Some(Key::new(-5))));
        }

        #[wasm_bindgen_test]
        pub async fn continue_to_primary_key() {
            let db = random_db_idx_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_idx!(db, Readonly > idx);
            let mut cursor = dyn_await!(idx.open_cursor()).unwrap().unwrap();

            let invalid = {
                let key = js_sys::Symbol::for_(&random_str()).unchecked_into::<JsValue>();
                cursor.continue_to_key(key)
            }
            .await
            .unwrap_err();
            assert_dom_exc!(invalid, DataError);

            assert_eq!(
                continue_to_pkey!(Value::new(5), Key::new(-5), cursor),
                Ok(())
            );

            let pk = primary_key!(Key, cursor);
            let key = next_record!(KeyVal, cursor);
            let pk2 = primary_key!(Key, cursor);

            assert_eq!(pk, Ok(Some(Key::new(-5))));
            assert_eq!(key, Ok(Some(KeyVal::new(Key::new(-5), Value::new(5)))));
            assert_eq!(pk2, Ok(Some(Key::new(-5))));
        }
    }
}
