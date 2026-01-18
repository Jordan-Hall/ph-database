use axum::{routing::{get, post}, Router};
use crate::AppState;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_stories))
        .route("/submit", post(submit_story))
}

async fn get_stories() -> &'static str {
    "TODO: Implement stories listing"
}

async fn submit_story() -> &'static str {
    "TODO: Implement story submission"
}
