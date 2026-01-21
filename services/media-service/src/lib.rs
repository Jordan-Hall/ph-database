// Media Service Library - Test Support Module

pub mod config;
pub mod error;
pub mod storage;
pub mod video;
pub mod virus_scan;

use axum::{
    routing::{get, post},
    Router,
};

pub use config::Config;
pub use storage::StorageClient;
pub use video::VideoProcessor;
pub use virus_scan::VirusScanner;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub storage: StorageClient,
    pub scanner: VirusScanner,
}

/// Build application router for testing
pub fn build_app(config: Config, storage: StorageClient) -> Router {
    let scanner = VirusScanner::new(config.clamd_host.clone(), config.clamd_port);
    let app_state = AppState {
        config,
        storage,
        scanner,
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/upload", post(upload_video))
        .with_state(app_state)
}

async fn health_check() -> &'static str {
    "media-service: healthy"
}

async fn upload_video() -> &'static str {
    "upload endpoint"
}
