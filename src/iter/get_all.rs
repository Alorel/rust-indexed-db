use super::ArrayMapIter;
use crate::primitive::TryFromJs;
use wasm_bindgen::prelude::*;

iffeat! {
    #[cfg(feature = "serde")]
    use crate::serde::DeserialiseFromJs;

    /// An iterator over [deserialisable](serde::Deserialize) array results.
    ///
    /// Created through [`QuerySource::get_all`](crate::query_source::QuerySource::get_all) and
    /// [`GetAll`](crate::query_source::GetAllRecords).
    #[cfg(feature = "serde")]
    pub type GetAllSerdeIter<V> = ArrayMapIter<V>;
}

/// An iterator over the primitive array results.
///
/// Created through [`QuerySource::get_all`](crate::query_source::QuerySource::get_all) and
/// [`GetAll`](crate::query_source::GetAllRecords).
pub type GetAllPrimitiveIter<V> = ArrayMapIter<V>;

impl<V> GetAllPrimitiveIter<V> {
    /// [`new`](ArrayMapIter::new) alias for [`GetAllPrimitiveIter`].
    pub(crate) fn get_all_primitive(array: js_sys::Array) -> Self
    where
        V: TryFromJs,
    {
        fn map_primitive<V: TryFromJs>(value: JsValue) -> crate::Result<V> {
            V::from_js(value).map_err(Into::into)
        }

        Self::new(array.into_iter(), map_primitive::<V>)
    }

    #[cfg(feature = "serde")]
    pub(crate) fn get_all_serde(array: js_sys::Array) -> Self
    where
        V: DeserialiseFromJs,
    {
        Self::new(array.into_iter(), V::deserialise_from_js)
    }
}
