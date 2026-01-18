use axum::{routing::get, Router};
use crate::AppState;

pub fn public_router() -> Router<AppState> {
    Router::new().route("/:slug", get(get_item))
}

async fn get_item() -> &'static str {
    "TODO: Implement publishable item endpoint"
}
