use accessory::Accessors;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{IdbOpenDbRequest, IdbRequest};

use crate::idb_database::IdbDatabase;
use crate::prelude::IdbTransaction;

/// The DB version has changed
#[derive(Debug, Accessors)]
pub struct IdbVersionChangeEvent {
    event: web_sys::IdbVersionChangeEvent,

    /// Database associated with the version change
    #[access(get)]
    db: IdbDatabase,

    req: IdbOpenDbRequest,
}

pub(crate) type IdbVersionChangeCallback =
    Closure<dyn FnMut(web_sys::IdbVersionChangeEvent) -> Result<(), JsValue> + 'static>;

impl IdbVersionChangeEvent {
    pub(crate) fn new(event: web_sys::IdbVersionChangeEvent) -> Self {
        let req: IdbOpenDbRequest = event
            .target()
            .expect("Failed to unwrap IdbOpenDbRequest event target")
            .unchecked_into();
        let base_db: web_sys::IdbDatabase = req
            .result()
            .expect("Failed to unwrap IdbOpenDbRequest result")
            .unchecked_into();

        Self {
            event,
            db: IdbDatabase::new(base_db),
            req,
        }
    }

    pub(crate) fn wrap_callback<F>(cb: F) -> IdbVersionChangeCallback
    where
        F: Fn(&Self) -> Result<(), JsValue> + 'static,
    {
        let b = Box::new(move |event: web_sys::IdbVersionChangeEvent| cb(&Self::new(event)));
        Closure::wrap(b)
    }

    /// Old DB version; set to 0 on new DBs
    #[inline]
    #[must_use]
    pub fn old_version(&self) -> f64 {
        self.event.old_version()
    }

    /// New DB version
    #[inline]
    #[must_use]
    pub fn new_version(&self) -> f64 {
        self.event
            .new_version()
            .expect("Unable to unwrap new version")
    }

    /// Transaction associated with the version change
    #[inline]
    #[must_use]
    pub fn transaction(&self) -> IdbTransaction {
        let inner = self
            .req
            .unchecked_ref::<IdbRequest>()
            .transaction()
            .expect("Failed to unwrap IdbOpenDbRequest transaction");

        IdbTransaction::new(inner, &self.db)
    }
}

impl AsRef<IdbDatabase> for IdbVersionChangeEvent {
    #[inline]
    fn as_ref(&self) -> &IdbDatabase {
        self.db()
    }
}
