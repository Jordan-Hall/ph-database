use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub upload_dir: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_bucket: String,
    pub minio_region: String,
    pub max_video_size_mb: usize,
    pub ffmpeg_path: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Config {
            port: std::env::var("MEDIA_SERVICE_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()?,
            upload_dir: std::env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "/tmp/uploads".to_string()),
            minio_endpoint: std::env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            minio_access_key: std::env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            minio_secret_key: std::env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            minio_bucket: std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "media".to_string()),
            minio_region: std::env::var("MINIO_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            max_video_size_mb: std::env::var("MAX_VIDEO_SIZE_MB")
                .unwrap_or_else(|_| "500".to_string())
                .parse()?,
            ffmpeg_path: std::env::var("FFMPEG_PATH")
                .unwrap_or_else(|_| "ffmpeg".to_string()),
        })
    }
}
