use crate::factory::DatabaseDetails;

use super::ArrayMapIter;

/// An iterator over the results of future returned by [`DBFactory::databases`](crate::factory::DBFactory::databases).
pub type ListDatabasesIter = ArrayMapIter<DatabaseDetails>;

impl ListDatabasesIter {
    /// [`new`](ArrayMapIter::new) alias for [`ListDatabasesIter`].
    pub(crate) fn list_databases(array: js_sys::Array) -> Self {
        Self::new(array.into_iter(), TryInto::try_into)
    }
}
