use sealed::sealed;
use wasm_bindgen::prelude::*;

use crate::internal_utils::SystemRepr;

use super::DBFactory;

/// A low-level trait marking the type as usable in [`Database::open_with_version`].
#[sealed]
pub trait DBVersion {
    /// Convert into a low level `IdbOpenDbRequest` object.
    fn into_idb_open_request(
        self,
        factory: &DBFactory,
        name: &str,
    ) -> Result<web_sys::IdbOpenDbRequest, JsValue>;
}

#[sealed]
impl DBVersion for u32 {
    #[inline]
    fn into_idb_open_request(
        self,
        factory: &DBFactory,
        name: &str,
    ) -> Result<web_sys::IdbOpenDbRequest, JsValue> {
        factory.as_sys().open_with_u32(name, self)
    }
}

#[sealed]
impl DBVersion for f64 {
    #[inline]
    fn into_idb_open_request(
        self,
        factory: &DBFactory,
        name: &str,
    ) -> Result<web_sys::IdbOpenDbRequest, JsValue> {
        factory.as_sys().open_with_f64(name, self)
    }
}

macro_rules! db_version_alias {
    ($([$src: ty, $as: ty]),+ $(,)?) => {
        $(
            #[::sealed::sealed]
            impl DBVersion for $src {
                fn into_idb_open_request(self, factory: &$crate::DBFactory, name: &str) -> Result<::web_sys::IdbOpenDbRequest, ::wasm_bindgen::JsValue> {
                    <$as as DBVersion>::into_idb_open_request(self.into(), factory, name)
                }
            }
        )+
    };
    (non-0: $($ty: ty),+ $(,)?) => {
        $(
            #[::sealed::sealed]
            impl DBVersion for ::std::num::NonZero<$ty> {
                #[inline]
                fn into_idb_open_request(self, factory: &$crate::DBFactory, name: &str) -> Result<::web_sys::IdbOpenDbRequest, ::wasm_bindgen::JsValue> {
                    self.get().into_idb_open_request(factory, name)
                }
            }
        )+
    };
}

db_version_alias!([u16, u32], [u8, u32], [f32, f64]);
db_version_alias!(non-0: u8, u16, u32);
