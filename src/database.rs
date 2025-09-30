//! An [`IDBDatabase`](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase) implementation.

use delegate_display::DelegateDebug;
use wasm_bindgen::prelude::*;

use internal_macros::{errdoc, generic_bounds};
pub use store_builder::StoreBuilder;
pub use store_name::ObjectStoreName;
pub use tx_builder::TransactionBuilder;
pub use version_change_event::VersionChangeEvent;

use crate::factory::{DBFactory, OpenDbRequestBuilder};
use crate::internal_utils::SystemRepr;
use crate::iter::DomStringIter;
use crate::transaction::TransactionOptionsSys;

pub(crate) mod db_sys;
mod store_builder;
mod store_name;
mod tx_builder;
mod version_change_event;

use crate::error::{SimpleValueError, UnexpectedDataError};
use crate::future::VoidRequest;
pub(crate) use db_sys::DbSys;

iffeat! {
    #[cfg(feature = "version-change")]
    mod version_change_listener;
    pub use version_change_listener::VersionChangeListener;
}

/// An [`IDBDatabase`](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase) implementation.
#[derive(DelegateDebug, Clone)]
pub struct Database(DbSys);

impl Database {
    /// Open a database with the given name. Convenience method for [`OpenDbRequestBuilder::new`] - use it when opening
    /// multiple databases.
    #[errdoc(Database(
        InvalidStateErrorObjectStore,
        TransactionInactiveError,
        ConstraintError,
        InvalidAccessErrorCreateObjectStore,
    ))]
    #[generic_bounds(db_name(N))]
    #[inline]
    pub fn open<N>(name: N) -> OpenDbRequestBuilder<N> {
        OpenDbRequestBuilder::new(name)
    }

    /// Create an object store with the given name.
    #[generic_bounds(store_name(N))]
    #[inline]
    pub fn create_object_store<N>(&self, name: N) -> StoreBuilder<'_, N> {
        StoreBuilder::new(self, name)
    }

    /// Close the database connection in a background thread.
    #[inline]
    pub fn close(self) {
        self.as_sys().close();
    }

    /// Close and delete the database.
    ///
    /// # Errors
    ///
    /// [Undocumented](https://developer.mozilla.org/en-US/docs/Web/API/IDBFactory/deleteDatabase) as of the release
    /// of this version.
    pub fn delete(self) -> crate::OpenDbResult<VoidRequest> {
        let name = self.name();
        self.close();

        Self::delete_by_name(&name)
    }

    /// Delete the object store with the given name.
    #[errdoc(Database(
        InvalidStateErrorObjectStore,
        TransactionInactiveError,
        NotFoundErrorDeleteObjectStore
    ))]
    #[allow(clippy::missing_errors_doc)]
    pub fn delete_object_store(&self, name: &str) -> crate::Result<()> {
        if let Err(e) = self.as_sys().delete_object_store(name) {
            Err(e.into())
        } else {
            Ok(())
        }
    }

    /// Get the database name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> String {
        self.as_sys().name()
    }

    /// List the names of the object stores within this database.
    #[inline]
    pub fn object_store_names(&self) -> DomStringIter<'_> {
        DomStringIter::new(self.as_sys().object_store_names())
    }

    /// Start a transaction on the given store name(s). Finish the builder with a call to
    /// [`Build::build`](crate::Build::build).
    #[errdoc(Database(NotFoundErrorTx, InvalidAccessErrorTx))]
    #[inline]
    pub fn transaction<S: ObjectStoreName>(&self, store_names: S) -> TransactionBuilder<'_, S> {
        TransactionBuilder::new(self, store_names)
    }

    /// Get the database version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> f64 {
        self.as_sys().version()
    }

    /// Delete the database with the given name. Convenience method for [`DBFactory::delete_db`] - use that if you need
    /// to delete more than one database.
    ///
    /// # Errors
    ///
    /// [Undocumented](https://developer.mozilla.org/en-US/docs/Web/API/IDBFactory/deleteDatabase) as of the release
    /// of this version.
    pub fn delete_by_name(name: &str) -> crate::OpenDbResult<VoidRequest> {
        DBFactory::new()?.delete_db(name).map_err(Into::into)
    }

    /// Create a new
    /// [`versionchange`](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/versionchange_event)
    /// listener.
    #[cfg(feature = "version-change")]
    #[allow(clippy::missing_errors_doc)]
    pub fn version_changes(&self) -> crate::Result<VersionChangeListener> {
        VersionChangeListener::new(self.clone())
    }
}

impl Database {
    pub(crate) fn from_req(req: &web_sys::IdbRequest) -> crate::Result<Self> {
        Self::from_js(req.result()?)
    }

    pub(crate) fn from_event(event: &web_sys::Event) -> crate::Result<Self> {
        match event.target() {
            Some(target) => match target.dyn_ref() {
                Some(req) => Self::from_req(req),
                None => Err(SimpleValueError::DynCast(target.unchecked_into()).into()),
            },
            None => Err(UnexpectedDataError::NoEventTarget.into()),
        }
    }

    fn from_js(js: JsValue) -> crate::Result<Self> {
        match js.dyn_into::<web_sys::IdbDatabase>() {
            Ok(base_db) => Ok(Self(base_db.unchecked_into())),
            Err(e) => Err(SimpleValueError::DynCast(e).into()),
        }
    }
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for Database {
    type Repr = DbSys;

    #[inline]
    #[doc(hidden)]
    fn as_sys(&self) -> &Self::Repr {
        &self.0
    }

    #[inline]
    #[doc(hidden)]
    fn into_sys(self) -> Self::Repr {
        self.0
    }
}
