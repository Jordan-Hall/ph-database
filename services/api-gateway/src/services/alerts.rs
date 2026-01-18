// Alerts service - Business logic for missing person alerts
// TODO: Implement full alerts service

use crate::db::Database;

pub struct AlertsService {
    db: Database,
}

impl AlertsService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // TODO: Implement methods for:
    // - create_alert
    // - get_active_alerts
    // - resolve_alert
    // - expire_alerts (background job)
}
