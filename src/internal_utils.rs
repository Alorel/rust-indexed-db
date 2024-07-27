use wasm_bindgen::prelude::*;

pub use internal_macros::StructName;

pub(crate) trait StructName {
    const TYPE_NAME: &'static str;
}

pub(crate) trait SystemRepr {
    type Repr: JsCast;

    /// Get the base representation
    #[must_use]
    fn as_sys(&self) -> &Self::Repr;

    /// Convert into the base representation
    #[must_use]
    fn into_sys(self) -> Self::Repr;
}

pub(crate) fn slice_to_arr<I, T>(slice: I) -> js_sys::Array
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    fn jsval_from_asref<T: AsRef<str>>(v: T) -> JsValue {
        JsValue::from_str(v.as_ref())
    }

    slice.into_iter().map(jsval_from_asref).collect()
}
