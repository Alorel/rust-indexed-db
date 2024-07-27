use cfg_if::cfg_if;
use indexed_db_futures as idb_fut;

#[cfg(feature = "_serialise-deserialise-dyn")]
use wasm_bindgen::prelude::*;

cfg_if! {
    if #[cfg(feature = "serde")] {
        use idb_fut::{
            BuildSerde as BaseBuild,
        };

        #[cfg(feature = "_serialise-deserialise-dyn")]
        use idb_fut::{SerialiseToJs as BaseSerialise, DeserialiseFromJs as BaseDeserialise};
    } else {
        #[cfg(feature = "_serialise-deserialise-dyn")]
        use crate::prelude::{TryToJs as BaseSerialise, TryFromJs as BaseDeserialise};
        use idb_fut::BuildPrimitive as BaseBuild;
    }
}

/// Build with `serde` if the feature's enabled, else build with `primitive`.
pub trait BuildDyn: BaseBuild {
    fn build_dyn(self) -> idb_fut::Result<Self::Fut>;
}

/// `TryToJs`/`SerialiseToJs` based on whether `serde` is enabled.
#[cfg(feature = "_serialise-deserialise-dyn")]
pub trait SerialiseDyn: BaseSerialise {
    fn serialise_dyn(&self) -> idb_fut::Result<JsValue>;
}

/// `TryFromJs`/`DeserialiseFromJs` based on whether `serde` is enabled.
#[cfg(feature = "_serialise-deserialise-dyn")]
pub trait DeserialiseDyn: BaseDeserialise {
    fn deserialise_dyn(js: JsValue) -> idb_fut::Result<Self>
    where
        Self: Sized;
}

impl<T: BaseBuild> BuildDyn for T {
    fn build_dyn(self) -> idb_fut::Result<Self::Fut> {
        cfg_if! {
            if #[cfg(feature = "serde")] {
                BaseBuild::serde(self)
            } else {
                BaseBuild::primitive(self)
            }
        }
    }
}

#[cfg(feature = "_serialise-deserialise-dyn")]
impl<T: BaseSerialise> SerialiseDyn for T {
    fn serialise_dyn(&self) -> idb_fut::Result<JsValue> {
        cfg_if! {
            if #[cfg(feature = "serde")] {
                BaseSerialise::serialise_to_js(self)
            } else {
                BaseSerialise::try_to_js(self)
            }
        }
    }
}

#[cfg(feature = "_serialise-deserialise-dyn")]
impl<T: BaseDeserialise> DeserialiseDyn for T {
    fn deserialise_dyn(js: JsValue) -> idb_fut::Result<Self> {
        cfg_if! {
            if #[cfg(feature = "serde")] {
                BaseDeserialise::deserialise_from_js(js)
            } else {
                BaseDeserialise::from_js(js).map_err(Into::into)
            }
        }
    }
}
