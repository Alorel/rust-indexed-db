use crate::internal_utils::StructName;
use std::fmt::{Debug, Formatter};

/// Fired for [`upgradeneeded`](https://developer.mozilla.org/en-US/docs/Web/API/IDBOpenDBRequest#events),
/// [`blocked`](https://developer.mozilla.org/en-US/docs/Web/API/IDBOpenDBRequest#events)
/// & [`versionchange`](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/versionchange_event) events.
#[allow(clippy::too_long_first_doc_paragraph)]
#[derive(StructName, Clone)]
pub struct VersionChangeEvent(web_sys::IdbVersionChangeEvent);

impl VersionChangeEvent {
    pub(crate) fn new(event: web_sys::IdbVersionChangeEvent) -> Self {
        Self(event)
    }

    /// The old version number of the database.
    #[inline]
    #[must_use]
    pub fn old_version(&self) -> f64 {
        self.0.old_version()
    }

    /// The new version number of the database. Will be `None` if the database is being deleted.
    #[inline]
    #[must_use]
    pub fn new_version(&self) -> Option<f64> {
        self.0.new_version()
    }
}

impl Debug for VersionChangeEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(Self::TYPE_NAME)
            .field("old_version", &self.old_version())
            .field("new_version", &self.new_version())
            .finish()
    }
}
