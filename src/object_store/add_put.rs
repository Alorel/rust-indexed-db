use super::ObjectStore;
use crate::future::{BasicRequest, VoidRequest};
use crate::internal_utils::Void;
use crate::primitive::{TryFromJs, TryToJs};
use derive_more::Debug;
use internal_macros::BuildIntoFut;
use kind::InsertKind;
use sealed::sealed;
use std::marker::PhantomData;

/// Builder for [`ObjectStore::add`]
pub type Add<'a, T, K = Void, KT = Void> = AddPut<'a, kind::Add, T, K, KT>;

/// Builder for [`ObjectStore::put`].
pub type Put<'a, T, K = Void, KT = Void> = AddPut<'a, kind::Put, T, K, KT>;

/// Builder for [`ObjectStore::add`] & [`ObjectStore::put`].
#[derive(Debug, BuildIntoFut)]
#[must_use]
pub struct AddPut<'a, AP, T, K = Void, KT = Void> {
    #[debug(skip)]
    object_store: &'a ObjectStore<'a>,
    value: T,
    key: K,

    #[debug(skip)]
    marker: PhantomData<(AP, KT)>,
}

impl<'a, AP: InsertKind, T> AddPut<'a, AP, T> {
    #[inline]
    pub(super) fn new(object_store: &'a ObjectStore<'a>, value: T) -> Self {
        Self {
            object_store,
            value,
            key: Void::VOID,
            marker: PhantomData,
        }
    }
}

impl<'a, AP, T, K, KT> AddPut<'a, AP, T, K, KT> {
    /// Set the key to use to identify the record.
    #[inline]
    pub fn with_key<K2>(self, key: K2) -> AddPut<'a, AP, T, K2, K2> {
        AddPut {
            object_store: self.object_store,
            value: self.value,
            key,
            marker: PhantomData,
        }
    }

    /// Set the type of the key to be returned if different from that set by [`with_key`](Self::with_key), or
    /// if that method is inapplicable for this operation.
    #[inline]
    pub fn with_key_type<KT2>(self) -> AddPut<'a, AP, T, K, KT2> {
        AddPut {
            object_store: self.object_store,
            value: self.value,
            key: self.key,
            marker: PhantomData,
        }
    }

    /// Unset the key type set by [`with_key`](Self::with_key) or [`with_key_type`](Self::with_key_type).
    #[inline]
    pub fn without_key_type(self) -> AddPut<'a, AP, T, K, Void> {
        AddPut {
            object_store: self.object_store,
            value: self.value,
            key: self.key,
            marker: PhantomData,
        }
    }

    fn jsify_key_value(&self) -> crate::Result<[::wasm_bindgen::JsValue; 2]>
    where
        T: TryToJs,
        K: TryToJs,
    {
        let key = self.key.try_to_js()?;
        let value = self.value.try_to_js()?;
        Ok([key, value])
    }
}

#[sealed]
impl<AP, T> crate::BuildPrimitive for AddPut<'_, AP, T>
where
    AP: InsertKind,
    T: TryToJs,
{
    type Fut = VoidRequest;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let Self {
            object_store,
            value,
            key: _,
            marker: _,
        } = self;

        let value = value.try_to_js()?;
        let req = AP::add(object_store, value)?;

        Ok(VoidRequest::new(req))
    }
}

#[sealed]
impl<AP, T, KT> crate::BuildPrimitive for AddPut<'_, AP, T, Void, KT>
where
    AP: InsertKind,
    T: TryToJs,
    KT: TryFromJs,
{
    type Fut = BasicRequest<KT>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let Self {
            object_store,
            value,
            key: _,
            marker: _,
        } = self;

        let value = value.try_to_js()?;
        let req = AP::add(object_store, value)?;

        Ok(BasicRequest::new_primitive(req))
    }
}

#[sealed]
impl<AP, T, K> crate::BuildPrimitive for AddPut<'_, AP, T, K>
where
    AP: InsertKind,
    T: TryToJs,
    K: TryToJs,
{
    type Fut = VoidRequest;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let [key, value] = self.jsify_key_value()?;
        let req = AP::add_with_key(self.object_store, key, value)?;

        Ok(VoidRequest::new(req))
    }
}

#[sealed]
impl<AP, T, K, KT> crate::BuildPrimitive for AddPut<'_, AP, T, K, KT>
where
    AP: InsertKind,
    T: TryToJs,
    K: TryToJs,
    KT: TryFromJs,
{
    type Fut = BasicRequest<KT>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let [key, value] = self.jsify_key_value()?;
        let req = AP::add_with_key(self.object_store, key, value)?;

        Ok(BasicRequest::new_primitive(req))
    }
}

/// Special handling for `&str` - UX for devs.
#[sealed]
impl<AP, T> crate::BuildPrimitive for AddPut<'_, AP, T, &str, &str>
where
    AP: InsertKind,
    T: TryToJs,
{
    type Fut = BasicRequest<String>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        self.with_key_type::<String>().primitive()
    }
}

#[cfg(feature = "serde")]
const _: () = {
    use crate::serde::{DeserialiseFromJs, SerialiseToJs};

    impl<AP, T, K, KT> AddPut<'_, AP, T, K, KT>
    where
        T: SerialiseToJs,
        K: SerialiseToJs,
    {
        fn serialise_key_value(&self) -> crate::Result<[::wasm_bindgen::JsValue; 2]> {
            Ok([self.key.serialise_to_js()?, self.value.serialise_to_js()?])
        }
    }

    #[sealed]
    impl<AP, T> crate::BuildSerde for AddPut<'_, AP, T>
    where
        AP: InsertKind,
        T: SerialiseToJs,
    {
        type Fut = VoidRequest;

        fn serde(self) -> crate::Result<Self::Fut> {
            let Self {
                object_store,
                value,
                key: _,
                marker: _,
            } = self;

            let value = value.serialise_to_js()?;
            let req = AP::add(object_store, value)?;

            Ok(VoidRequest::new(req))
        }
    }

    #[sealed]
    impl<AP, T, KT> crate::BuildSerde for AddPut<'_, AP, T, Void, KT>
    where
        AP: InsertKind,
        T: SerialiseToJs,
        KT: DeserialiseFromJs,
    {
        type Fut = BasicRequest<KT>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let Self {
                object_store,
                value,
                key: _,
                marker: _,
            } = self;

            let value = value.serialise_to_js()?;
            let req = AP::add(object_store, value)?;

            Ok(BasicRequest::new_ser(req))
        }
    }

    #[sealed]
    impl<AP, T, K> crate::BuildSerde for AddPut<'_, AP, T, K>
    where
        AP: InsertKind,
        T: SerialiseToJs,
        K: SerialiseToJs,
    {
        type Fut = VoidRequest;

        fn serde(self) -> crate::Result<Self::Fut> {
            let [key, value] = self.serialise_key_value()?;
            let req = AP::add_with_key(self.object_store, key, value)?;

            Ok(VoidRequest::new(req))
        }
    }

    #[sealed]
    impl<AP, T, K, KT> crate::BuildSerde for AddPut<'_, AP, T, K, KT>
    where
        AP: InsertKind,
        T: SerialiseToJs,
        K: SerialiseToJs,
        KT: DeserialiseFromJs,
    {
        type Fut = BasicRequest<KT>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let [key, value] = self.serialise_key_value()?;
            let req = AP::add_with_key(self.object_store, key, value)?;

            Ok(BasicRequest::new_ser(req))
        }
    }
};

pub(crate) mod kind {
    use super::super::ObjectStore;
    use crate::internal_utils::SystemRepr;
    use sealed::sealed;
    use wasm_bindgen::prelude::*;

    #[allow(missing_docs)]
    pub struct Add();

    #[allow(missing_docs)]
    pub struct Put();

    /// The type of record insertion operation.
    #[sealed]
    pub trait InsertKind {
        #[doc(hidden)]
        fn add(store: &ObjectStore<'_>, value: JsValue) -> Result<web_sys::IdbRequest, JsValue>;

        #[doc(hidden)]
        fn add_with_key(
            store: &ObjectStore<'_>,
            key: JsValue,
            value: JsValue,
        ) -> Result<web_sys::IdbRequest, JsValue>;
    }

    #[sealed]
    impl InsertKind for Add {
        #[inline]
        fn add(store: &ObjectStore<'_>, value: JsValue) -> Result<web_sys::IdbRequest, JsValue> {
            store.as_sys().add(&value)
        }

        #[inline]
        fn add_with_key(
            store: &ObjectStore<'_>,
            key: JsValue,
            value: JsValue,
        ) -> Result<web_sys::IdbRequest, JsValue> {
            store.as_sys().add_with_key(&value, &key)
        }
    }

    #[sealed]
    impl InsertKind for Put {
        #[inline]
        fn add(store: &ObjectStore<'_>, value: JsValue) -> Result<web_sys::IdbRequest, JsValue> {
            store.as_sys().put(&value)
        }

        #[inline]
        fn add_with_key(
            store: &ObjectStore<'_>,
            key: JsValue,
            value: JsValue,
        ) -> Result<web_sys::IdbRequest, JsValue> {
            store.as_sys().put_with_key(&value, &key)
        }
    }
}
