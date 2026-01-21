use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::fs;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod job_queue;
mod storage;
mod video;
mod virus_scan;

use config::Config;
use error::{AppError, AppResult};
use job_queue::{JobQueue, JobType, Job};
use storage::StorageClient;
use virus_scan::VirusScanner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "media_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize Prometheus metrics exporter
    let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");
    tracing::info!("Metrics exporter initialized");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Media Service starting on port {}", config.port);

    // Initialize storage client (MinIO)
    let storage = StorageClient::new(&config).await?;
    tracing::info!("MinIO storage initialized");

    // Initialize virus scanner
    let scanner = VirusScanner::new(config.clamd_host.clone(), config.clamd_port);
    if scanner.ping().await {
        if let Ok(version) = scanner.version().await {
            tracing::info!("ClamAV virus scanner initialized: {}", version);
        } else {
            tracing::info!("ClamAV daemon connected");
        }
    } else {
        tracing::warn!(
            "ClamAV daemon not available at {}:{} - virus scanning will be skipped",
            config.clamd_host,
            config.clamd_port
        );
    }

    // Ensure required directories exist
    fs::create_dir_all(&config.upload_dir).await?;
    tracing::info!("Upload directory ready: {}", config.upload_dir);

    fs::create_dir_all(&config.quarantine_dir).await?;
    tracing::info!("Quarantine directory ready: {}", config.quarantine_dir);

    // Initialize job queue
    let job_queue = JobQueue::new(&config.redis_url, "media-processing")
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize job queue: {}", e))?;
    tracing::info!("Job queue initialized");

    // Build application state
    let app_state = AppState {
        config: config.clone(),
        storage,
        scanner,
        job_queue: job_queue.clone(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(move || async move {
            metrics_handle.render()
        }))
        .route("/upload", post(upload_video))
        .route("/media/:id", get(get_media))
        .route("/media/:id/thumbnail", get(get_thumbnail))
        .route("/media/:id/status", get(get_processing_status))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024 * 1024)) // 5GB max upload
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Media Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub storage: StorageClient,
    pub scanner: VirusScanner,
    pub job_queue: JobQueue,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    virus_scanner: String,
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let virus_scanner_status = if state.scanner.ping().await {
        "available".to_string()
    } else {
        "unavailable".to_string()
    };

    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "media-service".to_string(),
        virus_scanner: virus_scanner_status,
    })
}

#[derive(Serialize)]
struct UploadResponse {
    media_id: String,
    status: String,
    message: String,
}

async fn upload_video(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadResponse>> {
    let media_id = uuid::Uuid::new_v4().to_string();
    let mut file_path = None;
    let mut original_filename = String::from("unknown");

    // Process multipart upload
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        AppError::InvalidInput("Failed to read upload data".to_string())
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "video" {
            original_filename = field
                .file_name()
                .unwrap_or("video.mp4")
                .to_string();

            let data = field.bytes().await.map_err(|e| {
                tracing::error!("Failed to read file bytes: {}", e);
                AppError::InvalidInput("Failed to read file data".to_string())
            })?;

            // Save to temporary directory
            let temp_path = format!("{}/{}.tmp", state.config.upload_dir, media_id);
            fs::write(&temp_path, &data).await.map_err(|e| {
                tracing::error!("Failed to write temp file: {}", e);
                AppError::StorageError("Failed to save upload".to_string())
            })?;

            file_path = Some(temp_path);
            tracing::info!(
                "Received upload: {} ({} bytes)",
                original_filename,
                data.len()
            );
        }
    }

    let file_path = file_path.ok_or_else(|| {
        AppError::InvalidInput("No video file provided in upload".to_string())
    })?;

    // Virus scanning
    if state.scanner.ping().await {
        tracing::info!("Scanning {} for viruses...", media_id);
        let scan_result = state.scanner.scan_file(std::path::Path::new(&file_path)).await?;

        if scan_result.is_infected {
            // Move infected file to quarantine
            let quarantine_path = format!("{}/{}_INFECTED.tmp", state.config.quarantine_dir, media_id);
            fs::rename(&file_path, &quarantine_path).await.map_err(|e| {
                tracing::error!("Failed to quarantine infected file: {}", e);
                AppError::StorageError("Failed to quarantine infected file".to_string())
            })?;

            let virus_name = scan_result.virus_name.as_deref().unwrap_or("Unknown");
            tracing::error!(
                "VIRUS DETECTED: {} in upload {} ({}), quarantined to {}",
                virus_name,
                media_id,
                original_filename,
                quarantine_path
            );

            return Err(AppError::InvalidInput(format!(
                "File rejected: virus detected ({}). Upload has been quarantined.",
                virus_name
            )));
        }

        tracing::info!(
            "Virus scan passed for {} in {}ms",
            media_id,
            scan_result.scan_time_ms
        );
    } else {
        tracing::warn!("Virus scanner unavailable, skipping scan for {}", media_id);
    }

    // Queue jobs for processing
    let thumbnail_job = Job::new(JobType::GenerateThumbnail {
        media_id: media_id.clone(),
        video_path: file_path.clone(),
        timestamp_seconds: 5,
    });

    let metadata_job = Job::new(JobType::ExtractMetadata {
        media_id: media_id.clone(),
        video_path: file_path.clone(),
    });

    let transcode_job = Job::new(JobType::TranscodeVideo {
        media_id: media_id.clone(),
        input_path: file_path.clone(),
        output_path: format!("{}/{}_transcoded.mp4", state.config.upload_dir, media_id),
    });

    state.job_queue.clone().enqueue(thumbnail_job).await.map_err(|e| {
        tracing::error!("Failed to enqueue thumbnail job: {}", e);
        AppError::ProcessingError(format!("Failed to queue processing: {}", e))
    })?;

    state.job_queue.clone().enqueue(metadata_job).await.map_err(|e| {
        tracing::error!("Failed to enqueue metadata job: {}", e);
        AppError::ProcessingError(format!("Failed to queue processing: {}", e))
    })?;

    state.job_queue.clone().enqueue(transcode_job).await.map_err(|e| {
        tracing::error!("Failed to enqueue transcode job: {}", e);
        AppError::ProcessingError(format!("Failed to queue processing: {}", e))
    })?;

    tracing::info!("Upload received: {} -> {}, queued 3 processing jobs", original_filename, media_id);

    Ok(Json(UploadResponse {
        media_id: media_id.clone(),
        status: "queued".to_string(),
        message: format!("Upload successful (virus scan passed), {} processing jobs queued", 3),
    }))
}

#[derive(Serialize)]
struct MediaResponse {
    media_id: String,
    url: String,
    mime_type: String,
}

async fn get_media(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<MediaResponse>> {
    // TODO: Implement media retrieval from MinIO
    Ok(Json(MediaResponse {
        media_id: id.clone(),
        url: format!("/media/{}/download", id),
        mime_type: "video/mp4".to_string(),
    }))
}

async fn get_thumbnail(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Implement thumbnail retrieval
    Ok(Json(serde_json::json!({
        "media_id": id,
        "thumbnail_url": format!("/media/{}/thumbnail.jpg", id)
    })))
}

#[derive(Serialize)]
struct ProcessingStatus {
    media_id: String,
    status: String,
    progress: u8,
    stages: Vec<ProcessingStage>,
}

#[derive(Serialize)]
struct ProcessingStage {
    name: String,
    status: String,
    completed_at: Option<String>,
}

async fn get_processing_status(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ProcessingStatus>> {
    // TODO: Implement status tracking from database or Redis
    Ok(Json(ProcessingStatus {
        media_id: id.clone(),
        status: "processing".to_string(),
        progress: 50,
        stages: vec![
            ProcessingStage {
                name: "Upload".to_string(),
                status: "completed".to_string(),
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
            },
            ProcessingStage {
                name: "Virus Scan".to_string(),
                status: "in_progress".to_string(),
                completed_at: None,
            },
            ProcessingStage {
                name: "Thumbnail Generation".to_string(),
                status: "pending".to_string(),
                completed_at: None,
            },
            ProcessingStage {
                name: "Transcoding".to_string(),
                status: "pending".to_string(),
                completed_at: None,
            },
        ],
    }))
}
