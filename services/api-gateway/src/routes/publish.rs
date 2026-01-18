use axum::{routing::post, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/:item_id", post(publish_item))
}

async fn publish_item() -> &'static str {
    "TODO: Implement publish endpoint"
}
