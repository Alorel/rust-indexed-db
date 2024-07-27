use super::QuerySourceInternal;
use crate::cursor::CursorDirection;
use crate::future::{CursorRequest, PollUnpinned};
use crate::internal_utils::SystemRepr;
use crate::primitive::TryToJs;
use crate::KeyRange;
use derive_more::Debug;
use internal_macros::BuildIntoFut;
use kind::CursorKind;
use sealed::sealed;
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;

/// Builder for a [`Cursor`](crate::cursor::Cursor).
pub type CursorBuilder<'a, Qs, Q = (), D = ()> = AnyCursorBuilder<'a, kind::Record, Qs, Q, D>;

/// Builder for a [`KeyCursor`](crate::cursor::KeyCursor).
pub type KeyCursorBuilder<'a, Qs, Q = (), D = ()> = AnyCursorBuilder<'a, kind::Key, Qs, Q, D>;

/// Builder for a [`Cursor`](crate::cursor::Cursor) & [`KeyCursor`](crate::cursor::KeyCursor).
#[derive(Debug, BuildIntoFut)]
#[must_use]
pub struct AnyCursorBuilder<'qs, K, Qs, Q = (), D = ()> {
    #[debug(skip)]
    query_source: &'qs Qs,
    query: Q,
    direction: D,

    #[debug(skip)]
    kind: PhantomData<K>,
}

impl<'qs, K, Qs> AnyCursorBuilder<'qs, K, Qs> {
    #[inline]
    pub(super) fn new<Sys>(query_source: &'qs Qs) -> Self
    where
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
    {
        Self {
            query_source,
            query: (),
            direction: (),
            kind: PhantomData,
        }
    }
}

impl<'qs, K, Qs, Q, D> AnyCursorBuilder<'qs, K, Qs, Q, D> {
    /// Set the key or key range to be queried.
    pub fn with_query<QK, I>(self, query: I) -> AnyCursorBuilder<'qs, K, Qs, KeyRange<QK>, D>
    where
        I: Into<KeyRange<QK>>,
    {
        AnyCursorBuilder {
            query_source: self.query_source,
            query: query.into(),
            direction: self.direction,
            kind: PhantomData,
        }
    }

    /// Set the direction of the cursor.
    #[inline]
    pub fn with_direction(
        self,
        direction: CursorDirection,
    ) -> AnyCursorBuilder<'qs, K, Qs, Q, CursorDirection> {
        AnyCursorBuilder {
            query_source: self.query_source,
            query: self.query,
            direction,
            kind: PhantomData,
        }
    }
}

impl<K, Qs, Q, D> Clone for AnyCursorBuilder<'_, K, Qs, Q, D>
where
    Q: Clone,
    D: Clone,
{
    fn clone(&self) -> Self {
        Self {
            query_source: self.query_source,
            query: self.query.clone(),
            direction: self.direction.clone(),
            kind: PhantomData,
        }
    }
}

// Build impls
const _: () = {
    #[sealed]
    impl<'qs, Sys, K, Qs, Q> crate::BuildPrimitive
        for AnyCursorBuilder<'qs, K, Qs, KeyRange<Q>, CursorDirection>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        KeyRange<Q>: TryToJs,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let query = self.query.try_to_js()?;
            let req = K::open_w_range_n_direction(self.query_source, query, self.direction)?;
            Ok(CursorRequest::new(req, self.query_source))
        }
    }

    #[sealed]
    impl<'qs, Sys, K, Qs, Q> crate::BuildPrimitive for AnyCursorBuilder<'qs, K, Qs, KeyRange<Q>>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        KeyRange<Q>: TryToJs,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let query = self.query.try_to_js()?;
            let req = K::open_w_range(self.query_source, query)?;
            Ok(CursorRequest::new(req, self.query_source))
        }
    }

    #[sealed]
    impl<'qs, Sys, K, Qs> crate::BuildPrimitive for AnyCursorBuilder<'qs, K, Qs, (), CursorDirection>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let req =
                K::open_w_range_n_direction(self.query_source, JsValue::UNDEFINED, self.direction)?;
            Ok(CursorRequest::new(req, self.query_source))
        }
    }

    #[sealed]
    impl<'qs, Sys, K, Qs> crate::BuildPrimitive for AnyCursorBuilder<'qs, K, Qs>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        fn primitive(self) -> crate::Result<Self::Fut> {
            let req = K::open(self.query_source)?;
            Ok(CursorRequest::new(req, self.query_source))
        }
    }
};

#[cfg(feature = "serde")]
const _: () = {
    use crate::serde::SerialiseToJs;

    #[sealed]
    impl<'qs, Sys, K, Qs, Q> crate::BuildSerde
        for AnyCursorBuilder<'qs, K, Qs, KeyRange<Q>, CursorDirection>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        KeyRange<Q>: SerialiseToJs,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let query = self.query.serialise_to_js()?;
            let req = K::open_w_range_n_direction(self.query_source, query, self.direction)?;
            Ok(CursorRequest::new(req, self.query_source))
        }
    }

    #[sealed]
    impl<'qs, Sys, K, Qs, Q> crate::BuildSerde for AnyCursorBuilder<'qs, K, Qs, KeyRange<Q>>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
        KeyRange<Q>: SerialiseToJs,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let query = self.query.serialise_to_js()?;
            let req = K::open_w_range(self.query_source, query)?;
            Ok(CursorRequest::new(req, self.query_source))
        }
    }

    #[sealed]
    impl<'qs, Sys, K, Qs> crate::BuildSerde for AnyCursorBuilder<'qs, K, Qs, (), CursorDirection>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        #[inline]
        fn serde(self) -> crate::Result<Self::Fut> {
            crate::BuildPrimitive::primitive(self)
        }
    }

    #[sealed]
    impl<'qs, Sys, K, Qs> crate::BuildSerde for AnyCursorBuilder<'qs, K, Qs>
    where
        K: CursorKind,
        CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>: PollUnpinned + Unpin,
        Qs: SystemRepr<Repr = Sys>,
        Sys: QuerySourceInternal,
    {
        type Fut = CursorRequest<'qs, K::Cursor<'qs, Qs>, Qs>;

        #[inline]
        fn serde(self) -> crate::Result<Self::Fut> {
            crate::BuildPrimitive::primitive(self)
        }
    }
};

pub(crate) mod kind {
    #[allow(missing_docs)]
    pub struct Record();
    #[allow(missing_docs)]
    pub struct Key();

    use super::super::QuerySourceInternal;
    use crate::cursor::CursorDirection;
    use crate::internal_utils::SystemRepr;
    use sealed::sealed;
    use wasm_bindgen::prelude::*;

    /// The type of cursor being built.
    #[sealed]
    pub trait CursorKind {
        /// The cursor returned by opening with this type.
        type Cursor<'a, Qs: 'a>;

        #[doc(hidden)]
        fn open<Qs, Sys>(qs: &Qs) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal;

        #[doc(hidden)]
        fn open_w_range<Qs, Sys>(qs: &Qs, query: JsValue) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal;

        #[doc(hidden)]
        fn open_w_range_n_direction<Qs, Sys>(
            qs: &Qs,
            query: JsValue,
            direction: CursorDirection,
        ) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal;
    }

    #[sealed]
    impl CursorKind for Record {
        type Cursor<'a, Qs: 'a> = crate::cursor::Cursor<'a, Qs>;

        #[inline]
        fn open<Qs, Sys>(qs: &Qs) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().open_cursor()
        }

        #[inline]
        fn open_w_range<Qs, Sys>(qs: &Qs, query: JsValue) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().open_cursor_with_range(&query)
        }

        #[inline]
        fn open_w_range_n_direction<Qs, Sys>(
            qs: &Qs,
            query: JsValue,
            direction: CursorDirection,
        ) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys()
                .open_cursor_with_range_and_direction(&query, direction)
        }
    }

    #[sealed]
    impl CursorKind for Key {
        type Cursor<'a, Qs: 'a> = crate::cursor::KeyCursor<'a, Qs>;

        #[inline]
        fn open<Qs, Sys>(qs: &Qs) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().open_key_cursor()
        }

        #[inline]
        fn open_w_range<Qs, Sys>(qs: &Qs, query: JsValue) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys().open_key_cursor_with_range(&query)
        }

        #[inline]
        fn open_w_range_n_direction<Qs, Sys>(
            qs: &Qs,
            query: JsValue,
            direction: CursorDirection,
        ) -> Result<web_sys::IdbRequest, JsValue>
        where
            Qs: SystemRepr<Repr = Sys>,
            Sys: QuerySourceInternal,
        {
            qs.as_sys()
                .open_key_cursor_with_range_and_direction(&query, direction)
        }
    }
}
