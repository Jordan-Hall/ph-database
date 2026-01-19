// Legacy pages
pub mod home;
pub mod add_record;
pub mod view_record;

// Auth pages
pub mod login;
pub mod register;

// Dashboard and reports
pub mod dashboard;
pub mod reports;
pub mod new_report;
pub mod report_detail;

// Alerts
pub mod public_alerts;
pub mod manage_alerts;
pub mod new_alert;

// Stories
pub mod stories;
pub mod submit_story;

// Map
pub mod map_view;

// Published items
pub mod published_item;

// Profile
pub mod profile;

// Admin
pub mod admin_dashboard;
pub mod admin_users;
pub mod admin_tenants;
pub mod admin_review;

// Re-exports
pub use home::*;
pub use add_record::*;
pub use view_record::*;
pub use login::*;
pub use register::*;
pub use dashboard::*;
pub use reports::*;
pub use new_report::*;
pub use report_detail::*;
pub use public_alerts::*;
pub use manage_alerts::*;
pub use new_alert::*;
pub use stories::*;
pub use submit_story::*;
pub use map_view::*;
pub use published_item::*;
pub use profile::*;
pub use admin_dashboard::*;
pub use admin_users::*;
pub use admin_tenants::*;
pub use admin_review::*;
