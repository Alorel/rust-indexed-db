use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::task::Waker;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

/// unwrap_unchecked if running in nightly, else just unwrap
#[inline]
pub(crate) fn safe_unwrap_option<T>(option: Option<T>) -> T {
    cfg_if! {
        if #[cfg(feature = "nightly")] {
            unsafe { option.unwrap_unchecked() }
        } else {
            option.unwrap()
        }
    }
}

/// unwrap_unchecked if running in nightly, else just unwrap
#[inline]
pub(crate) fn safe_unwrap_result<O, E: std::fmt::Debug>(result: Result<O, E>) -> O {
    cfg_if! {
        if #[cfg(feature = "nightly")] {
            unsafe { result.unwrap_unchecked() }
        } else {
            result.unwrap()
        }
    }
}

/// Wake the given option ref cell
pub(crate) fn wake(waker: &RefCell<Option<Waker>>) {
    if let Some(w) = waker.borrow().deref() {
        w.wake_by_ref();
    }
}

/// Return `None` if `val` is undefined, else `Some(val)`
#[inline]
pub(crate) fn optional_jsvalue_undefined(val: JsValue) -> Option<JsValue> {
    if val.is_undefined() {
        None
    } else {
        Some(val)
    }
}

#[inline]
pub(crate) fn create_lazy_ref_cell<T>() -> Rc<RefCell<Option<T>>> {
    Rc::new(RefCell::new(None))
}

pub(crate) fn arrayify_slice(slice: &[&str]) -> js_sys::Array {
    slice.iter().map(jsvalue_from).collect()
}

#[inline]
fn jsvalue_from(v: &&str) -> JsValue {
    JsValue::from_str(v)
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub mod arrayify_slice {
        use js_sys::Array;

        test_mod_init!();

        fn assert_array_eq(arr: &Array, slice: &[&str]) {
            let arr_length = arr.length();
            assert_eq!(arr.length() as usize, slice.len(), "Lengths");
            for i in 0..arr_length {
                assert_eq!(
                    arr.get(i),
                    JsValue::from_str(slice[i as usize]),
                    "Item at idx {}",
                    i
                );
            }
        }

        test_case!(empty_slice => {
            assert_array_eq(&Array::new(), &[]);
        });

        test_case!(non_empty_slice => {
            assert_array_eq(&Array::of2(&"foo".into(), &"bar".into()), &["foo", "bar"]);
        });
    }

    pub mod optional_jsvalue_undefined {
        test_mod_init!();

        macro_rules! run_case {
            ($name: ident, $val: expr, $expect: literal) => {
                test_case!($name => {
                    let val = optional_jsvalue_undefined($val).is_none();
                    assert_eq!(val, $expect);
                });
            };
        }

        run_case!(undefined, JsValue::undefined(), true);
        run_case!(null, JsValue::null(), false);
        run_case!(string, JsValue::from_str("foo"), false);
        run_case!(num_0, JsValue::from(0), false);
        run_case!(bool_false, JsValue::from(false), false);
    }
}
