use axum::{routing::{get, post, delete}, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/validate", post(validate))
        .route("/usage", get(get_usage))
        .route("/keys", post(create_key))
        .route("/keys/:id", delete(delete_key))
}

async fn validate() -> &'static str {
    "TODO: Implement business validation endpoint"
}

async fn get_usage() -> &'static str {
    "TODO: Implement usage statistics"
}

async fn create_key() -> &'static str {
    "TODO: Implement API key creation"
}

async fn delete_key() -> &'static str {
    "TODO: Implement API key deletion"
}
