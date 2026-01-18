use axum::{routing::get, Router};
use crate::AppState;

pub fn public_router() -> Router<AppState> {
    Router::new().route("/entries", get(get_map_entries))
}

async fn get_map_entries() -> &'static str {
    "TODO: Implement map entries endpoint"
}
