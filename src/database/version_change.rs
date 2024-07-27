use accessory::Accessors;

use super::Database;

/// Fired for [`upgradeneeded`](super::OpenDbRequestBuilder::with_on_upgrade_needed) &
/// [`blocked`](super::OpenDbRequestBuilder::with_on_blocked) events
#[derive(Accessors, Debug, Clone)]
pub struct VersionChangeEvent {
    base: web_sys::IdbVersionChangeEvent,

    /// The database associated with this event
    #[access(get)]
    db: Database,
}

impl VersionChangeEvent {
    #[must_use]
    pub(crate) fn new(event: web_sys::IdbVersionChangeEvent) -> Self {
        Self {
            db: Database::from_event(&event),
            base: event,
        }
    }

    /// The old version number of the database.
    #[inline]
    #[must_use]
    pub fn old_version(&self) -> f64 {
        self.base.old_version()
    }

    /// The new version number of the database.
    #[inline]
    #[must_use]
    pub fn new_version(&self) -> Option<f64> {
        self.base.new_version()
    }

    /// Extract the database associated with this event
    #[inline]
    #[must_use]
    pub fn into_db(self) -> Database {
        self.db
    }
}
