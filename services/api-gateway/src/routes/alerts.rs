use axum::{routing::get, Router};
use crate::AppState;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/active", get(get_active_alerts))
        .route("/:id", get(get_alert))
}

async fn get_active_alerts() -> &'static str {
    "TODO: Implement active alerts endpoint"
}

async fn get_alert() -> &'static str {
    "TODO: Implement alert detail endpoint"
}
