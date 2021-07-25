use crate::idb_key_path::IdbKeyPath;

/// Wrapper for [IdbObjectStore][crate::idb_object_store::IdbObjectStore] optional parameters
#[derive(Debug, Clone)]
pub struct IdbObjectStoreParameters(web_sys::IdbObjectStoreParameters);

impl IdbObjectStoreParameters {
    #[inline]
    pub fn new() -> Self {
        Self::from(web_sys::IdbObjectStoreParameters::new())
    }

    /// Set the auto_increment option
    #[inline]
    pub fn auto_increment(&mut self, val: bool) -> &mut Self {
        self.0.auto_increment(val);
        self
    }

    /// Set the key_path option
    pub fn key_path(&mut self, val: Option<&IdbKeyPath>) -> &mut Self {
        self.0.key_path(val.map(|v| v.as_js_value()));
        self
    }

    /// Get the enclosed web_sys parameters object
    #[inline]
    pub fn as_js_value(&self) -> &web_sys::IdbObjectStoreParameters {
        self.0.as_ref()
    }
}

impl AsRef<web_sys::IdbObjectStoreParameters> for IdbObjectStoreParameters {
    #[inline]
    fn as_ref(&self) -> &web_sys::IdbObjectStoreParameters {
        self.as_js_value()
    }
}

impl Default for IdbObjectStoreParameters {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<web_sys::IdbObjectStoreParameters> for IdbObjectStoreParameters {
    #[inline]
    fn from(raw: web_sys::IdbObjectStoreParameters) -> Self {
        Self(raw)
    }
}
