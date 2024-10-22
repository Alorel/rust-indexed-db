use super::Database;
use crate::error::Error;
use crate::internal_utils::SystemRepr;
use crate::object_store::ObjectStore;
use crate::KeyPath;
use internal_macros::generic_bounds;
use sealed::sealed;

/// Builder for [`Database::create_object_store`].
///
/// Finalise with a call to [`Build::build`](crate::Build::build).
#[must_use]
pub struct StoreBuilder<'a, N, AI = (), KP = ()> {
    db: &'a Database,
    store_name: N,
    auto_increment: AI,
    key_path: KP,
}

impl<'a, N> StoreBuilder<'a, N> {
    #[generic_bounds(store_name(N))]
    #[inline]
    pub(super) fn new(db: &'a Database, store_name: N) -> Self {
        Self {
            db,
            store_name,
            auto_increment: (),
            key_path: (),
        }
    }
}

impl<'a, N, AI, KP> StoreBuilder<'a, N, AI, KP> {
    /// If `true`, the object store has a
    /// [key generator](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_generator).
    ///
    /// Defaults to `false`.
    #[inline]
    pub fn with_auto_increment(self, auto_increment: bool) -> StoreBuilder<'a, N, bool, KP> {
        StoreBuilder {
            db: self.db,
            store_name: self.store_name,
            auto_increment,
            key_path: self.key_path,
        }
    }

    /// The [key path](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_path) to be
    /// used by the new object store.
    /// If empty, the object store is created without a key path and uses
    /// [out-of-line keys](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#out-of-line_key).
    #[generic_bounds(key_path(KP2))]
    #[inline]
    pub fn with_key_path<KP2>(
        self,
        key_path: KeyPath<KP2>,
    ) -> StoreBuilder<'a, N, AI, KeyPath<KP2>> {
        StoreBuilder {
            db: self.db,
            store_name: self.store_name,
            auto_increment: self.auto_increment,
            key_path,
        }
    }

    #[generic_bounds(store_name(N))]
    fn build_with_params(
        &self,
        p: &web_sys::IdbObjectStoreParameters,
    ) -> crate::Result<ObjectStore<'a>> {
        let sys = self
            .db
            .as_sys()
            .create_object_store_with_optional_parameters(self.store_name.as_ref(), p)?;

        Ok(ObjectStore::from_version_change(sys, self.db))
    }
}

#[generic_bounds(store_name(N))]
#[sealed]
impl<'a, N> crate::Build for StoreBuilder<'a, N> {
    type Ok = ObjectStore<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let sys = self
            .db
            .as_sys()
            .create_object_store(self.store_name.as_ref())?;
        Ok(ObjectStore::from_version_change(sys, self.db))
    }
}

#[generic_bounds(store_name(N))]
#[sealed]
impl<'a, N> crate::Build for StoreBuilder<'a, N, bool> {
    type Ok = ObjectStore<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let p = web_sys::IdbObjectStoreParameters::new();
        p.set_auto_increment(self.auto_increment);

        self.build_with_params(&p)
    }
}

#[generic_bounds(store_name(N), key_path(KP))]
#[sealed]
impl<'a, N, KP> crate::Build for StoreBuilder<'a, N, (), KeyPath<KP>> {
    type Ok = ObjectStore<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let p = web_sys::IdbObjectStoreParameters::new();
        p.set_key_path(&self.key_path.to_js());

        self.build_with_params(&p)
    }
}

#[generic_bounds(store_name(N), key_path(KP))]
#[sealed]
impl<'a, N, KP> crate::Build for StoreBuilder<'a, N, bool, KeyPath<KP>> {
    type Ok = ObjectStore<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let p = web_sys::IdbObjectStoreParameters::new();
        p.set_auto_increment(self.auto_increment);
        p.set_key_path(&self.key_path.to_js());

        self.build_with_params(&p)
    }
}
