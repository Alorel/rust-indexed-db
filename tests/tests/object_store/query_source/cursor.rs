pub mod key_cursor {
    use crate::prelude::*;
    use idb_fut::cursor::CursorDirection;

    #[wasm_bindgen_test]
    pub async fn empty() {
        let db = random_db_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;
        open_tx!(db, Readonly > (tx, store));

        let cur = {
            let req = store.open_key_cursor().with_query(random_str());
            dyn_await!(req).unwrap()
        };
        assert!(cur.is_none());
    }

    #[wasm_bindgen_test]
    pub async fn next() {
        let db = random_db_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_key_cursor()).unwrap().unwrap();
            next_key!(Key, cursor)
        };

        assert_eq!(next, Ok(Some(Key::MIN)));
    }

    #[wasm_bindgen_test]
    pub async fn direction() {
        let db = random_db_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_tx!(db, Readonly > (tx, store));
            let req = store
                .open_key_cursor()
                .with_direction(CursorDirection::Prev);
            let mut cursor = dyn_await!(req).unwrap().unwrap();
            next_key!(Key, cursor)
        };

        assert_eq!(next, Ok(Some(Key::MAX)));
    }

    pub mod base_cursor {
        use crate::prelude::*;

        #[wasm_bindgen_test]
        pub async fn primary_key() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_key_cursor()).unwrap().unwrap();

            let initial = primary_key!(Key, cursor);
            assert_eq!(initial, Ok(Some(Key::MIN)));

            // First next() shouldn't actually advance the cursor, but, rather, return the 1st element without advancing
            let next_key = next_key!(Key, cursor);
            assert_eq!(next_key, Ok(Some(Key::MIN)));

            // Should remain the same
            assert_eq!(primary_key!(Key, cursor), initial);

            let next_key = next_key!(Key, cursor);
            assert_eq!(next_key, Ok(Some(Key::MIN + 1)));

            assert_eq!(primary_key!(Key, cursor), Ok(Some(Key::MIN + 1)));
        }

        #[wasm_bindgen_test]
        pub async fn advance_by() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_key_cursor()).unwrap().unwrap();

            cursor.advance_by(5).await.unwrap();

            let pk1 = primary_key!(Key, cursor);
            let key1 = next_key!(Key, cursor);
            let pk2 = primary_key!(Key, cursor);
            let key2 = next_key!(Key, cursor);
            let pk3 = primary_key!(Key, cursor);

            assert_eq!(pk1, Ok(Some(Key::MIN + 5)));
            assert_eq!(key1, Ok(Some(Key::MIN + 5)));
            assert_eq!(pk2, Ok(Some(Key::MIN + 5)));
            assert_eq!(key2, Ok(Some(Key::MIN + 6)));
            assert_eq!(pk3, Ok(Some(Key::MIN + 6)));

            cursor.advance_by(u8::MAX as u32).await.unwrap();

            assert_dom_exc!(next_key!(Key, cursor).unwrap_err(), InvalidStateError);
            assert_dom_exc!(cursor.advance_by(1).await.unwrap_err(), InvalidStateError);
        }

        #[wasm_bindgen_test]
        pub async fn continue_to_key() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_key_cursor()).unwrap().unwrap();

            let invalid = {
                let key = js_sys::Symbol::for_(&random_str()).unchecked_into::<JsValue>();
                cursor.continue_to_key(key)
            }
            .await
            .unwrap_err();
            assert_dom_exc!(invalid, DataError);

            assert_eq!(continue_to_key!(Key::new(-5), cursor), Ok(()));

            let pk = primary_key!(Key, cursor);
            let key = next_key!(Key, cursor);
            let pk2 = primary_key!(Key, cursor);

            assert_eq!(pk, Ok(Some(Key::new(-5))));
            assert_eq!(key, Ok(Some(Key::new(-5))));
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
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = dyn_await!(store.open_key_cursor()).unwrap().unwrap();
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = Key::iter_range().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_query() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store
                    .open_key_cursor()
                    .with_query(Key::new(-7)..=Key::new(-4));
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (-7i8..=-4).map(Key::new).collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_direction() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store
                    .open_key_cursor()
                    .with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = Key::iter_range().rev().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_query_and_direction() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store
                    .open_key_cursor()
                    .with_query(Key::new(-7)..=Key::new(-4))
                    .with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (-7i8..=-4).map(Key::new).rev().collect::<Vec<_>>();

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
        let db = random_db_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;
        open_tx!(db, Readonly > (tx, store));

        let cur = {
            let req = store.open_cursor().with_query(random_str());
            dyn_await!(req).unwrap()
        };
        assert!(cur.is_none());
    }

    #[wasm_bindgen_test]
    pub async fn next() {
        let db = random_db_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();
            next_record!(KeyVal, cursor)
        };

        assert_eq!(next, Ok(Some(KeyVal::new(Key::MIN, Value::MAX))));
    }

    #[wasm_bindgen_test]
    pub async fn direction() {
        let db = random_db_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        let next = {
            open_tx!(db, Readonly > (tx, store));
            let req = store.open_cursor().with_direction(CursorDirection::Prev);
            let mut cursor = dyn_await!(req).unwrap().unwrap();
            next_record!(KeyVal, cursor)
        };

        assert_eq!(next, Ok(Some(KeyVal::new(Key::MAX, Value::MIN))));
    }

    #[wasm_bindgen_test]
    pub async fn iterate_whole_db() {
        let db = random_db_keyval().await;
        KeyVal::insert_keyval_docs(&db).await;

        open_tx!(db, Readonly > (tx, store));
        let mut all = Vec::with_capacity(KeyVal::RANGE_LEN as usize);
        let mut cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();

        while let Some(record) = next_record!(KeyVal, cursor).unwrap() {
            all.push(record);
        }

        let expect = KeyVal::iter_range().collect::<Vec<_>>();
        assert_eq!(all, expect);
    }

    pub mod base_cursor {
        use crate::prelude::*;

        #[wasm_bindgen_test]
        pub async fn primary_key() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();

            let initial = primary_key!(Key, cursor);
            assert_eq!(initial, Ok(Some(Key::MIN)));

            // First next() shouldn't actually advance the cursor, but, rather, return the 1st element without advancing
            let next_record = next_record!(KeyVal, cursor);
            assert_eq!(next_record, Ok(Some(KeyVal::new(Key::MIN, Value::MAX))));

            // Should remain the same
            assert_eq!(primary_key!(Key, cursor), initial);

            let next_record = next_record!(KeyVal, cursor);
            assert_eq!(
                next_record,
                Ok(Some(KeyVal::new(Key::MIN + 1, Value::MAX - 1)))
            );

            assert_eq!(primary_key!(Key, cursor), Ok(Some(Key::MIN + 1)));
        }

        #[wasm_bindgen_test]
        pub async fn advance_by() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();

            cursor.advance_by(5).await.unwrap();

            let pk1 = primary_key!(Key, cursor);
            let record1 = next_record!(KeyVal, cursor);
            let pk2 = primary_key!(Key, cursor);
            let record2 = next_record!(KeyVal, cursor);
            let pk3 = primary_key!(Key, cursor);

            assert_eq!(pk1, Ok(Some(Key::MIN + 5)));
            assert_eq!(record1, Ok(Some(KeyVal::new(Key::MIN + 5, Value::MAX - 5))));
            assert_eq!(pk2, Ok(Some(Key::MIN + 5)));
            assert_eq!(record2, Ok(Some(KeyVal::new(Key::MIN + 6, Value::MAX - 6))));
            assert_eq!(pk3, Ok(Some(Key::MIN + 6)));

            cursor.advance_by(u8::MAX as u32).await.unwrap();

            assert_dom_exc!(next_record!(KeyVal, cursor).unwrap_err(), InvalidStateError);
            assert_dom_exc!(cursor.advance_by(1).await.unwrap_err(), InvalidStateError);
        }

        #[wasm_bindgen_test]
        pub async fn continue_to_key() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let mut cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();

            let invalid = {
                let key = js_sys::Symbol::for_(&random_str()).unchecked_into::<JsValue>();
                cursor.continue_to_key(key)
            }
            .await
            .unwrap_err();
            assert_dom_exc!(invalid, DataError);

            assert_eq!(continue_to_key!(Key::new(-5), cursor), Ok(()));

            let pk = primary_key!(Key, cursor);
            let key = next_record!(KeyVal, cursor);
            let pk2 = primary_key!(Key, cursor);

            assert_eq!(pk, Ok(Some(Key::new(-5))));
            assert_eq!(key, Ok(Some(KeyVal::new(Key::new(-5), Value::new(5)))));
            assert_eq!(pk2, Ok(Some(Key::new(-5))));
        }
    }

    #[cfg(feature = "streams")]
    pub mod stream {
        use crate::prelude::*;
        use futures::{StreamExt, TryStreamExt};
        use idb_fut::cursor::CursorDirection;
        use idb_fut::database::Database;

        #[wasm_bindgen_test]
        pub async fn all() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();
            let stream = open_stream!(cursor, KeyVal);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = KeyVal::iter_range().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        #[cfg(all(feature = "serde", feature = "streams", feature = "cursors"))]
        pub async fn cursor_stream_one() {
            use serde::{Deserialize, Serialize};
            let db = random_db_keyval().await;
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
            struct Foo {
                a: String,
                b: u32,
                key: u32,
            }
            let data_1 = Foo {
                a: "hello".to_string(),
                b: 42,
                key: 1,
            };
            let data_2 = Foo {
                a: "world".to_string(),
                b: 42,
                key: 2,
            };
            // seed db
            {
                open_tx!(db, Readwrite > (tx, store));

                store.add(data_1.clone()).serde().unwrap().await.unwrap();
                store.add(data_2.clone()).serde().unwrap().await.unwrap();
                drop(store);
                tx.commit().await.unwrap();
            }
            open_tx!(db, Readonly > (tx, store));
            let cursor = store.open_cursor().serde().unwrap().await.unwrap().unwrap();
            let res: Vec<Foo> = cursor.stream_ser::<Foo>().try_collect().await.unwrap();
            assert_eq!(res.len(), 2);
            assert_eq!(res.first().unwrap(), &data_1);
            assert_eq!(res.get(1).unwrap(), &data_2);
        }

        #[wasm_bindgen_test]
        pub async fn with_query() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store.open_cursor().with_query(Key::new(-7)..=Key::new(-4));
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_stream!(cursor, KeyVal);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (-7i8..=-4)
                .map(move |k| KeyVal::new(k, -k as u8))
                .collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_direction() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store.open_cursor().with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_stream!(cursor, KeyVal);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = KeyVal::iter_range().rev().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_query_and_direction() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store
                    .open_cursor()
                    .with_query(Key::new(-7)..=Key::new(-4))
                    .with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_stream!(cursor, KeyVal);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (-7i8..=-4)
                .map(move |k| KeyVal::new(k, -k as u8))
                .rev()
                .collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn update() {
            async fn mutate(db: &Database) {
                open_tx!(db, Readwrite > (tx, store));
                let cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();
                let mut stream = open_stream!(cursor, KeyVal);

                while let Some(kv) = next_record!(KeyVal, stream).unwrap() {
                    let new_record = KeyVal::new(kv.key(), kv.value() * 2);
                    stream.update(new_record).build_dyn().unwrap();
                }

                drop(store);
                tx.commit().await.unwrap();
            }

            async fn read(db: &Database) {
                open_tx!(db, Readonly > (tx, store));
                let cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();
                let stream = open_stream!(cursor, KeyVal);

                let data = stream.try_collect::<Vec<_>>().await.unwrap();
                let expect = KeyVal::iter_range()
                    .map(move |kv| KeyVal::new(kv.key(), kv.value() * 2))
                    .collect::<Vec<_>>();

                assert_eq!(data, expect);
            }

            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            mutate(&db).await;
            read(&db).await;
        }

        #[wasm_bindgen_test]
        pub async fn delete() {
            async fn mutate(db: &Database) {
                open_tx!(db, Readwrite > (tx, store));
                let cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();
                let mut stream = open_stream!(cursor, KeyVal);

                stream.delete().unwrap();
                assert_eq!(primary_key!(Key, stream), Ok(Some(Key::MIN)));
                assert_eq!(
                    stream.next().await,
                    Some(Ok(KeyVal::new(Key::MIN + 1, Value::MAX - 1)))
                );
                assert_eq!(primary_key!(Key, stream), Ok(Some(Key::MIN + 1)));

                assert_eq!(
                    stream.next().await,
                    Some(Ok(KeyVal::new(Key::MIN + 2, Value::MAX - 2)))
                );
                stream.delete().unwrap();

                drop(store);
                tx.commit().await.unwrap();
            }

            async fn read(db: &Database) {
                open_tx!(db, Readonly > (tx, store));
                let cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();
                let stream = open_stream!(cursor, KeyVal);

                let data = stream.try_collect::<Vec<_>>().await.unwrap();
                let expect = KeyVal::iter_range()
                    .filter(move |kv| kv.key() != Key::MIN && kv.key() != Key::MIN + 2)
                    .collect::<Vec<_>>();

                assert_eq!(data, expect);
            }

            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;

            mutate(&db).await;
            read(&db).await;
        }
    }

    #[cfg(feature = "streams")]
    pub mod key_stream {
        use crate::prelude::*;
        use idb_fut::cursor::CursorDirection;

        #[wasm_bindgen_test]
        pub async fn all() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = dyn_await!(store.open_cursor()).unwrap().unwrap();
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = Key::iter_range().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_query() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store.open_cursor().with_query(Key::new(-7)..=Key::new(-4));
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (-7i8..=-4).map(Key::new).collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_direction() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store.open_cursor().with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = Key::iter_range().rev().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }

        #[wasm_bindgen_test]
        pub async fn with_query_and_direction() {
            let db = random_db_keyval().await;
            KeyVal::insert_keyval_docs(&db).await;
            open_tx!(db, Readonly > (tx, store));
            let cursor = {
                let req = store
                    .open_cursor()
                    .with_query(Key::new(-7)..=Key::new(-4))
                    .with_direction(CursorDirection::Prev);
                dyn_await!(req).unwrap().unwrap()
            };
            let stream = open_kstream!(cursor, Key);

            let data = stream.try_collect::<Vec<_>>().await.unwrap();
            let expected = (-7i8..=-4).map(Key::new).rev().collect::<Vec<_>>();

            assert_eq!(data, expected);
        }
    }
}
