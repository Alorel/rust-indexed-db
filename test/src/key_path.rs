use crate::prelude::*;
use idb_fut::KeyPath;
use smallvec::smallvec;

#[wasm_bindgen_test]
pub fn one() {
    let js = KeyPath::from("foo").to_js();
    let rs = KeyPath::from(js.clone());
    let expect = KeyPath::One(String::from("foo"));

    assert_eq!(rs, expect);
}

#[wasm_bindgen_test]
pub fn sequence() {
    let js = KeyPath::from(["foo", "bar"]).to_js();
    let rs = KeyPath::from(js.clone());
    let expect = KeyPath::Sequence(smallvec![String::from("foo"), String::from("bar")]);

    assert_eq!(rs, expect);
}

#[wasm_bindgen_test]
pub fn raw() {
    let js = KeyPath::from(JsValue::NULL).to_js();
    let rs = KeyPath::from(js.clone());
    let expect = KeyPath::JsValue(JsValue::NULL);

    assert_eq!(rs, expect);
}
