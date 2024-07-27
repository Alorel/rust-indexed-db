use super::QuerySourceInternal;
use crate::future::BasicRequest;
use crate::internal_utils::SystemRepr;
use crate::primitive::{TryFromJs, TryToJs};
use crate::KeyRange;
use fancy_constructor::new;
use internal_macros::BuildIntoFut;
use sealed::sealed;
use std::marker::PhantomData;

/// Builder for [`QuerySource::get`](super::QuerySource::get).
#[derive(new, BuildIntoFut)]
#[new(vis(pub(super)))]
#[must_use]
pub struct Get<'a, Qs, K, V> {
    query_source: &'a Qs,
    key: KeyRange<K>,

    #[new(val(PhantomData))]
    t_value: PhantomData<V>,
}

#[sealed]
impl<Qs, K, V, Sys> crate::BuildPrimitive for Get<'_, Qs, K, V>
where
    Qs: SystemRepr<Repr = Sys>,
    KeyRange<K>: TryToJs,
    Option<V>: TryFromJs,
    Sys: QuerySourceInternal,
{
    type Fut = BasicRequest<Option<V>>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let Self {
            query_source,
            key,
            t_value: _,
        } = self;

        let js = key.try_to_js()?;
        let req = query_source.as_sys().get(&js)?;

        Ok(BasicRequest::new_primitive(req))
    }
}

#[sealed]
#[cfg(feature = "serde")]
impl<Q, K, V, Sys> crate::BuildSerde for Get<'_, Q, K, V>
where
    Q: SystemRepr<Repr = Sys>,
    KeyRange<K>: crate::serde::SerialiseToJs,
    Option<V>: crate::serde::DeserialiseFromJs,
    Sys: QuerySourceInternal,
{
    type Fut = BasicRequest<Option<V>>;

    fn serde(self) -> crate::Result<Self::Fut> {
        let Self {
            query_source,
            key,
            t_value: _,
        } = self;

        let js = crate::serde::SerialiseToJs::serialise_to_js(&key)?;
        let req = query_source.as_sys().get(&js)?;

        Ok(BasicRequest::new_ser(req))
    }
}
