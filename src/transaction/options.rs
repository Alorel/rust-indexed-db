use accessory::Accessors;
use fancy_constructor::new;
use wasm_bindgen::prelude::*;

pub use durability::TransactionDurability;
use internal_macros::generate_with;

use crate::error::Error;

/// Options to pass to
/// [`TransactionBuilder::with_options`](crate::database::TransactionBuilder::with_options).
#[derive(new, Accessors, Debug, Clone, PartialEq)]
pub struct TransactionOptions {
    /// See docs of individual [`TransactionDurability`] variants.
    #[access(get(cp))]
    #[new(val(None))]
    durability: Option<TransactionDurability>,
}

impl TransactionOptions {
    /// Set the durability of the transaction
    #[generate_with]
    #[inline]
    pub fn set_durability(&mut self, durability: TransactionDurability) -> &mut Self {
        self.durability = Some(durability);
        self
    }
}

impl TransactionDurability {
    /// String representation of the durability
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Relaxed => "relaxed",
            _ => "default",
        }
    }
}

impl AsRef<str> for TransactionDurability {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<TransactionOptions> for TransactionOptionsSys {
    type Error = Error;

    fn try_from(this: TransactionOptions) -> Result<Self, Self::Error> {
        let opts = TransactionOptionsSys::new();

        if let Some(durability) = this.durability {
            opts.with_durability(durability).map_err(Into::into)
        } else {
            Ok(opts)
        }
    }
}

impl TransactionOptionsSys {
    pub(super) fn new() -> Self {
        js_sys::Object::new().unchecked_into()
    }

    pub(super) fn with_durability(
        self,
        durability: TransactionDurability,
    ) -> Result<Self, JsValue> {
        js_sys::Reflect::set(self.as_ref(), &"durability".into(), &durability.into())?;
        Ok(self)
    }
}

#[allow(missing_docs)]
mod durability {
    use wasm_bindgen::prelude::*;

    /// Transaction durability option
    #[wasm_bindgen]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TransactionDurability {
        /// The user agent may consider that the transaction has successfully committed only after verifying that all
        /// outstanding changes have been successfully written to a persistent storage medium. This is recommended where
        /// the risk of data loss outweighs the impact of its use on performance and power
        /// (compared to [`relaxed`](crate::database::tx_opts::IdbTransactionDurability::Relaxed)).
        Strict = "strict",

        /// The user agent may consider that the transaction has successfully committed as soon as all outstanding changes
        /// have been written to the operating system, without subsequent verification. This offers better performance
        /// than [`strict`](crate::database::tx_opts::IdbTransactionDurability::Strict), and is recommended for ephemeral
        /// data such as caches or quickly changing records.
        Relaxed = "relaxed",

        /// The user agent should use its default durability behavior for the storage bucket.
        /// This is the default for transactions if not otherwise specified.
        Default = "default",
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    #[wasm_bindgen(extends = js_sys::Object)]
    pub type TransactionOptionsSys;
}
