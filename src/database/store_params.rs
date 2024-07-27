use delegate_display::DelegateDebug;

use internal_macros::{errdoc, generate_with};

use crate::internal_utils::SystemRepr;
use crate::{KeyPath, ObjectStore};

use super::Database;

/// Parameters to pass to
/// [`create_object_store_with_params`](Database::create_object_store_with_params).
#[derive(Clone, PartialEq, Eq, DelegateDebug)]
pub struct ObjectStoreParameters(web_sys::IdbObjectStoreParameters);

impl ObjectStoreParameters {
    /// Create a new instance of the struct.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(web_sys::IdbObjectStoreParameters::new())
    }

    /// If `true`, the object store has a
    /// [key generator](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_generator).
    ///
    /// Defaults to `false`.
    #[generate_with]
    #[inline]
    pub fn set_auto_increment(&mut self, auto_increment: bool) -> &mut Self {
        self.0.auto_increment(auto_increment);
        self
    }

    /// The [key path](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_path) to be
    /// used by the new object store.
    /// If empty, the object store is created without a key path and uses
    /// [out-of-line keys](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#out-of-line_key).
    #[generate_with]
    #[inline]
    pub fn set_key_path<I, S>(&mut self, key_path: &KeyPath<S>) -> &mut Self
    where
        for<'a> &'a S: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        self.0.key_path(Some(&key_path.to_js_value()));
        self
    }
}

/// Object store creation.
impl Database {
    /// Create an object store with the given name
    #[errdoc(Database(
        InvalidStateErrorObjectStore,
        TransactionInactiveError,
        ConstraintError,
    ))]
    pub fn create_object_store(&self, name: &str) -> crate::Result<ObjectStore> {
        match self.as_sys().create_object_store(name) {
            Ok(store) => Ok(ObjectStore::from_version_change(store, self)),
            Err(e) => Err(e.into()),
        }
    }

    /// Create an object store with the given name and config
    #[errdoc(Database(
        InvalidStateErrorObjectStore,
        TransactionInactiveError,
        ConstraintError,
        InvalidAccessErrorCreateObjectStore,
    ))]
    pub fn create_object_store_with_params(
        &self,
        name: &str,
        params: &ObjectStoreParameters,
    ) -> crate::Result<ObjectStore> {
        match self
            .as_sys()
            .create_object_store_with_optional_parameters(name, params.as_sys())
        {
            Ok(store) => Ok(ObjectStore::from_version_change(store, self)),
            Err(e) => Err(e.into()),
        }
    }
}

impl SystemRepr for ObjectStoreParameters {
    type Repr = web_sys::IdbObjectStoreParameters;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.0
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.0
    }
}
