use aws_sdk_s3::{config::Region, Client as S3Client};
use tokio::fs;

use crate::{config::Config, error::{AppError, AppResult}};

#[derive(Clone)]
pub struct StorageClient {
    client: S3Client,
    bucket: String,
}

impl StorageClient {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        // Configure AWS SDK for MinIO
        let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(&config.minio_endpoint)
            .region(Region::new(config.minio_region.clone()))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                &config.minio_access_key,
                &config.minio_secret_key,
                None,
                None,
                "static",
            ))
            .load()
            .await;

        let s3_client = S3Client::new(&s3_config);

        Ok(Self {
            client: s3_client,
            bucket: config.minio_bucket.clone(),
        })
    }

    /// Upload a file to MinIO
    pub async fn upload_file(&self, key: &str, file_path: &str) -> AppResult<String> {
        let body = fs::read(file_path)
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to read file: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body.into())
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to upload to MinIO: {}", e)))?;

        Ok(format!("{}/{}", self.bucket, key))
    }

    /// Get a presigned URL for downloading
    pub async fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> AppResult<String> {
        // TODO: Implement presigned URL generation
        // For now, return a placeholder
        Ok(format!("http://localhost:9000/{}/{}", self.bucket, key))
    }

    /// Delete a file from MinIO
    pub async fn delete_file(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to delete from MinIO: {}", e)))?;

        Ok(())
    }

    /// Check if a file exists
    pub async fn file_exists(&self, key: &str) -> bool {
        self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .is_ok()
    }
}
