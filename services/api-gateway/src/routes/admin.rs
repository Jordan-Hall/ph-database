use axum::{routing::get, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/tenants", get(list_tenants))
}

async fn list_users() -> &'static str {
    "TODO: Implement user management"
}

async fn list_tenants() -> &'static str {
    "TODO: Implement tenant management"
}
