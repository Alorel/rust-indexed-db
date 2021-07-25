use wasm_bindgen::{prelude::*, JsCast};
use web_sys::IdbOpenDbRequest;

use crate::idb_database::IdbDatabase;

/// The DB version has changed
#[derive(Debug)]
pub struct IdbVersionChangeEvent {
    event: web_sys::IdbVersionChangeEvent,
    db: IdbDatabase,
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
    pub fn old_version(&self) -> f64 {
        self.event.old_version()
    }

    /// New DB version
    #[inline]
    pub fn new_version(&self) -> f64 {
        self.event
            .new_version()
            .expect("Unable to unwrap new version")
    }

    #[inline]
    pub fn db(&self) -> &IdbDatabase {
        &self.db
    }
}

impl AsRef<IdbDatabase> for IdbVersionChangeEvent {
    #[inline]
    fn as_ref(&self) -> &IdbDatabase {
        self.db()
    }
}
