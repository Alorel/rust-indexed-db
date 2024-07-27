use std::borrow::Cow;
use std::future::IntoFuture;
use std::marker::PhantomData;

use internal_macros::{callback_bounds, StructName};

use crate::error::{Error, OpenDbOpError};
use crate::future::{MaybeErrored, OpenDbRequest};
use crate::Database;

use super::{DBFactory, DBVersion};

type OpenDbOpResult<B = Error, U = Error> = crate::OpenDbOpResult<Database, B, U>;
type IntoFut<B = Error, U = Error> = MaybeErrored<OpenDbRequest<B, U>, OpenDbOpError<B, U>>;

type Factoried<'a, V = (), B = (), U = (), FB = (), FU = ()> =
    OpenDbRequestBuilder<'a, V, B, U, FB, FU, DBFactory>;

/// Database open request builder. Does nothing until [turned into a future](IntoFuture).
///
/// # Generics
///
/// | Generic param | Description |
/// |---|---|
/// | `'a` | Name lifetime |
/// | `V` | Database version |
/// | `B` | `blocked` event handler. Tied to `FB`. |
/// | `U` | `upgradeneeded` event handler. Tied to `FU`. |
/// | `FB` | `blocked` future. Tied to `B`. |
/// | `FU` | `upgradeneeded` future. Tied to `U`. |
/// | `Fa` | Either [`DBFactory`] or `()` if a factory should be generated on the fly |
#[derive(StructName)]
#[must_use]
pub struct OpenDbRequestBuilder<'a, V = (), B = (), U = (), FB = (), FU = (), Fa = ()> {
    name: Cow<'a, str>,
    version: V,
    on_blocked: B,
    on_upgrade_needed: U,
    factory: Fa,
    future_types: PhantomData<(FB, FU)>,
}

impl<'a> OpenDbRequestBuilder<'a> {
    /// Open a database with the given name.
    pub fn new<T: Into<Cow<'a, str>>>(name: T) -> Self {
        Self {
            name: name.into(),
            version: (),
            on_blocked: (),
            on_upgrade_needed: (),
            factory: (),
            future_types: PhantomData,
        }
    }
}

impl<'a, V, B, U, FB, FU, Fa> OpenDbRequestBuilder<'a, V, B, U, FB, FU, Fa> {
    /// Set the name of the database being opened.
    pub fn with_name<'a2, N>(self, name: N) -> OpenDbRequestBuilder<'a2, V, B, U, FB, FU, Fa>
    where
        N: Into<Cow<'a2, str>>,
    {
        OpenDbRequestBuilder {
            name: name.into(),
            version: self.version,
            on_blocked: self.on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory: self.factory,
            future_types: PhantomData,
        }
    }

    /// Set the version of the database being opened.
    pub fn with_version<V2>(self, version: V2) -> OpenDbRequestBuilder<'a, V2, B, U, FB, FU, Fa>
    where
        V2: DBVersion,
    {
        OpenDbRequestBuilder {
            name: self.name,
            version,
            on_blocked: self.on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory: self.factory,
            future_types: PhantomData,
        }
    }

    /// Set the [`DBFactory`] to use. One will be created if it's not set explicitly.
    pub fn with_factory(self, factory: DBFactory) -> Factoried<'a, V, B, U, FB, FU> {
        OpenDbRequestBuilder {
            name: self.name,
            version: self.version,
            on_blocked: self.on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory: factory.into(),
            future_types: PhantomData,
        }
    }

    /// Set the [blocked](https://developer.mozilla.org/en-US/docs/Web/API/IDBOpenDBRequest/blocked_event)
    /// event handler.
    #[callback_bounds(err(E), fut(FB2), func(B2))]
    pub fn with_on_blocked<E, B2, FB2>(
        self,
        on_blocked: B2,
    ) -> OpenDbRequestBuilder<'a, V, B2, U, FB2, FU, Fa> {
        OpenDbRequestBuilder {
            name: self.name,
            version: self.version,
            on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory: self.factory,
            future_types: PhantomData,
        }
    }

    /// Set the [upgradeneeded](https://developer.mozilla.org/en-US/docs/Web/API/IDBOpenDBRequest/upgradeneeded_event)
    /// event handler.
    #[callback_bounds(err(E), fut(FU2), func(U2))]
    pub fn with_on_upgrade_needed<E, U2, FU2>(
        self,
        on_upgrade_needed: U2,
    ) -> OpenDbRequestBuilder<'a, V, B, U2, FB, FU2, Fa> {
        OpenDbRequestBuilder {
            name: self.name,
            version: self.version,
            on_blocked: self.on_blocked,
            on_upgrade_needed,
            factory: self.factory,
            future_types: PhantomData,
        }
    }

    /// The name of the database being opened.
    #[inline]
    #[must_use]
    pub fn name(&'a self) -> &'a str {
        self.name.as_ref()
    }

    /// Get the version of the database being opened
    #[inline]
    #[must_use]
    pub fn version(&self) -> V
    where
        V: Copy,
    {
        self.version
    }
}

impl IntoFuture for Factoried<'_> {
    type Output = OpenDbOpResult;
    type IntoFuture = IntoFut;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version: _,
            on_blocked: _,
            on_upgrade_needed: _,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(factory.open_request(name.as_ref()), |req| {
            OpenDbRequest::bare(req)
        })
    }
}

#[callback_bounds(err(EB), fut(FB), func(B))]
#[callback_bounds(err(EU), fut(FU), func(U))]
impl<EB, EU, V: DBVersion, B, U, FB, FU> IntoFuture for Factoried<'_, V, B, U, FB, FU> {
    type Output = OpenDbOpResult<EB, EU>;
    type IntoFuture = IntoFut<EB, EU>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version,
            on_blocked,
            on_upgrade_needed,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(
            factory.open_versioned_request(name.as_ref(), version),
            |req| OpenDbRequest::new(req, on_blocked, on_upgrade_needed)
        )
    }
}

#[callback_bounds(err(EB), fut(FB), func(B))]
#[callback_bounds(err(EU), fut(FU), func(U))]
impl<EB, EU, B, U, FB, FU> IntoFuture for Factoried<'_, (), B, U, FB, FU> {
    type Output = OpenDbOpResult<EB, EU>;
    type IntoFuture = IntoFut<EB, EU>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version: _,
            on_blocked,
            on_upgrade_needed,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(factory.open_request(name.as_ref()), |req| {
            OpenDbRequest::new(req, on_blocked, on_upgrade_needed)
        })
    }
}

#[callback_bounds(err(E), fut(FU), func(U))]
impl<E, V: DBVersion, U, FU> IntoFuture for Factoried<'_, V, (), U, (), FU> {
    type Output = OpenDbOpResult<Error, E>;
    type IntoFuture = IntoFut<Error, E>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version,
            on_blocked: _,
            on_upgrade_needed,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(
            factory.open_versioned_request(name.as_ref(), version),
            |req| OpenDbRequest::with_upgrade(req, on_upgrade_needed)
        )
    }
}

#[callback_bounds(err(E), fut(FU), func(U))]
impl<E, U, FU> IntoFuture for Factoried<'_, (), (), U, (), FU> {
    type Output = OpenDbOpResult<Error, E>;
    type IntoFuture = IntoFut<Error, E>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version: _,
            on_blocked: _,
            on_upgrade_needed,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(factory.open_request(name.as_ref()), |req| {
            OpenDbRequest::with_upgrade(req, on_upgrade_needed)
        })
    }
}

#[callback_bounds(err(E), fut(FB), func(B))]
impl<E, V: DBVersion, B, FB> IntoFuture for Factoried<'_, V, B, (), FB, ()> {
    type Output = OpenDbOpResult<E>;
    type IntoFuture = IntoFut<E>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version,
            on_blocked,
            on_upgrade_needed: _,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(
            factory.open_versioned_request(name.as_ref(), version),
            |req| OpenDbRequest::with_block(req, on_blocked)
        )
    }
}

#[callback_bounds(err(E), fut(FB), func(B))]
impl<E, B, FB> IntoFuture for Factoried<'_, (), B, (), FB, ()> {
    type Output = OpenDbOpResult<E>;
    type IntoFuture = IntoFut<E>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version: _,
            on_blocked,
            on_upgrade_needed: _,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(factory.open_request(name.as_ref()), |req| {
            OpenDbRequest::with_block(req, on_blocked)
        })
    }
}

impl<V: DBVersion> IntoFuture for Factoried<'_, V> {
    type Output = OpenDbOpResult;
    type IntoFuture = IntoFut;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            name,
            version,
            on_blocked: _,
            on_upgrade_needed: _,
            factory,
            future_types: _,
        } = self;

        maybe_errored!(
            factory.open_versioned_request(name.as_ref(), version),
            |req| OpenDbRequest::bare(req)
        )
    }
}

impl<'a, EB, EU, V, B, U, FB, FU> IntoFuture for OpenDbRequestBuilder<'a, V, B, U, FB, FU>
where
    EB: Unpin,
    EU: Unpin,
    Factoried<'a, V, B, U, FB, FU>:
        IntoFuture<Output = OpenDbOpResult<EB, EU>, IntoFuture = IntoFut<EB, EU>>,
{
    type Output = OpenDbOpResult<EB, EU>;
    type IntoFuture = IntoFut<EB, EU>;

    fn into_future(self) -> Self::IntoFuture {
        match DBFactory::new() {
            Ok(factory) => self.with_factory(factory).into_future(),
            Err(e) => MaybeErrored::errored(e.into()),
        }
    }
}

impl<V, B, U, FB, FU, Fa> Clone for OpenDbRequestBuilder<'_, V, B, U, FB, FU, Fa>
where
    V: Clone,
    B: Clone,
    U: Clone,
    Fa: Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            on_blocked: self.on_blocked.clone(),
            on_upgrade_needed: self.on_upgrade_needed.clone(),
            factory: self.factory.clone(),
            future_types: PhantomData,
        }
    }
}
