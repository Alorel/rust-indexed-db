use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use wasm_bindgen::prelude::*;

/// Convert to a [`JsValue`].
pub trait TryToJs {
    /// Convert to a [`JsValue`].
    #[allow(clippy::missing_errors_doc)]
    fn try_to_js(&self) -> crate::Result<JsValue>;

    /// Convert into a [`JsValue`].
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    fn try_into_js(self) -> crate::Result<JsValue>
    where
        Self: Sized,
    {
        self.try_to_js()
    }
}

impl<T: TryToJs> TryToJs for &T {
    #[inline]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        TryToJs::try_to_js(*self)
    }
}

impl<T: TryToJs> TryToJs for Option<T> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        match *self {
            Some(ref v) => v.try_to_js(),
            None => Ok(JsValue::UNDEFINED),
        }
    }
}

impl TryToJs for char {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        String::from(*self).try_to_js()
    }
}

impl TryToJs for &str {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        Ok(JsValue::from_str(self))
    }
}

impl TryToJs for String {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        self.as_str().try_to_js()
    }
}

impl TryToJs for JsValue {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        Ok(self.clone())
    }
}

impl TryToJs for bool {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        Ok(if *self { JsValue::TRUE } else { JsValue::FALSE })
    }
}

fn list_to_js<E, It>(iter: It) -> crate::Result<JsValue>
where
    It: IntoIterator<Item = E>,
    E: TryToJs,
{
    let arr = iter
        .into_iter()
        .map(TryToJs::try_into_js)
        .collect::<crate::Result<js_sys::Array>>()?;

    Ok(arr.unchecked_into())
}

pub(crate) fn map_to_js<K, V, It>(iter: It) -> crate::Result<JsValue>
where
    It: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: TryToJs,
{
    let obj = js_sys::Object::new();

    for (k, v) in iter {
        let v = v.try_to_js()?;
        js_sys::Reflect::set(&obj, &JsValue::from_str(k.as_ref()), &v)?;
    }

    Ok(obj.unchecked_into())
}

impl<K: AsRef<str>, V: TryToJs> TryToJs for BTreeMap<K, V> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        map_to_js(self.iter())
    }
}

impl<K: AsRef<str>, V: TryToJs, H> TryToJs for HashMap<K, V, H> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        map_to_js(self.iter())
    }
}

impl<T: TryToJs> TryToJs for &[T] {
    #[inline]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        list_to_js(*self)
    }
}

impl<T: TryToJs, const N: usize> TryToJs for [T; N] {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        list_to_js(self.iter())
    }
}

impl<T: TryToJs> TryToJs for Vec<T> {
    #[inline]
    fn try_to_js(&self) -> crate::Result<JsValue> {
        self.as_slice().try_to_js()
    }
}

impl<T: TryToJs> TryToJs for VecDeque<T> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        list_to_js(self.iter())
    }
}

impl<T: TryToJs, H> TryToJs for HashSet<T, H> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        list_to_js(self.iter())
    }
}

impl<T: TryToJs> TryToJs for BTreeSet<T> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        list_to_js(self.iter())
    }
}

impl<T: TryToJs> TryToJs for LinkedList<T> {
    fn try_to_js(&self) -> crate::Result<JsValue> {
        list_to_js(self.iter())
    }
}

macro_rules! impl_for_num {
    ($($ty: ident),+ $(,)?) => {
        $(
            impl TryToJs for $ty {
                #[inline]
                fn try_to_js(&self) -> crate::Result<JsValue> {
                    Ok(JsValue::from(*self))
                }
            }
        )+
    };
}

impl_for_num!(f32, f64, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, i128, u128);
