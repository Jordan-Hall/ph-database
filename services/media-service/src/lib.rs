// Media Service Library - Test Support Module

pub mod config;
pub mod error;
pub mod storage;
pub mod video;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub use config::Config;
pub use storage::StorageClient;
pub use video::VideoProcessor;

/// Build application router for testing
pub fn build_app(config: Config, storage: StorageClient) -> Router {
    let shared_storage = Arc::new(RwLock::new(storage));

    Router::new()
        .route("/health", get(health_check))
        .route("/upload", post(upload_video))
        .with_state(shared_storage)
}

async fn health_check() -> &'static str {
    "media-service: healthy"
}

async fn upload_video() -> &'static str {
    "upload endpoint"
}
