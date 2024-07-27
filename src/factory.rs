use std::borrow::Cow;

use delegate_display::DelegateDebug;
use wasm_bindgen::prelude::*;
use web_sys::{Window, WorkerGlobalScope};

pub use db_version::DBVersion;
pub use req_builder::OpenDbRequestBuilder;

use crate::error::OpenDbError;
use crate::future::{MaybeErrored, Request, VoidRequest};
use crate::internal_utils::SystemRepr;
use crate::{OpenDbOpResult, OpenDbResult};

mod db_version;

iffeat! {
    #[cfg(feature = "list-databases")]
    mod list_dbs;
    pub use list_dbs::DatabaseDetails;
}

mod req_builder;

/// An [`IDBFactory`](https://developer.mozilla.org/en-US/docs/Web/API/IDBFactory) implementation.
///
/// Use this instead of [`Database::open`](crate::Database::open) or
/// [`Database::open_with_version`](crate::Database::open_with_version) when you want to open multiple databases.
#[derive(Clone, Eq, PartialEq, DelegateDebug)]
pub struct DBFactory(FactorySys);

impl DBFactory {
    /// Create a new instance of the factory.
    pub fn new() -> OpenDbResult<Self> {
        raw_factory().map(Self)
    }

    /// Delete the database with the given name
    ///
    /// # Errors
    ///
    /// [Undocumented](https://developer.mozilla.org/en-US/docs/Web/API/IDBFactory/deleteDatabase) as of the release
    /// of this version.
    pub fn delete_db(&self, name: &str) -> MaybeErrored<VoidRequest> {
        match self.as_sys().delete_database(name) {
            Ok(req) => MaybeErrored::running(Request::void(req.unchecked_into())),
            Err(e) => MaybeErrored::errored(e.into()),
        }
    }

    /// Open a database with the given name. Convenience method for [`OpenDbRequestBuilder::new`] followed by
    /// [`with_factory`](OpenDbRequestBuilder::with_factory).
    pub fn open_db<'a, N>(&self, name: N) -> OpenDbRequestBuilder<'a, (), (), (), (), (), Self>
    where
        N: Into<Cow<'a, str>>,
    {
        OpenDbRequestBuilder::new(name).with_factory(self.clone())
    }

    pub(crate) fn open_versioned_request<V, B, U>(
        &self,
        name: &str,
        version: V,
    ) -> OpenDbOpResult<VoidRequest, B, U>
    where
        V: DBVersion,
    {
        let res = version.into_idb_open_request(self, name);
        fmt_open_raw(res).map_err(Into::into)
    }

    pub(crate) fn open_request<B, U>(&self, name: &str) -> OpenDbOpResult<VoidRequest, B, U> {
        fmt_open_raw(self.as_sys().open(name)).map_err(Into::into)
    }
}

impl SystemRepr for DBFactory {
    type Repr = FactorySys;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.0
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.0
    }
}

fn fmt_open_raw(res: Result<web_sys::IdbOpenDbRequest, JsValue>) -> OpenDbResult<VoidRequest> {
    if let Ok(v) = res {
        Ok(Request::void(v.unchecked_into()))
    } else {
        Err(OpenDbError::VersionZero)
    }
}

/// Access to the low-level `wasm-bindgen` factory
fn raw_factory() -> OpenDbResult<FactorySys> {
    let global: Global = js_sys::global().unchecked_into();

    let maybe_factory = if !global.window().is_undefined() {
        global.unchecked_into::<Window>().indexed_db()
    } else if !global.worker().is_undefined() {
        global.unchecked_into::<WorkerGlobalScope>().indexed_db()
    } else if !global.node_global().is_undefined() {
        global.unchecked_into::<NodeGlobal>().indexed_db()
    } else {
        return Err(OpenDbError::UnsupportedEnvironment);
    }?;

    if let Some(f) = maybe_factory {
        Ok(f.unchecked_into())
    } else {
        Err(OpenDbError::NullFactory)
    }
}

#[wasm_bindgen]
extern "C" {
    type Global;
    type NodeGlobal;

    #[wasm_bindgen(extends = web_sys::IdbFactory, js_name = IDBFactory)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) type FactorySys;

    #[wasm_bindgen(catch, method, structural, js_class = "IDBFactory", js_name = databases, skip_typescript)]
    #[cfg(feature = "list-databases")]
    pub(crate) fn databases(this: &FactorySys) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, getter, js_name = Window)]
    fn window(this: &Global) -> JsValue;

    #[wasm_bindgen(method, getter, js_name = WorkerGlobalScope)]
    fn worker(this: &Global) -> JsValue;

    #[wasm_bindgen(method, getter, js_name = global)]
    fn node_global(this: &Global) -> JsValue;

    #[wasm_bindgen(method, getter, catch, js_name = indexedDB)]
    fn indexed_db(this: &NodeGlobal) -> Result<Option<web_sys::IdbFactory>, JsValue>;
}
