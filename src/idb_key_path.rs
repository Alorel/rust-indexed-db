use std::hash::{Hash, Hasher};

use wasm_bindgen::{prelude::*, JsCast};

use crate::internal_utils::arrayify_slice;

/// A [key path](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#gloss_keypath)
/// representation.
#[derive(Debug, Clone, PartialEq)]
pub struct IdbKeyPath(JsValue);

impl IdbKeyPath {
    /// Create a key path from a &str
    #[inline]
    pub fn str(key_path: &str) -> Self {
        Self::new(key_path.into())
    }

    /// Create a key path from a &str sequence
    #[inline]
    pub fn str_sequence(key_paths: &[&str]) -> Self {
        Self::new(arrayify_slice(key_paths).unchecked_into())
    }

    /// Create a key path from a JsValue. The value should be a JS string or array of strings.
    #[inline]
    pub fn new(key_paths: JsValue) -> Self {
        Self(key_paths)
    }

    /// The converted JS value
    #[inline]
    pub fn as_js_value(&self) -> &JsValue {
        &self.0
    }

    pub(crate) fn try_from_js(v: Result<JsValue, JsValue>) -> Option<Self> {
        let v = v.ok()?;
        if v.is_null() {
            None
        } else {
            Some(Self::new(v))
        }
    }
}

impl AsRef<JsValue> for IdbKeyPath {
    #[inline]
    fn as_ref(&self) -> &JsValue {
        self.as_js_value()
    }
}

impl From<IdbKeyPath> for JsValue {
    #[inline]
    fn from(path: IdbKeyPath) -> Self {
        path.0
    }
}

impl From<JsValue> for IdbKeyPath {
    #[inline]
    fn from(val: JsValue) -> Self {
        Self::new(val)
    }
}

impl From<&str> for IdbKeyPath {
    #[inline]
    fn from(key_path: &str) -> Self {
        Self::str(key_path)
    }
}

impl From<&[&str]> for IdbKeyPath {
    #[inline]
    fn from(key_path: &[&str]) -> Self {
        Self::str_sequence(key_path)
    }
}

impl From<Vec<&str>> for IdbKeyPath {
    #[inline]
    fn from(key_path: Vec<&str>) -> Self {
        Self::str_sequence(key_path.as_slice())
    }
}

impl Hash for IdbKeyPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(v) = self.as_js_value().as_string() {
            state.write_u8(1);
            v.hash(state);
        } else {
            state.write_u8(2);
            hash_array(self.as_js_value().unchecked_ref(), state);
        }
    }
}

fn hash_array<H: Hasher>(arr: &js_sys::Array, h: &mut H) {
    let len = arr.length() as u32;

    h.write_u32(len);

    for i in 0..len {
        if let Some(v) = arr.get(i).as_string() {
            h.write_u8(1);
            v.hash(h);
        } else {
            h.write(&[0, 0]);
        }
    }
}
