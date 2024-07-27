use delegate_display::DelegateDebug;

use internal_macros::generate_with;

use crate::internal_utils::SystemRepr;

/// Parameters for creating an [`Index`](super::Index).
#[derive(Clone, PartialEq, Eq, DelegateDebug)]
pub struct IndexParameters(web_sys::IdbIndexParameters);

impl IndexParameters {
    /// Create an empty instance of the struct
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(web_sys::IdbIndexParameters::new())
    }

    /// If true, the index will not allow duplicate values for a single key.
    ///
    /// Defaults to `false`.
    #[generate_with]
    #[inline]
    pub fn set_unique(&mut self, unique: bool) -> &mut Self {
        self.0.unique(unique);
        self
    }

    /// If `true`, the index will add an entry in the index for each array element when the key path resolves to an
    /// array. If `false`, it will add one single entry containing the array.
    ///
    /// Defaults to `false`.
    #[generate_with]
    #[inline]
    pub fn set_multi_entry(&mut self, multi_entry: bool) -> &mut Self {
        self.0.multi_entry(multi_entry);
        self
    }
}

impl SystemRepr for IndexParameters {
    type Repr = web_sys::IdbIndexParameters;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.0
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.0
    }
}
