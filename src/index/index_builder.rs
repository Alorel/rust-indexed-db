use super::Index;
use crate::error::Error;
use crate::internal_utils::SystemRepr;
use crate::object_store::ObjectStore;
use crate::KeyPath;
use internal_macros::generic_bounds;
use sealed::sealed;

/// Builder for [`ObjectStore::create_index`].
///
/// Finalise with a call to [`Build::build`](crate::Build::build).
#[must_use]
pub struct IndexBuilder<'a, N, KP, U = (), ME = ()> {
    store: &'a ObjectStore<'a>,
    name: N,
    key_path: KeyPath<KP>,
    unique: U,
    multi_entry: ME,
}

impl<'a, N, KP> IndexBuilder<'a, N, KP> {
    #[generic_bounds(index_name(N), key_path(KP))]
    #[inline]
    pub(super) fn new(store: &'a ObjectStore<'a>, name: N, key_path: KeyPath<KP>) -> Self {
        Self {
            store,
            name,
            key_path,
            unique: (),
            multi_entry: (),
        }
    }
}

impl<'a, N, KP, U, ME> IndexBuilder<'a, N, KP, U, ME> {
    /// If true, the index will not allow duplicate values for a single key.
    ///
    /// Defaults to `false`.
    #[inline]
    pub fn with_unique(self, unique: bool) -> IndexBuilder<'a, N, KP, bool, ME> {
        IndexBuilder {
            store: self.store,
            name: self.name,
            key_path: self.key_path,
            unique,
            multi_entry: self.multi_entry,
        }
    }

    /// If `true`, the index will add an entry in the index for each array element when the key path resolves to an
    /// array. If `false`, it will add one single entry containing the array.
    ///
    /// Defaults to `false`.
    #[inline]
    pub fn with_multi_entry(self, multi_entry: bool) -> IndexBuilder<'a, N, KP, U, bool> {
        IndexBuilder {
            store: self.store,
            name: self.name,
            key_path: self.key_path,
            unique: self.unique,
            multi_entry,
        }
    }

    #[generic_bounds(index_name(N), key_path(KP))]
    fn build_with_params(&self, params: &web_sys::IdbIndexParameters) -> crate::Result<Index<'a>> {
        let kp = self.key_path.to_js();
        let sys = self
            .store
            .as_sys()
            .create_index_with_str_sequence_and_optional_parameters(
                self.name.as_ref(),
                &kp,
                params,
            )?;

        Ok(Index::new(self.store, sys))
    }
}

#[generic_bounds(index_name(N), key_path(KP))]
#[sealed]
impl<'a, N, KP> crate::Build for IndexBuilder<'a, N, KP> {
    type Ok = Index<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let kp = self.key_path.to_js();
        let sys = self
            .store
            .as_sys()
            .create_index_with_str_sequence(self.name.as_ref(), &kp)?;
        Ok(Index::new(self.store, sys))
    }
}

#[generic_bounds(index_name(N), key_path(KP))]
#[sealed]
impl<'a, N, KP> crate::Build for IndexBuilder<'a, N, KP, bool> {
    type Ok = Index<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let p = web_sys::IdbIndexParameters::new();
        p.set_unique(self.unique);

        self.build_with_params(&p)
    }
}

#[generic_bounds(index_name(N), key_path(KP))]
#[sealed]
impl<'a, N, KP> crate::Build for IndexBuilder<'a, N, KP, (), bool> {
    type Ok = Index<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let p = web_sys::IdbIndexParameters::new();
        p.set_multi_entry(self.multi_entry);

        self.build_with_params(&p)
    }
}

#[generic_bounds(index_name(N), key_path(KP))]
#[sealed]
impl<'a, N, KP> crate::Build for IndexBuilder<'a, N, KP, bool, bool> {
    type Ok = Index<'a>;
    type Err = Error;

    fn build(self) -> Result<Self::Ok, Self::Err> {
        let p = web_sys::IdbIndexParameters::new();
        p.set_unique(self.unique);
        p.set_multi_entry(self.multi_entry);

        self.build_with_params(&p)
    }
}
