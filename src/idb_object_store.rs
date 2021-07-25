//! Object store-related code

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::DomException;

pub use idb_object_store_parameters::*;
#[cfg(feature = "indices")]
use {
    crate::{idb_index::IdbIndex, idb_key_path::IdbKeyPath},
    web_sys::IdbIndexParameters,
};

use crate::dom_string_iterator::DomStringIterator;
use crate::idb_database::IdbDatabase;
use crate::idb_query_source::IdbQuerySource;
use crate::idb_transaction::IdbTransaction;
use crate::request::VoidRequest;

mod idb_object_store_parameters;

#[derive(Debug)]
pub struct IdbObjectStore<'a> {
    inner: web_sys::IdbObjectStore,
    db: &'a IdbDatabase,
    tx: Option<&'a IdbTransaction<'a>>,
}

impl IdbObjectStore<'_> {
    /// Clear all the documents in the object store
    pub fn clear(&self) -> Result<VoidRequest, DomException> {
        Ok(VoidRequest::new(self.inner.clear()?))
    }

    /// Clone and store the value on the object store. Throws if the computed key already exists.
    pub fn add_val<V: JsCast>(&self, val: &V) -> Result<VoidRequest, DomException> {
        Ok(VoidRequest::new(self.inner.add(val.unchecked_ref())?))
    }

    /// Clone and store the value on the object store. Throws if the computed key already exists.
    #[inline]
    pub fn add_val_owned<V: Into<JsValue>>(&self, val: V) -> Result<VoidRequest, DomException> {
        self.add_val(&val.into())
    }

    /// Clone and store the value in the object store at the given key. Throws if the key already
    /// exists.
    pub fn add_key_val<K, V>(&self, key: &K, val: &V) -> Result<VoidRequest, DomException>
    where
        K: JsCast,
        V: JsCast,
    {
        let base = self
            .inner
            .add_with_key(val.unchecked_ref(), key.unchecked_ref())?;
        Ok(VoidRequest::new(base))
    }

    /// Clone and store the value in the object store at the given key. Throws if the key already
    /// exists.
    #[inline]
    pub fn add_key_val_owned<K, V>(&self, key: K, val: &V) -> Result<VoidRequest, DomException>
    where
        K: Into<JsValue>,
        V: JsCast,
    {
        self.add_key_val(&key.into(), val)
    }

    /// Clone and store the value in the object store, overwriting any existing value.
    pub fn put_val<V: JsCast>(&self, val: &V) -> Result<VoidRequest, DomException> {
        Ok(VoidRequest::new(self.inner.put(val.unchecked_ref())?))
    }

    /// Clone and store the value in the object store, overwriting any existing value.
    #[inline]
    pub fn put_val_owned<V: Into<JsValue>>(&self, val: V) -> Result<VoidRequest, DomException> {
        self.put_val(&val.into())
    }

    /// Clone and store the value in the object store at the given key, overwriting any existing
    /// value.
    pub fn put_key_val<K, V>(&self, key: &K, val: &V) -> Result<VoidRequest, DomException>
    where
        K: JsCast,
        V: JsCast,
    {
        let base = self
            .inner
            .put_with_key(val.unchecked_ref(), key.unchecked_ref())?;
        Ok(VoidRequest::new(base))
    }

    /// Clone and store the value in the object store at the given key, overwriting any existing
    /// value.
    #[inline]
    pub fn put_key_val_owned<K, V>(&self, key: K, val: &V) -> Result<VoidRequest, DomException>
    where
        K: Into<JsValue>,
        V: JsCast,
    {
        self.put_key_val(&key.into(), val)
    }

    /// A list of the names of indices on objects in this object store.
    #[inline]
    pub fn index_names(&self) -> impl Iterator<Item = String> {
        DomStringIterator::from(self.inner.index_names())
    }

    /// The value of the auto increment flag for this object store.
    #[inline]
    pub fn auto_increment(&self) -> bool {
        self.inner.auto_increment()
    }

    // Indices
    cfg_if::cfg_if! {
        if #[cfg(feature = "indices")] {

            /// Create an index at the given key path
            ///
            /// Features required: `indices`
            pub fn create_index(&self, name: &str, key_path: &IdbKeyPath) -> Result<IdbIndex, DomException> {
                let base = self.inner.create_index_with_str_sequence(name, key_path.as_js_value());
                self.create_idx_common(base)
            }

            /// Create an index at the given key path with the given parameters
            ///
            /// Features required: `indices`
            pub fn create_index_with_params(
                &self,
                name: &str,
                key_path: &IdbKeyPath,
                params: &IdbIndexParameters
            ) -> Result<IdbIndex, DomException> {
                let base = self.inner
                  .create_index_with_str_sequence_and_optional_parameters(name, key_path.as_js_value(), params);
                self.create_idx_common(base)
            }

            fn create_idx_common(
                &self,
                src: Result<web_sys::IdbIndex, JsValue>,
            ) -> Result<IdbIndex, DomException> {
                Ok(IdbIndex::new(src?, &self))
            }
        }
    }
}

impl<'a> IdbObjectStore<'a> {
    #[inline]
    pub(crate) fn from_db(inner: web_sys::IdbObjectStore, db: &'a IdbDatabase) -> Self {
        Self {
            inner,
            db,
            tx: None,
        }
    }

    #[inline]
    pub(crate) fn from_tx(inner: web_sys::IdbObjectStore, tx: &'a IdbTransaction) -> Self {
        Self {
            inner,
            db: tx.db(),
            tx: Some(tx),
        }
    }

    /// The DB that spawned this store
    #[inline]
    pub fn db(&self) -> &'a IdbDatabase {
        &self.db
    }

    /// The transaction that spawned this store
    #[inline]
    pub fn transaction(&self) -> &Option<&'a IdbTransaction<'a>> {
        &self.tx
    }

    /// Delete this object store
    #[inline]
    pub fn delete_object_store(self) -> Result<(), DomException> {
        self.db().delete_object_store(&self.name())
    }

    /// Delete the record at the with the given key
    pub fn delete<K: JsCast>(&self, key: &K) -> Result<VoidRequest, DomException> {
        Ok(VoidRequest::new(self.inner.delete(key.unchecked_ref())?))
    }

    /// Delete the record at the with the given key
    #[inline]
    pub fn delete_owned<K: Into<JsValue>>(&self, key: K) -> Result<VoidRequest, DomException> {
        self.delete(&key.into())
    }
}

impl_query_source!(IdbObjectStore<'_>);
