// Services module for business logic
// TODO: Implement service layer for each domain

pub mod reports;
pub mod alerts;
pub mod review;
pub mod surrealdb_features;
pub mod audit_log;

// Re-export for convenience
pub use reports::*;
pub use alerts::*;
pub use review::*;
pub use surrealdb_features::*;
pub use audit_log::*;
