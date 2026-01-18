use axum::{routing::{get, post}, Router};
use crate::AppState;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_report))
        .route("/:id", get(get_report))
        .route("/:id/evidence", post(upload_evidence))
}

async fn create_report() -> &'static str {
    "TODO: Implement report creation"
}

async fn get_report() -> &'static str {
    "TODO: Implement report retrieval"
}

async fn upload_evidence() -> &'static str {
    "TODO: Implement evidence upload"
}
