use axum::{routing::get, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/queue", get(get_review_queue))
        .route("/:id", get(get_review_detail))
}

async fn get_review_queue() -> &'static str {
    "TODO: Implement review queue"
}

async fn get_review_detail() -> &'static str {
    "TODO: Implement review detail"
}
