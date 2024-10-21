use super::QuerySourceInternal;
use crate::future::BasicRequest;
use crate::internal_utils::SystemRepr;
use crate::primitive::TryToJs;
use crate::KeyRange;
use internal_macros::BuildIntoFut;
use sealed::sealed;

/// Builder for [`QuerySource::count`](super::QuerySource::count).
#[derive(BuildIntoFut)]
#[must_use]
pub struct Count<'q, Qs, Q = ()> {
    query_source: &'q Qs,
    query: Q,
}

impl<'q, Qs> Count<'q, Qs> {
    #[inline]
    pub(super) fn new<Sys>(query_source: &'q Qs) -> Self
    where
        Sys: QuerySourceInternal,
        Qs: SystemRepr<Repr = Sys>,
    {
        Self {
            query_source,
            query: (),
        }
    }
}

impl<'q, Qs, Q> Count<'q, Qs, Q> {
    /// Set the query.
    pub fn with_query<KR, I>(self, query: I) -> Count<'q, Qs, KeyRange<KR>>
    where
        I: Into<KeyRange<KR>>,
    {
        Count {
            query_source: self.query_source,
            query: query.into(),
        }
    }
}

#[sealed]
impl<Qs, Sys> crate::BuildPrimitive for Count<'_, Qs>
where
    Qs: SystemRepr<Repr = Sys>,
    Sys: QuerySourceInternal,
{
    type Fut = BasicRequest<u32>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let req = self.query_source.as_sys().count()?;
        Ok(BasicRequest::new_primitive(req))
    }
}

#[sealed]
impl<Qs, Sys, KR> crate::BuildPrimitive for Count<'_, Qs, KeyRange<KR>>
where
    Qs: SystemRepr<Repr = Sys>,
    Sys: QuerySourceInternal,
    KeyRange<KR>: TryToJs,
{
    type Fut = BasicRequest<u32>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let js = self.query.try_to_js()?;
        let req = self.query_source.as_sys().count_with_key(&js)?;
        Ok(BasicRequest::new_primitive(req))
    }
}

#[cfg(feature = "serde")]
const _: () = {
    use crate::{BuildPrimitive, SerialiseToJs};

    #[sealed]
    impl<Qs, Sys> crate::BuildSerde for Count<'_, Qs>
    where
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
    {
        type Fut = BasicRequest<u32>;

        #[inline]
        fn serde(self) -> crate::Result<Self::Fut> {
            Self::primitive(self)
        }
    }

    #[sealed]
    impl<Qs, Sys, KR> crate::BuildSerde for Count<'_, Qs, KeyRange<KR>>
    where
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        KeyRange<KR>: SerialiseToJs,
    {
        type Fut = BasicRequest<u32>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let js = self.query.serialise_to_js()?;
            let req = self.query_source.as_sys().count_with_key(&js)?;

            Ok(BasicRequest::new_primitive(req))
        }
    }
};
