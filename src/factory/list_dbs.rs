use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Write};

use accessory::Accessors;
use fancy_constructor::new;
use impartial_ord::ImpartialOrd;
use js_sys::Reflect;
use wasm_bindgen::prelude::*;

use crate::error::SimpleValueError;
use crate::future::ListDatabasesFuture;
use crate::internal_utils::SystemRepr;

use super::DBFactory;

/// Database info returned by [`DBFactory::databases`].
#[derive(Accessors, new, Clone, ImpartialOrd, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[new(const_fn)]
pub struct DatabaseDetails {
    /// The name of the database
    #[access(get(ty(&str)))]
    name: String,

    /// The version of the database
    #[access(get(const_fn, cp))]
    version: f64,
}

impl DBFactory {
    /// List the names and versions of all databases.
    #[allow(clippy::missing_errors_doc)]
    pub fn databases(&self) -> crate::Result<ListDatabasesFuture> {
        match self.as_sys().databases()?.dyn_into::<js_sys::Promise>() {
            Ok(promise) => Ok(ListDatabasesFuture::list_databases(promise)),
            Err(jsval) => Err(SimpleValueError::DynCast(jsval).into()),
        }
    }
}

impl Display for DatabaseDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())?;
        f.write_str(" (v")?;
        Display::fmt(&self.version, f)?;
        f.write_char(')')
    }
}

impl TryFrom<JsValue> for DatabaseDetails {
    type Error = crate::error::Error;

    fn try_from(js: JsValue) -> Result<Self, Self::Error> {
        let name = Reflect::get(&js, &JsValue::from_str("name"))?;
        let version = Reflect::get(&js, &JsValue::from_str("version"))?;

        match name.as_string() {
            Some(name) => match version.as_f64() {
                Some(version) => Ok(Self::new(name, version)),
                None => Err(SimpleValueError::NotANumber(version).into()),
            },
            None => Err(SimpleValueError::NotAString(name).into()),
        }
    }
}

impl PartialEq for DatabaseDetails {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && matches!(self.version.total_cmp(&other.version), Ordering::Equal)
    }
}

impl Eq for DatabaseDetails {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for DatabaseDetails {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.name.cmp(&other.name) {
            Ordering::Equal => self.version.total_cmp(&other.version),
            ord => ord,
        }
    }
}
