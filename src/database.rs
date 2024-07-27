use std::borrow::Cow;

use delegate_display::DelegateDebug;
use wasm_bindgen::prelude::*;

use internal_macros::errdoc;
pub use store_name::ObjectStoreName;
pub use store_params::ObjectStoreParameters;
pub use tx_builder::TransactionBuilder;
pub use version_change::VersionChangeEvent;

use crate::internal_utils::SystemRepr;
use crate::iter::DomStringIter;
use crate::transaction::TransactionOptionsSys;
use crate::OpenDbResult;
use crate::TransactionOptions;
use crate::{DBFactory, OpenDbRequestBuilder, TransactionMode};

mod store_name;
mod store_params;
mod tx_builder;
mod version_change;

/// Wrapper around an [`IndexedDB`](web_sys::Database)
#[derive(DelegateDebug, Clone)]
pub struct Database(DbSys);

impl Database {
    /// Close the database connection
    #[inline]
    pub fn close(self) {
        self.as_sys().close();
    }

    /// Close and delete the database
    ///
    /// # Errors
    ///
    /// [Undocumented](https://developer.mozilla.org/en-US/docs/Web/API/IDBFactory/deleteDatabase) as of the release
    /// of this version.
    pub async fn delete(self) -> OpenDbResult<()> {
        let name = self.name();
        self.as_sys().close();

        Self::delete_by_name(&name).await
    }

    /// Delete the object store with the given name
    #[errdoc(Database(
        InvalidStateErrorObjectStore,
        TransactionInactiveError,
        NotFoundErrorDeleteObjectStore
    ))]
    pub fn delete_object_store(&self, name: &str) -> crate::Result<()> {
        if let Err(e) = self.as_sys().delete_object_store(name) {
            Err(e.into())
        } else {
            Ok(())
        }
    }

    /// Get the database name
    #[inline]
    #[must_use]
    pub fn name(&self) -> String {
        self.as_sys().name()
    }

    /// List the names of the object stores within this database
    #[inline]
    pub fn object_store_names(&self) -> DomStringIter {
        DomStringIter::new(self.as_sys().object_store_names())
    }

    /// Start a transaction on the given store name(s). Finish the builder with a call to [`TryInto::try_into`].
    #[errdoc(Database(NotFoundErrorTx, InvalidAccessErrorTx, TypeErrorTx))]
    #[inline]
    pub fn transaction<S: ObjectStoreName>(&self, store_names: S) -> TransactionBuilder<S> {
        TransactionBuilder::new(self, store_names)
    }

    /// Get the database version
    #[inline]
    #[must_use]
    pub fn version(&self) -> f64 {
        self.as_sys().version()
    }
}

impl Database {
    /// Delete the database with the given name. Convenience method for [`DBFactory::delete_db`] - use that if you need
    /// to delete more than one database.
    ///
    /// # Errors
    ///
    /// [Undocumented](https://developer.mozilla.org/en-US/docs/Web/API/IDBFactory/deleteDatabase) as of the release
    /// of this version.
    pub async fn delete_by_name(name: &str) -> OpenDbResult<()> {
        DBFactory::new()?.delete_db(name).await.map_err(Into::into)
    }

    /// Open a database with the given name. Convenience method for [`OpenDbRequestBuilder::new`] - use it when opening
    /// multiple databases.
    #[inline]
    pub fn open<'a, N: Into<Cow<'a, str>>>(name: N) -> OpenDbRequestBuilder<'a> {
        OpenDbRequestBuilder::new(name)
    }
}

impl Database {
    pub(crate) fn from_req(req: &web_sys::IdbRequest) -> Self {
        let raw_db = req
            .result()
            .expect("IndexedDB open DB event DB unwrap errored")
            .unchecked_into::<DbSys>();

        Self(raw_db)
    }

    pub(crate) fn from_event(event: &web_sys::Event) -> Self {
        let req = event
            .target()
            .expect("IndexedDB open DB event had no target")
            .unchecked_into();

        Self::from_req(&req)
    }
}

impl SystemRepr for Database {
    type Repr = DbSys;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.0
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.0
    }
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(extends = web_sys::IdbDatabase, js_name = IDBDatabase)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) type DbSys;

    #[wasm_bindgen(catch, method, structural, js_class = "IDBDatabase", js_name = transaction, skip_typescript)]
    pub(crate) fn transaction_with_str_and_mode_and_opts(
        this: &DbSys,
        store_name: &str,
        mode: TransactionMode,
        opts: &TransactionOptionsSys,
    ) -> Result<web_sys::IdbTransaction, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "IDBDatabase", js_name = transaction, skip_typescript)]
    pub(crate) fn transaction_with_str_sequence_and_mode_and_opts(
        this: &DbSys,
        store_names: &js_sys::Array,
        mode: TransactionMode,
        opts: &TransactionOptionsSys,
    ) -> Result<web_sys::IdbTransaction, JsValue>;
}

#[cfg(test)]
mod create_object_store {
    use std::future;

    use tokio::sync::oneshot;

    use crate::test_util::prelude::*;

    use super::*;

    #[wasm_bindgen_test]
    async fn constraint_error() {
        let name = random_name();
        let (tx, rx) = oneshot::channel();
        let _ = Database::open(&name)
            .with_on_upgrade_needed({
                let name = name.clone();
                move |evt| {
                    let a = evt.db().create_object_store(&name).err();
                    let b = evt.db().create_object_store(&name).err();
                    let _ = tx.send((a, b));
                    future::ready(Ok::<_, Error>(()))
                }
            })
            .await
            .expect(MSG_DB_OPEN);

        let (a, b) = rx.await.expect("rx");
        assert!(a.is_none(), "first");
        assert!(
            matches!(
                b,
                Some(Error::DomException(DomException::ConstraintError(_)))
            ),
            "second"
        );
    }

    #[wasm_bindgen_test]
    async fn invalid_state_error() {
        let name = random_name();
        let db = Database::open(&name).await.expect(MSG_DB_OPEN);

        assert!(
            matches!(
                db.create_object_store(&name),
                Err(Error::DomException(DomException::InvalidStateError(_)))
            ),
            "Error kind"
        );
    }
}
