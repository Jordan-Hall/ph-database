use axum::{routing::post, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(search_faces))
}

async fn search_faces() -> &'static str {
    "TODO: Implement face search (ephemeral, privacy-preserving)"
}
