use crate::prelude::*;
use rand::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

macro_rules! check {
    ($value: ident, $ty: ty) => {{
        let js = $value
            .try_to_js()
            .expect(concat!("`", stringify!($ty), "` to js"));
        let rs = <$ty>::from_js(js).expect(concat!("`", stringify!($ty), "` from js"));
        assert_eq!(rs, $value, stringify!($ty));
    }};
}

#[wasm_bindgen_test]
pub fn floats() {
    let mut rng = thread_rng();

    macro_rules! gen {
        ($rng: ident, $($ty: ty),+) => {
            $({
                let value = $rng.gen::<u16>() as $ty;
                check!(value, $ty);
            })+
        };
    }

    gen!(rng, f32, f64);
}

#[wasm_bindgen_test]
pub fn integers() {
    let mut rng = thread_rng();
    macro_rules! gen {
        ($rng: ident, $($ty: ty),+) => {
            $({
                let value: $ty = $rng.gen();
                check!(value, $ty);
            })+
        };
    }

    gen!(rng, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, i128, u128);
}

#[wasm_bindgen_test]
pub fn lists() {
    let mut rng = thread_rng();
    macro_rules! gen {
        ($rng: ident, $($ty: ty),+ $(,)?) => {$({
            let value = <$ty>::from_iter([$rng.gen(), $rng.gen()]);
            check!(value, $ty);
        })+};
    }

    gen!(
        rng,
        Vec<usize>,
        VecDeque<isize>,
        HashSet<usize>,
        BTreeSet<isize>
    );

    let v = rng.gen::<u8>();
    let js = [v].try_to_js().expect("Arr to");
    assert_eq!(
        Vec::<u8>::from_js(js).expect("arr from"),
        vec![v],
        "arr from cmp"
    );

    let js = [v].as_slice().try_to_js().expect("Arr to");
    assert_eq!(
        Vec::<u8>::from_js(js).expect("slice from"),
        vec![v],
        "slice from cmp"
    );
}

#[wasm_bindgen_test]
pub fn maps() {
    let mut rng = thread_rng();

    macro_rules! gen {
        ($rng: ident, $($ty: ty),+ $(,)?) => {
            $({
                let value = <$ty>::from_iter([(random_str(), $rng.gen())]);
                check!(value, $ty);
            })+
        };
    }

    gen!(rng, HashMap<String, isize>, BTreeMap<String, usize>);
}

#[wasm_bindgen_test]
pub fn options() {
    {
        let value = Some(random::<u8>());
        check!(value, Option<u8>);
    }

    assert_eq!(
        None::<u8>.try_to_js().expect("none to js"),
        JsValue::UNDEFINED,
        "none to js cmp"
    );

    assert_eq!(
        None,
        <Option<u8>>::from_js(JsValue::UNDEFINED).expect("undefined from js"),
        "undefined from js cmp"
    );

    assert_eq!(
        None,
        <Option<u8>>::from_js(JsValue::NULL).expect("null from js"),
        "null from js cmp"
    );
}
