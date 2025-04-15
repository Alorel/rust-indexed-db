macro_rules! common_tests {
    ($method: ident) => {
        #[wasm_bindgen_test]
        pub async fn readonly_error() {
            let db = random_db_keyval().await;
            open_tx!(db, Readonly > (tx, store));
            let err = store.$method(Value::default()).build_dyn().unwrap_err();

            assert_dom_exc!(err, ReadOnlyError);
        }

        #[wasm_bindgen_test]
        pub async fn data_error_inline_key() {
            let db = random_db_with_init(move |_, db| {
                db.create_object_store(&db.name())
                    .with_key_path("foo".into())
                    .build()?;
                Ok(())
            })
            .await;
            open_tx!(db, Readwrite > (tx, store));
            let err = store
                .$method(Value::default())
                .with_key(Key::default())
                .build_dyn()
                .unwrap_err();

            assert_dom_exc!(err, DataError);
        }

        #[wasm_bindgen_test]
        pub async fn data_error_out_of_line_key() {
            let db = random_db_with_store().await;
            open_tx!(db, Readwrite > (tx, store));
            let err = store.$method(Value::default()).build_dyn().unwrap_err();

            assert_dom_exc!(err, DataError);
        }

        #[wasm_bindgen_test]
        pub async fn data_error_invalid_key() {
            let db = random_db_with_store().await;
            open_tx!(db, Readwrite > (tx, store));
            let err = store
                .$method("foo")
                .with_key(js_sys::Symbol::for_("foo").unchecked_into::<JsValue>())
                .primitive()
                .unwrap_err();

            assert_dom_exc!(err, DataError);
        }

        #[wasm_bindgen_test]
        pub async fn str_key_autohandling() {
            let expect_str = random_str();
            let db = random_db_with_store().await;
            open_tx!(db, Readwrite > (tx, store));

            let res = store.$method(1u8).with_key(expect_str.as_str()).await;
            assert_eq!(res, Ok(expect_str));
        }

        /// Verify that we get objects and not maps as output
        #[cfg(feature = "serde")]
        #[wasm_bindgen_test]
        pub async fn serde_object_nesting() {
            let db = random_db_with_init(move |_, db| {
                db.create_object_store(&db.name())
                    .with_key_path("foo".into())
                    .build()?;
                Ok(())
            })
            .await;
            open_tx!(db, Readwrite > (tx, store));

            #[derive(serde::Serialize, serde::Deserialize)]
            struct Foo {
                foo: String,
            }

            #[derive(serde::Serialize, serde::Deserialize)]
            struct Bar {
                bar: String,
            }

            #[derive(serde::Serialize, serde::Deserialize)]
            struct Entry {
                #[serde(flatten)]
                key: Foo,
                value: Bar,
            }

            let res = store
                .$method(Entry {
                    key: Foo {
                        foo: "foo".to_string(),
                    },
                    value: Bar {
                        bar: "bar".to_string(),
                    },
                })
                .serde()
                .unwrap()
                .await;

            assert!(res.is_ok());
        }
    };
}

pub mod add {
    use crate::prelude::*;

    common_tests!(add);

    #[wasm_bindgen_test]
    pub async fn duplicate_add() {
        let db = random_db_keyval().await;
        open_tx!(db, Readwrite > (tx, store));

        let record = KeyVal::new(Key::MIN, Value::MIN);
        let initial = dyn_await!(store.add(record).with_key_type::<Key>());
        assert_eq!(initial, Ok(Key::MIN));

        let err = store
            .add(record)
            .with_key_type::<Key>()
            .build_dyn()
            .unwrap()
            .await
            .unwrap_err();
        assert_dom_exc!(err, ConstraintError);
    }
}

pub mod put {
    use crate::prelude::*;

    common_tests!(put);

    #[wasm_bindgen_test]
    pub async fn duplicate_add() {
        let db = random_db_keyval().await;
        open_tx!(db, Readwrite > (tx, store));

        let record = KeyVal::new(Key::MAX, Value::MAX);
        let initial = dyn_await!(store.add(record).with_key_type::<Key>());
        assert_eq!(initial, Ok(Key::MAX));
    }
}
