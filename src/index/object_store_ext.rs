pub use super::{Index, IndexBuilder};
use internal_macros::{errdoc, generic_bounds};

use crate::internal_utils::SystemRepr;
use crate::iter::DomStringIter;
use crate::object_store::ObjectStore;
use crate::KeyPath;

impl ObjectStore<'_> {
    /// Create and return a new index on the connected database. Note that this method must be called only from a
    /// [`VersionChange`](crate::transaction::TransactionMode::Versionchange) transaction mode callback.
    ///
    /// You'll want to set the key path to a [`KeyPath`](crate::KeyPathOld) or `&KeyPath`.
    #[errdoc(Index(
        ConstraintError,
        InvalidStateError,
        SyntaxError,
        TransactionInactiveError
    ))]
    #[generic_bounds(index_name(N), key_path(KP))]
    #[inline]
    pub fn create_index<N, KP>(&self, name: N, key_path: KeyPath<KP>) -> IndexBuilder<'_, N, KP> {
        IndexBuilder::new(self, name, key_path)
    }

    /// Delete the index with the given name. Must me called within a
    /// [`Versionchange`](crate::transaction::TransactionMode::Versionchange) transaction.
    #[errdoc(Index(InvalidStateError, TransactionInactiveError, NotFoundError))]
    #[allow(clippy::missing_errors_doc)]
    pub fn delete_index(&self, name: &str) -> crate::Result<()> {
        if let Err(e) = self.as_sys().delete_index(name) {
            Err(e.into())
        } else {
            Ok(())
        }
    }

    /// Open an index with the given name
    #[errdoc(Index(InvalidStateErrorIndex, NotFoundError))]
    #[allow(clippy::missing_errors_doc)]
    pub fn index(&self, name: &str) -> crate::Result<Index<'_>> {
        match self.as_sys().index(name) {
            Ok(sys) => Ok(Index::new(self, sys)),
            Err(e) => Err(e.into()),
        }
    }

    /// Return the names of the indices on this object store.
    pub fn index_names(&self) -> DomStringIter<'_> {
        DomStringIter::new(self.as_sys().index_names())
    }
}
