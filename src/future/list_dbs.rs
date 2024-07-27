use super::ArrayMapFuture;
use crate::factory::DatabaseDetails;
use crate::iter::ListDatabasesIter;
use wasm_bindgen_futures::JsFuture;

/// A [`Future`](std::future::Future) returned by [`DBFactory::databases`](crate::factory::DBFactory::databases).
pub type ListDatabasesFuture = ArrayMapFuture<DatabaseDetails, JsFuture>;

impl ListDatabasesFuture {
    pub(crate) fn list_databases(promise: js_sys::Promise) -> Self {
        Self::new(promise.into(), ListDatabasesIter::list_databases)
    }
}
