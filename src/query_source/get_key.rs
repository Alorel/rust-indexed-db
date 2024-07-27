use super::QuerySourceInternal;
use crate::future::BasicRequest;
use crate::internal_utils::SystemRepr;
use crate::primitive::{TryFromJs, TryToJs};
use crate::KeyRange;
use fancy_constructor::new;
use internal_macros::BuildIntoFut;
use sealed::sealed;
use std::marker::PhantomData;

/// Builder for [`QuerySource::get_key`](super::QuerySource::get_key).
#[derive(new, BuildIntoFut)]
#[new(vis(pub(super)))]
#[must_use]
pub struct GetKey<'a, Qs, K, KT = K> {
    query_source: &'a Qs,
    key_range: KeyRange<K>,

    #[new(val(PhantomData))]
    _key_type: PhantomData<KT>,
}

impl<'a, Qs, K, KT> GetKey<'a, Qs, K, KT> {
    /// Set the key type returned by the query.
    #[inline]
    pub fn with_key_type<KT2>(self) -> GetKey<'a, Qs, K, KT2> {
        GetKey {
            query_source: self.query_source,
            key_range: self.key_range,
            _key_type: PhantomData,
        }
    }
}

#[sealed]
impl<Sys, Qs, K, KT> crate::BuildPrimitive for GetKey<'_, Qs, K, KT>
where
    Qs: SystemRepr<Repr = Sys>,
    Sys: QuerySourceInternal,
    KeyRange<K>: TryToJs,
    Option<KT>: TryFromJs,
{
    type Fut = BasicRequest<Option<KT>>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let key = self.key_range.try_into_js()?;
        let req = self.query_source.as_sys().get_key(&key)?;
        Ok(BasicRequest::new_primitive(req))
    }
}

#[cfg(feature = "serde")]
const _: () = {
    use crate::serde::{DeserialiseFromJs, SerialiseToJs};

    #[sealed]
    impl<Sys, Qs, K, KT> crate::BuildSerde for GetKey<'_, Qs, K, KT>
    where
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        KeyRange<K>: SerialiseToJs,
        Option<KT>: DeserialiseFromJs,
    {
        type Fut = BasicRequest<Option<KT>>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let key = self.key_range.serialise_to_js()?;
            let req = self.query_source.as_sys().get_key(&key)?;
            Ok(BasicRequest::new_ser(req))
        }
    }
};
