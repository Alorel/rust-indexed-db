use sealed::sealed;
use wasm_bindgen::prelude::*;

pub use internal_macros::StructName;

/// Marker type equivalent to `()` that's neither [`TryToJs`](crate::primitive::TryToJs) nor
/// [`TryFromJs`](crate::primitive::TryFromJs) nor [`Serialize`](serde::Serialize) nor
/// [`Deserialize`](serde::Deserialize).
#[derive(Copy, Clone, Debug)]
pub struct Void();
impl Void {
    pub(crate) const VOID: Void = Void {};
}

pub(crate) trait StructName {
    const TYPE_NAME: &'static str;
}

/// Internal [`web_sys`]-based representation.
#[sealed(pub(crate))]
pub trait SystemRepr {
    /// Internal [`web_sys`]-based representation type.
    type Repr: JsCast;

    /// Get the base representation. Using this fn to bypass the library can lead to unaccounted for errors.
    #[must_use]
    #[doc(hidden)]
    fn as_sys(&self) -> &Self::Repr;

    /// Convert into the base representation. Using this fn to bypass the library can lead to unaccounted for errors.
    #[must_use]
    #[doc(hidden)]
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
