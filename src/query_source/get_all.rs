use super::QuerySourceInternal;
use crate::future::GetAllPrimitiveRequest;
use crate::internal_utils::SystemRepr;
use crate::primitive::{TryFromJs, TryToJs};
use crate::KeyRange;
use derive_more::Debug;
use internal_macros::BuildIntoFut;
use kind::GetAllKind;
use sealed::sealed;
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;

/// Builder for [`get_all`](super::QuerySource::get_all).
///
/// Retrieves all the keys/records in the index or object store if no [query](GetAll::with_query)
/// is provided.
pub type GetAllRecords<'a, Qs, T, Q = (), C = ()> = GetAll<'a, kind::Record, Qs, T, Q, C>;

/// Builder for [`get_all_keys`](super::QuerySource::get_all_keys).
///
/// Retrieves all the keys/records in the index or object store if no [query](GetAll::with_query)
/// is provided.
pub type GetAllKeys<'a, Qs, T, Q = (), C = ()> = GetAll<'a, kind::Key, Qs, T, Q, C>;

/// Builder for [`get_all`](super::QuerySource::get_all) &
/// [`get_all_keys`](super::QuerySource::get_all_keys).
///
/// Retrieves all the keys/records in the index or object store if no [query](GetAll::with_query)
/// is provided.
#[derive(Debug, BuildIntoFut)]
#[must_use]
pub struct GetAll<'a, K, Qs, T, Q = (), C = ()> {
    #[debug(skip)]
    query_source: &'a Qs,
    query: Q,
    limit: C,

    #[debug(skip)]
    marker: PhantomData<(K, T)>,
}

impl<'a, K, Qs, T> GetAll<'a, K, Qs, T> {
    #[inline]
    pub(super) fn new<Sys>(query_source: &'a Qs) -> Self
    where
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
    {
        Self {
            query_source,
            query: (),
            limit: (),
            marker: PhantomData,
        }
    }
}

impl<'a, K, Qs, T, Q, C> GetAll<'a, K, Qs, T, Q, C> {
    /// Set the key or key range to be queried.
    pub fn with_query<QK, I>(self, query: I) -> GetAll<'a, K, Qs, T, KeyRange<QK>, C>
    where
        I: Into<KeyRange<QK>>,
    {
        GetAll {
            query_source: self.query_source,
            query: query.into(),
            limit: self.limit,
            marker: PhantomData,
        }
    }

    /// Set the raw query to be used.
    pub fn with_raw_query(self, query: JsValue) -> GetAll<'a, K, Qs, T, JsValue, C> {
        GetAll {
            query_source: self.query_source,
            query,
            limit: self.limit,
            marker: PhantomData,
        }
    }

    /// Set the maximum number of results to return.
    #[inline]
    pub fn with_limit(self, limit: u32) -> GetAll<'a, K, Qs, T, Q, u32> {
        GetAll {
            query_source: self.query_source,
            query: self.query,
            limit,
            marker: PhantomData,
        }
    }
}

impl<K, Qs, T, Q, C> Clone for GetAll<'_, K, Qs, T, Q, C>
where
    Q: Clone,
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            query_source: self.query_source,
            query: self.query.clone(),
            limit: self.limit.clone(),
            marker: PhantomData,
        }
    }
}

// BuildPrimitive impls
const _: () = {
    #[sealed::sealed]
    impl<K, Qs, Sys, T> crate::BuildPrimitive for GetAll<'_, K, Qs, T>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: TryFromJs,
    {
        type Fut = GetAllPrimitiveRequest<T>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let req = K::get(self.query_source)?;
            Ok(GetAllPrimitiveRequest::get_all_primitive(req))
        }
    }

    #[sealed]
    impl<K, Qs, Sys, T, KR> crate::BuildPrimitive for GetAll<'_, K, Qs, T, KeyRange<KR>>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: TryFromJs,
        KeyRange<KR>: TryToJs,
    {
        type Fut = GetAllPrimitiveRequest<T>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let js = self.query.try_to_js()?;
            let req = K::get_with_key(self.query_source, js)?;
            Ok(GetAllPrimitiveRequest::get_all_primitive(req))
        }
    }

    #[sealed]
    impl<K, Qs, Sys, T> crate::BuildPrimitive for GetAll<'_, K, Qs, T, (), u32>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: TryFromJs,
    {
        type Fut = GetAllPrimitiveRequest<T>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let req = K::get_with_key_limit(self.query_source, JsValue::UNDEFINED, self.limit)?;
            Ok(GetAllPrimitiveRequest::get_all_primitive(req))
        }
    }

    #[sealed]
    impl<K, Qs, Sys, T, KR> crate::BuildPrimitive for GetAll<'_, K, Qs, T, KeyRange<KR>, u32>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: TryFromJs,
        KeyRange<KR>: TryToJs,
    {
        type Fut = GetAllPrimitiveRequest<T>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let js = self.query.try_to_js()?;
            let req = K::get_with_key_limit(self.query_source, js, self.limit)?;
            Ok(GetAllPrimitiveRequest::get_all_primitive(req))
        }
    }
};

#[cfg(feature = "serde")]
const _: () = {
    use crate::future::GetAllSerdeRequest;
    use crate::serde::{DeserialiseFromJs, SerialiseToJs};

    #[sealed]
    impl<K, Qs, Sys, T> crate::BuildSerde for GetAll<'_, K, Qs, T, JsValue>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: DeserialiseFromJs,
    {
        type Fut = GetAllSerdeRequest<T>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let req = K::get_with_key(self.query_source, self.query)?;
            Ok(GetAllSerdeRequest::get_all_serde(req))
        }
    }

    #[sealed]
    impl<K, Qs, Sys, T> crate::BuildSerde for GetAll<'_, K, Qs, T>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: DeserialiseFromJs,
    {
        type Fut = GetAllSerdeRequest<T>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let req = K::get(self.query_source)?;
            Ok(GetAllSerdeRequest::get_all_serde(req))
        }
    }

    #[sealed]
    impl<K, Qs, Sys, T, KR> crate::BuildSerde for GetAll<'_, K, Qs, T, KeyRange<KR>>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: DeserialiseFromJs,
        KeyRange<KR>: SerialiseToJs,
    {
        type Fut = GetAllSerdeRequest<T>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let js = self.query.serialise_to_js()?;
            let req = K::get_with_key(self.query_source, js)?;
            Ok(GetAllSerdeRequest::get_all_serde(req))
        }
    }

    #[sealed]
    impl<K, Qs, Sys, T> crate::BuildSerde for GetAll<'_, K, Qs, T, (), u32>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: DeserialiseFromJs,
    {
        type Fut = GetAllSerdeRequest<T>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let req = K::get_with_key_limit(self.query_source, JsValue::UNDEFINED, self.limit)?;
            Ok(GetAllSerdeRequest::get_all_serde(req))
        }
    }

    #[sealed]
    impl<K, Qs, Sys, T, KR> crate::BuildSerde for GetAll<'_, K, Qs, T, KeyRange<KR>, u32>
    where
        K: GetAllKind,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        T: DeserialiseFromJs,
        KeyRange<KR>: SerialiseToJs,
    {
        type Fut = GetAllSerdeRequest<T>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let js = self.query.serialise_to_js()?;
            let req = K::get_with_key_limit(self.query_source, js, self.limit)?;
            Ok(GetAllSerdeRequest::get_all_serde(req))
        }
    }
};

pub(crate) mod kind {
    use super::super::QuerySourceInternal;
    use crate::internal_utils::SystemRepr;
    use sealed::sealed;
    use wasm_bindgen::prelude::*;

    #[allow(missing_docs)]
    pub struct Record();

    #[allow(missing_docs)]
    pub struct Key();

    /// Type of [`get_all`](super::super::QuerySource::get_all) operation.
    #[sealed]
    pub trait GetAllKind {
        #[doc(hidden)]
        fn get<Qs, Sys>(qs: &Qs) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal;

        #[doc(hidden)]
        fn get_with_key<Qs, Sys>(qs: &Qs, key: JsValue) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal;

        #[doc(hidden)]
        fn get_with_key_limit<Qs, Sys>(
            qs: &Qs,
            key: JsValue,
            limit: u32,
        ) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal;
    }

    #[sealed]
    impl GetAllKind for Record {
        #[inline]
        fn get<Qs, Sys>(qs: &Qs) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().get_all()
        }

        #[inline]
        fn get_with_key<Qs, Sys>(qs: &Qs, key: JsValue) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().get_all_with_key(&key)
        }

        #[inline]
        fn get_with_key_limit<Qs, Sys>(
            qs: &Qs,
            key: JsValue,
            limit: u32,
        ) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().get_all_with_key_and_limit(&key, limit)
        }
    }

    #[sealed]
    impl GetAllKind for Key {
        fn get<Qs, Sys>(qs: &Qs) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().get_all_keys()
        }

        fn get_with_key<Qs, Sys>(qs: &Qs, key: JsValue) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().get_all_keys_with_key(&key)
        }

        fn get_with_key_limit<Qs, Sys>(
            qs: &Qs,
            key: JsValue,
            limit: u32,
        ) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().get_all_keys_with_key_and_limit(&key, limit)
        }
    }
}
