use crate::error::{AppError, AppResult};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Job types for background processing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JobType {
    /// Generate thumbnail for video
    GenerateThumbnail {
        media_id: String,
        video_path: String,
        timestamp_seconds: u32,
    },
    /// Transcode video to web-friendly format
    TranscodeVideo {
        media_id: String,
        input_path: String,
        output_path: String,
    },
    /// Extract video metadata
    ExtractMetadata {
        media_id: String,
        video_path: String,
    },
    /// Generate preview clip
    GeneratePreview {
        media_id: String,
        input_path: String,
        output_path: String,
        duration_seconds: u32,
    },
    /// Upload processed file to MinIO
    UploadToStorage {
        media_id: String,
        local_path: String,
        storage_key: String,
    },
    /// Cleanup temporary files
    CleanupTempFiles {
        file_paths: Vec<String>,
    },
}

/// Job with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub attempts: u32,
    pub max_attempts: u32,
}

impl Job {
    pub fn new(job_type: JobType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_type,
            created_at: chrono::Utc::now(),
            attempts: 0,
            max_attempts: 3,
        }
    }

    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }
}

/// Job status for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed { error: String },
}

/// Job queue for managing background tasks
#[derive(Clone)]
pub struct JobQueue {
    redis: redis::aio::ConnectionManager,
    queue_name: String,
    processing_queue: String,
    failed_queue: String,
}

impl JobQueue {
    /// Create new job queue
    pub async fn new(redis_url: &str, queue_name: &str) -> AppResult<Self> {
        let client = redis::Client::open(redis_url).map_err(|e| {
            AppError::StorageError(format!("Failed to connect to Redis: {}", e))
        })?;

        let redis = client.get_connection_manager().await.map_err(|e| {
            AppError::StorageError(format!("Failed to get Redis connection: {}", e))
        })?;

        Ok(Self {
            redis,
            queue_name: format!("queue:{}", queue_name),
            processing_queue: format!("queue:{}:processing", queue_name),
            failed_queue: format!("queue:{}:failed", queue_name),
        })
    }

    /// Enqueue a job
    pub async fn enqueue(&mut self, job: Job) -> AppResult<()> {
        let job_json = serde_json::to_string(&job).map_err(|e| {
            AppError::ProcessingError(format!("Failed to serialize job: {}", e))
        })?;

        self.redis
            .rpush::<_, _, ()>(&self.queue_name, job_json)
            .await
            .map_err(|e| {
                AppError::StorageError(format!("Failed to enqueue job: {}", e))
            })?;

        tracing::info!("Enqueued job {} (type: {:?})", job.id, job.job_type);
        Ok(())
    }

    /// Dequeue a job (blocking with timeout)
    pub async fn dequeue(&mut self, timeout_seconds: u64) -> AppResult<Option<Job>> {
        let result: Option<(String, String)> = self
            .redis
            .blmove(
                &self.queue_name,
                &self.processing_queue,
                redis::Direction::Left,
                redis::Direction::Right,
                timeout_seconds as f64,
            )
            .await
            .map_err(|e| {
                AppError::StorageError(format!("Failed to dequeue job: {}", e))
            })?;

        match result {
            Some((_, job_json)) => {
                let mut job: Job = serde_json::from_str(&job_json).map_err(|e| {
                    AppError::ProcessingError(format!("Failed to deserialize job: {}", e))
                })?;

                job.attempts += 1;
                tracing::debug!("Dequeued job {} (attempt {})", job.id, job.attempts);

                Ok(Some(job))
            }
            None => Ok(None), // Timeout, no jobs available
        }
    }

    /// Mark job as completed
    pub async fn complete(&mut self, job: &Job) -> AppResult<()> {
        // Remove from processing queue
        let job_json = serde_json::to_string(job).map_err(|e| {
            AppError::ProcessingError(format!("Failed to serialize job: {}", e))
        })?;

        self.redis
            .lrem::<_, _, ()>(&self.processing_queue, 1, &job_json)
            .await
            .map_err(|e| {
                AppError::StorageError(format!("Failed to remove job from processing: {}", e))
            })?;

        tracing::info!("Completed job {}", job.id);
        Ok(())
    }

    /// Mark job as failed and optionally retry
    pub async fn fail(&mut self, mut job: Job, error: String) -> AppResult<()> {
        let job_json = serde_json::to_string(&job).map_err(|e| {
            AppError::ProcessingError(format!("Failed to serialize job: {}", e))
        })?;

        // Remove from processing queue
        self.redis
            .lrem::<_, _, ()>(&self.processing_queue, 1, &job_json)
            .await
            .map_err(|e| {
                AppError::StorageError(format!("Failed to remove job from processing: {}", e))
            })?;

        if job.can_retry() {
            // Re-queue for retry
            tracing::warn!(
                "Job {} failed (attempt {}/{}), re-queueing: {}",
                job.id,
                job.attempts,
                job.max_attempts,
                error
            );

            self.enqueue(job).await?;
        } else {
            // Move to failed queue
            tracing::error!(
                "Job {} failed permanently after {} attempts: {}",
                job.id,
                job.attempts,
                error
            );

            let failed_job_json = serde_json::to_string(&(job, error)).map_err(|e| {
                AppError::ProcessingError(format!("Failed to serialize failed job: {}", e))
            })?;

            self.redis
                .rpush::<_, _, ()>(&self.failed_queue, failed_job_json)
                .await
                .map_err(|e| {
                    AppError::StorageError(format!("Failed to add to failed queue: {}", e))
                })?;
        }

        Ok(())
    }

    /// Get queue statistics
    pub async fn stats(&mut self) -> AppResult<QueueStats> {
        let queued: usize = self
            .redis
            .llen(&self.queue_name)
            .await
            .unwrap_or(0);

        let processing: usize = self
            .redis
            .llen(&self.processing_queue)
            .await
            .unwrap_or(0);

        let failed: usize = self
            .redis
            .llen(&self.failed_queue)
            .await
            .unwrap_or(0);

        Ok(QueueStats {
            queued,
            processing,
            failed,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueStats {
    pub queued: usize,
    pub processing: usize,
    pub failed: usize,
}

/// Worker for processing jobs
pub struct Worker {
    queue: JobQueue,
    shutdown: tokio::sync::watch::Receiver<bool>,
}

impl Worker {
    pub fn new(queue: JobQueue, shutdown: tokio::sync::watch::Receiver<bool>) -> Self {
        Self { queue, shutdown }
    }

    /// Start worker loop
    pub async fn start<F, Fut>(mut self, process_fn: F)
    where
        F: Fn(Job) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send,
    {
        tracing::info!("Worker started");

        loop {
            // Check for shutdown signal
            if *self.shutdown.borrow() {
                tracing::info!("Worker received shutdown signal");
                break;
            }

            // Dequeue job with 5-second timeout
            match self.queue.dequeue(5).await {
                Ok(Some(job)) => {
                    let job_id = job.id.clone();
                    tracing::info!("Processing job {}", job_id);

                    // Process job
                    match process_fn(job.clone()).await {
                        Ok(_) => {
                            if let Err(e) = self.queue.complete(&job).await {
                                tracing::error!("Failed to mark job {} as complete: {}", job_id, e);
                            }
                        }
                        Err(e) => {
                            if let Err(err) = self.queue.fail(job, e).await {
                                tracing::error!("Failed to handle job {} failure: {}", job_id, err);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Timeout, no jobs available - this is normal
                    sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    tracing::error!("Error dequeuing job: {}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        tracing::info!("Worker stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis running
    async fn test_enqueue_dequeue() {
        let mut queue = JobQueue::new("redis://localhost:6379", "test")
            .await
            .expect("Failed to create queue");

        let job = Job::new(JobType::GenerateThumbnail {
            media_id: "test123".to_string(),
            video_path: "/tmp/video.mp4".to_string(),
            timestamp_seconds: 5,
        });

        queue.enqueue(job.clone()).await.expect("Failed to enqueue");

        let dequeued = queue
            .dequeue(1)
            .await
            .expect("Failed to dequeue")
            .expect("No job dequeued");

        assert_eq!(dequeued.id, job.id);
        assert_eq!(dequeued.attempts, 1);
    }

    #[tokio::test]
    #[ignore] // Requires Redis running
    async fn test_retry_logic() {
        let mut queue = JobQueue::new("redis://localhost:6379", "test-retry")
            .await
            .expect("Failed to create queue");

        let job = Job::new(JobType::TranscodeVideo {
            media_id: "test456".to_string(),
            input_path: "/tmp/input.mp4".to_string(),
            output_path: "/tmp/output.mp4".to_string(),
        });

        queue.enqueue(job.clone()).await.expect("Failed to enqueue");

        // First attempt
        let mut job1 = queue.dequeue(1).await.unwrap().unwrap();
        assert_eq!(job1.attempts, 1);
        assert!(job1.can_retry());

        // Fail and retry
        queue
            .fail(job1.clone(), "Test error".to_string())
            .await
            .unwrap();

        // Second attempt
        let job2 = queue.dequeue(1).await.unwrap().unwrap();
        assert_eq!(job2.attempts, 2);
        assert!(job2.can_retry());
    }
}
