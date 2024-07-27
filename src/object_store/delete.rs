use super::ObjectStore;
use crate::future::VoidRequest;
use crate::internal_utils::SystemRepr;
use crate::primitive::TryToJs;
use crate::KeyRange;
use fancy_constructor::new;
use internal_macros::BuildIntoFut;
use sealed::sealed;
use wasm_bindgen::prelude::*;

/// Builder for [`ObjectStore::delete`].
#[derive(BuildIntoFut, new)]
#[new(vis(pub(super)))]
#[must_use]
pub struct Delete<'a, K> {
    store: &'a ObjectStore<'a>,
    key: KeyRange<K>,
}

impl<K> Delete<'_, K> {
    fn into_req(self, key: &JsValue) -> crate::Result<VoidRequest> {
        let req = self.store.as_sys().delete(key)?;
        Ok(VoidRequest::new(req))
    }
}

#[sealed]
impl<K> crate::BuildPrimitive for Delete<'_, K>
where
    KeyRange<K>: TryToJs,
{
    type Fut = VoidRequest;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let key = self.key.try_to_js()?;
        self.into_req(&key)
    }
}

#[cfg(feature = "serde")]
#[sealed]
impl<K> crate::BuildSerde for Delete<'_, K>
where
    KeyRange<K>: crate::serde::SerialiseToJs,
{
    type Fut = VoidRequest;

    fn serde(self) -> crate::Result<Self::Fut> {
        let key = crate::serde::SerialiseToJs::serialise_to_js(&self.key)?;
        self.into_req(&key)
    }
}
