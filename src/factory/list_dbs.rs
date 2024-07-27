use std::fmt::{Display, Formatter, Write};

use accessory::Accessors;
use fancy_constructor::new;
use js_sys::Reflect;
use wasm_bindgen::prelude::*;

use crate::error::SimpleValueError;
use crate::future::{ListDatabasesFuture, MaybeErrored};
use crate::internal_utils::SystemRepr;

use super::DBFactory;

#[derive(Accessors, new, Clone, PartialEq, PartialOrd, Debug)]
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
    pub fn databases(&self) -> MaybeErrored<ListDatabasesFuture> {
        match self.as_sys().databases() {
            Ok(promise_jsval) => match promise_jsval.dyn_into::<js_sys::Promise>() {
                Ok(promise_jsval) => {
                    let fut = ListDatabasesFuture::new(promise_jsval);
                    MaybeErrored::running(fut)
                }
                Err(jsval) => MaybeErrored::errored(SimpleValueError::DynCast(jsval).into()),
            },
            Err(e) => MaybeErrored::errored(e.into()),
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
