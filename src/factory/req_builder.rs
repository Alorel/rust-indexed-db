use super::DBFactory;
use crate::error::OpenDbError;
use crate::future::OpenDbRequest;
use internal_macros::{generic_bounds, BuildIntoFut, StructName};
use sealed::sealed;

type WithFactory<N, V = (), B = (), U = ()> = OpenDbRequestBuilder<N, V, B, U, DBFactory>;

/// Database open request builder.
///
/// # Generics
///
/// | Generic param | Description |
/// |---|---|
/// | `N` | Database name |
/// | `V` | Database version |
/// | `B` | `blocked` event handler. |
/// | `U` | `upgradeneeded` event handler. |
/// | `Fa` | Either [`DBFactory`] or `()` if a factory should be generated on the fly |
#[derive(Clone, StructName, BuildIntoFut)]
#[must_use]
pub struct OpenDbRequestBuilder<N, V = (), B = (), U = (), Fa = ()> {
    name: N,
    version: V,
    on_blocked: B,
    on_upgrade_needed: U,
    factory: Fa,
}

impl<N> OpenDbRequestBuilder<N> {
    /// Open a database with the given name.
    #[inline]
    #[generic_bounds(db_name(N))]
    pub fn new(name: N) -> Self {
        Self {
            name,
            version: (),
            on_blocked: (),
            on_upgrade_needed: (),
            factory: (),
        }
    }
}

impl<N, V, B, U, Fa> OpenDbRequestBuilder<N, V, B, U, Fa> {
    /// Set the name of the database being opened.
    #[generic_bounds(db_name(N2))]
    pub fn with_name<N2>(self, name: N2) -> OpenDbRequestBuilder<N2, V, B, U, Fa> {
        OpenDbRequestBuilder {
            name,
            version: self.version,
            on_blocked: self.on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory: self.factory,
        }
    }

    /// Set the version of the database being opened.
    #[generic_bounds(db_version(V2))]
    pub fn with_version<V2>(self, version: V2) -> OpenDbRequestBuilder<N, V2, B, U, Fa> {
        OpenDbRequestBuilder {
            name: self.name,
            version,
            on_blocked: self.on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory: self.factory,
        }
    }

    /// Set the [`DBFactory`] to use. One will be created if it's not set explicitly.
    pub fn with_factory(self, factory: DBFactory) -> WithFactory<N, V, B, U> {
        OpenDbRequestBuilder {
            name: self.name,
            version: self.version,
            on_blocked: self.on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory,
        }
    }

    /// Set the [blocked](https://developer.mozilla.org/en-US/docs/Web/API/IDBOpenDBRequest/blocked_event)
    /// event handler.
    #[generic_bounds(blocked_cb(B2))]
    pub fn with_on_blocked<B2>(self, on_blocked: B2) -> OpenDbRequestBuilder<N, V, B2, U, Fa> {
        OpenDbRequestBuilder {
            name: self.name,
            version: self.version,
            on_blocked,
            on_upgrade_needed: self.on_upgrade_needed,
            factory: self.factory,
        }
    }

    /// Set the [upgradeneeded](https://developer.mozilla.org/en-US/docs/Web/API/IDBOpenDBRequest/upgradeneeded_event)
    /// event handler.
    #[generic_bounds(upgrade_cb(U2))]
    pub fn with_on_upgrade_needed<U2>(
        self,
        on_upgrade_needed: U2,
    ) -> OpenDbRequestBuilder<N, V, B, U2, Fa> {
        OpenDbRequestBuilder {
            name: self.name,
            version: self.version,
            on_blocked: self.on_blocked,
            on_upgrade_needed,
            factory: self.factory,
        }
    }

    /// The name of the database being opened.
    #[generic_bounds(db_name(N))]
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the version of the database being opened.
    #[generic_bounds(db_version(V))]
    #[inline]
    #[must_use]
    pub fn version(&self) -> V {
        self.version
    }
}

#[generic_bounds(db_name(N))]
#[sealed]
impl<N> crate::Build for WithFactory<N> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self.factory.open_request(self.name())?;
        Ok(OpenDbRequest::bare(req))
    }
}

#[generic_bounds(db_name(N), db_version(V), upgrade_cb(U), blocked_cb(B))]
#[sealed]
impl<N, V, B, U> crate::Build for WithFactory<N, V, B, U> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self
            .factory
            .open_versioned_request(self.name(), self.version())?;
        Ok(OpenDbRequest::with_both(
            req,
            self.on_blocked,
            self.on_upgrade_needed,
        ))
    }
}

#[generic_bounds(db_name(N), upgrade_cb(U), blocked_cb(B))]
#[sealed]
impl<N, B, U> crate::Build for WithFactory<N, (), B, U> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self.factory.open_request(self.name())?;
        Ok(OpenDbRequest::with_both(
            req,
            self.on_blocked,
            self.on_upgrade_needed,
        ))
    }
}

#[generic_bounds(db_name(N), db_version(V), upgrade_cb(U))]
#[sealed]
impl<N, V, U> crate::Build for WithFactory<N, V, (), U> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self
            .factory
            .open_versioned_request(self.name(), self.version())?;
        Ok(OpenDbRequest::with_upgrade(req, self.on_upgrade_needed))
    }
}

#[generic_bounds(db_name(N), upgrade_cb(U))]
#[sealed]
impl<N, U> crate::Build for WithFactory<N, (), (), U> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self.factory.open_request(self.name())?;
        Ok(OpenDbRequest::with_upgrade(req, self.on_upgrade_needed))
    }
}

#[generic_bounds(db_name(N), db_version(V), blocked_cb(B))]
#[sealed]
impl<N, V, B> crate::Build for WithFactory<N, V, B> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self
            .factory
            .open_versioned_request(self.name(), self.version())?;
        Ok(OpenDbRequest::with_block(req, self.on_blocked))
    }
}

#[generic_bounds(db_name(N), blocked_cb(B))]
#[sealed]
impl<N, B> crate::Build for WithFactory<N, (), B> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self.factory.open_request(self.name())?;
        Ok(OpenDbRequest::with_block(req, self.on_blocked))
    }
}

#[generic_bounds(db_name(N), db_version(V))]
#[sealed]
impl<N, V> crate::Build for WithFactory<N, V> {
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> crate::OpenDbResult<Self::Ok> {
        let req = self
            .factory
            .open_versioned_request(self.name(), self.version())?;
        Ok(OpenDbRequest::bare(req))
    }
}

#[sealed]
impl<N, V, B, U> crate::Build for OpenDbRequestBuilder<N, V, B, U>
where
    WithFactory<N, V, B, U>: crate::Build<Ok = OpenDbRequest, Err = OpenDbError>,
{
    type Ok = OpenDbRequest;
    type Err = OpenDbError;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let factory = DBFactory::new()?;

        self.with_factory(factory).build()
    }
}
