use sealed::sealed;
use wasm_bindgen::prelude::*;

use internal_macros::errdoc;

use crate::future::{CountRequest, MaybeErrored, PrimitiveRequest};
use crate::internal_utils::SystemRepr;
use crate::{FromJs, KeyPath, KeyRange, TryToJs};

iffeat! {
    #[cfg(feature = "serde")]
    use {
        crate::future::SerdeRequest,
        serde::de::DeserializeOwned,
        internal_macros::serrdoc,
    };
}

#[sealed]
pub(crate) trait QuerySourceInternal {
    fn name(&self) -> String;
    fn set_name(&self, name: &str);
    fn count(&self) -> Result<web_sys::IdbRequest, JsValue>;
    fn count_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;
    fn key_path(&self) -> Result<JsValue, JsValue>;
    fn get(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue>;
    fn get_all(&self) -> Result<web_sys::IdbRequest, JsValue>;
}

/// Common functionality for making queries
#[sealed]
pub trait QuerySource {
    /// Count the number of documents in the index/object store.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError))]
    fn count(&self) -> MaybeErrored<CountRequest>;

    /// Count the number of documents in the index/object store that match the given key range.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, DataError))]
    fn count_with_key<K: TryToJs>(&self, key: KeyRange<K>) -> MaybeErrored<CountRequest>;

    /// Get one JS primitive value. Returns the first match if a non-[only](KeyRange::Only) key is provided and multiple
    /// records match.
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, DataError))]
    fn get_primitive<V, K>(&self, key: KeyRange<K>) -> MaybeErrored<PrimitiveRequest<Option<V>>>
    where
        V: FromJs,
        K: TryToJs;

    /// Get the index/object store key path.
    /// Returns `None` if the index isn't auto-populated.
    fn key_path(&self) -> Option<KeyPath<'static>>;

    /// Get the index/object store name
    fn name(&self) -> String;

    /// Set the index/object store name
    #[errdoc(QuerySource(InvalidStateError, TransactionInactiveError, ConstraintError))]
    fn set_name(&self, name: &str);

    iffeat! {
        #[cfg(feature = "serde")]
        /// Get one [deserialisable](serde::Deserialize) value.
        /// # Errors
        /// See [`get_primitive`](QuerySource::get_primitive).
        #[serrdoc]
        fn get<V, K>(&self, key: KeyRange<K>) -> MaybeErrored<SerdeRequest<V>>
        where
            V: DeserializeOwned,
            K: TryToJs;
    }
}

#[sealed]
impl<T: SystemRepr<Repr = R>, R: QuerySourceInternal> QuerySource for T {
    #[inline]
    fn name(&self) -> String {
        self.as_sys().name()
    }

    #[inline]
    fn set_name(&self, name: &str) {
        self.as_sys().set_name(name);
    }

    fn count(&self) -> MaybeErrored<CountRequest> {
        maybe_errored_dom!(self.as_sys().count(), |req| CountRequest::new(req))
    }

    fn count_with_key<K: TryToJs>(&self, key: KeyRange<K>) -> MaybeErrored<CountRequest> {
        match key.try_to_js() {
            Ok(js) => {
                maybe_errored_dom!(self.as_sys().count_with_key(&js), |req| CountRequest::new(
                    req
                ))
            }
            Err(e) => MaybeErrored::errored(e),
        }
    }

    fn key_path(&self) -> Option<KeyPath<'static>> {
        match self.as_sys().key_path() {
            Ok(path) if !path.is_null() => Some(path.into()),
            _ => None,
        }
    }

    fn get_primitive<V, K>(&self, key: KeyRange<K>) -> MaybeErrored<PrimitiveRequest<Option<V>>>
    where
        V: FromJs,
        K: TryToJs,
    {
        match key.try_to_js() {
            Ok(js) => maybe_errored_dom!(self.as_sys().get(&js), |req| PrimitiveRequest::new(req)),
            Err(e) => MaybeErrored::errored(e),
        }
    }

    iffeat! {
        #[cfg(feature = "serde")]
        fn get<V, K>(&self, key: KeyRange<K>) -> MaybeErrored<SerdeRequest<V>>
        where
            V: DeserializeOwned,
            K: TryToJs,
        {
            match key.try_to_js() {
                Ok(js) => maybe_errored_dom!(self.as_sys().get(&js), |req| SerdeRequest::new(req)),
                Err(e) => MaybeErrored::errored(e),
            }
        }
    }
}

macro_rules! impl_internal {
    ($for: ty) => {
        #[::sealed::sealed]
        impl QuerySourceInternal for $for {
            #[inline]
            fn name(&self) -> String {
                <$for>::name(self)
            }

            #[inline]
            fn set_name(&self, name: &str) {
                <$for>::set_name(self, name);
            }

            #[inline]
            fn count(&self) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::count(self)
            }

            #[inline]
            fn count_with_key(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::count_with_key(self, key)
            }

            #[inline]
            fn key_path(&self) -> Result<JsValue, JsValue> {
                <$for>::key_path(self)
            }

            #[inline]
            fn get(&self, key: &JsValue) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get(self, key)
            }

            #[inline]
            fn get_all(&self) -> Result<web_sys::IdbRequest, JsValue> {
                <$for>::get_all(self)
            }
        }
    };
}

impl_internal!(web_sys::IdbObjectStore);

#[cfg(feature = "indices")]
impl_internal!(web_sys::IdbIndex);
