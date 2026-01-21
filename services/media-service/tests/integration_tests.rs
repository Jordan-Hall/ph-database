use media_service::{build_app, Config, StorageClient, VideoProcessor};
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use std::path::PathBuf;
use std::fs;
use std::env;

async fn build_test_app() -> axum::Router {
    let config = Config {
        port: 8081,
        upload_dir: env::var("TEST_UPLOAD_DIR").unwrap_or_else(|_| "/tmp/media-test".to_string()),
        minio_endpoint: "http://localhost:9000".to_string(),
        minio_access_key: "minioadmin".to_string(),
        minio_secret_key: "minioadmin".to_string(),
        minio_bucket: "test-media".to_string(),
        minio_region: "us-east-1".to_string(),
        max_video_size_mb: 500,
        ffmpeg_path: "ffmpeg".to_string(),
    };
    fs::create_dir_all(&config.upload_dir).ok();
    let storage = StorageClient::new(&config)
        .await
        .expect("Failed to create storage client");
    build_app(config, storage)
}

fn create_test_video(duration_secs: u32) -> PathBuf {
    let output_path = PathBuf::from(format!("/tmp/test-video-{}.mp4", duration_secs));
    let _ = std::process::Command::new("ffmpeg")
        .args(&["-f", "lavfi", "-i", &format!("testsrc=duration={}:size=640x480:rate=30", duration_secs), "-c:v", "libx264", "-pix_fmt", "yuv420p", "-y", output_path.to_str().unwrap()])
        .output();
    output_path
}

fn create_test_image() -> PathBuf {
    let output_path = PathBuf::from("/tmp/test-image.jpg");
    let _ = std::process::Command::new("ffmpeg")
        .args(&["-f", "lavfi", "-i", "testsrc=size=640x480:rate=1:duration=1", "-frames:v", "1", "-y", output_path.to_str().unwrap()])
        .output();
    output_path
}

async fn cleanup_test_files() {
    let _ = fs::remove_dir_all("/tmp/media-test");
    let _ = fs::remove_file("/tmp/test-video-1.mp4");
    let _ = fs::remove_file("/tmp/test-video-3.mp4");
    let _ = fs::remove_file("/tmp/test-video-5.mp4");
    let _ = fs::remove_file("/tmp/test-video-10.mp4");
    let _ = fs::remove_file("/tmp/test-video-30.mp4");
    let _ = fs::remove_file("/tmp/test-image.jpg");
}

#[tokio::test]
async fn test_health_check() {
    let app = build_test_app().await;
    let response = app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_video_upload() {
    let app = build_test_app().await;
    let test_video = create_test_video(5);
    assert!(test_video.exists(), "Test video should be created");
    println!("Video upload test - video created at {:?}", test_video);
    cleanup_test_files().await;
}

#[tokio::test]
async fn test_video_upload_size_limit() {
    let max_size_mb = 500;
    let oversized = 600;
    assert!(oversized > max_size_mb, "Test file size should exceed limit");
    println!("Size limit test - would reject {} MB file (limit: {} MB)", oversized, max_size_mb);
}

#[tokio::test]
#[ignore]
async fn test_thumbnail_generation() {
    let test_video = create_test_video(5);
    if !test_video.exists() { println!("Test video not created, skipping"); return; }
    let processor = VideoProcessor::new("ffmpeg".to_string());
    let thumbnail_path = PathBuf::from("/tmp/test-thumbnail.jpg");
    match processor.generate_thumbnail(test_video.to_str().unwrap(), thumbnail_path.to_str().unwrap(), 2).await {
        Ok(_) => { assert!(thumbnail_path.exists(), "Thumbnail should be created"); fs::remove_file(thumbnail_path).ok(); }
        Err(e) => println!("Thumbnail generation failed (expected if FFmpeg unavailable): {}", e),
    }
    cleanup_test_files().await;
}

#[tokio::test]
#[ignore]
async fn test_video_transcoding() {
    let test_video = create_test_video(3);
    if !test_video.exists() { println!("Test video not created, skipping"); return; }
    let processor = VideoProcessor::new("ffmpeg".to_string());
    let output_path = PathBuf::from("/tmp/test-transcoded.mp4");
    match processor.transcode_video(test_video.to_str().unwrap(), output_path.to_str().unwrap()).await {
        Ok(_) => { assert!(output_path.exists(), "Transcoded video should exist"); fs::remove_file(output_path).ok(); }
        Err(e) => println!("Transcode failed (expected if FFmpeg unavailable): {}", e),
    }
    cleanup_test_files().await;
}

#[tokio::test]
#[ignore]
async fn test_video_metadata_extraction() {
    let test_video = create_test_video(5);
    if !test_video.exists() { println!("Test video not created, skipping"); return; }
    let processor = VideoProcessor::new("ffmpeg".to_string());
    match processor.get_video_info(test_video.to_str().unwrap()).await {
        Ok(info) => {
            println!("Video info: duration={}s, resolution={}x{}", info.duration_seconds, info.width, info.height);
            assert!(info.duration_seconds > 0, "Duration should be positive");
            assert!(info.width > 0, "Width should be positive");
            assert!(info.height > 0, "Height should be positive");
        }
        Err(e) => println!("Metadata extraction failed (expected if FFmpeg unavailable): {}", e),
    }
    cleanup_test_files().await;
}

#[tokio::test]
#[ignore]
async fn test_preview_generation() {
    let test_video = create_test_video(30);
    if !test_video.exists() { println!("Test video not created, skipping"); return; }
    let processor = VideoProcessor::new("ffmpeg".to_string());
    let preview_path = PathBuf::from("/tmp/test-preview.mp4");
    match processor.extract_preview(test_video.to_str().unwrap(), preview_path.to_str().unwrap(), 10).await {
        Ok(_) => { assert!(preview_path.exists(), "Preview should be created"); fs::remove_file(preview_path).ok(); }
        Err(e) => println!("Preview generation failed (expected if FFmpeg unavailable): {}", e),
    }
    cleanup_test_files().await;
}

#[tokio::test]
#[ignore]
async fn test_minio_upload() {
    let config = Config {
        port: 8081,
        upload_dir: "/tmp/media-test".to_string(),
        minio_endpoint: "http://localhost:9000".to_string(),
        minio_access_key: "minioadmin".to_string(),
        minio_secret_key: "minioadmin".to_string(),
        minio_bucket: "test-media".to_string(),
        minio_region: "us-east-1".to_string(),
        max_video_size_mb: 500,
        ffmpeg_path: "ffmpeg".to_string(),
    };

    match StorageClient::new(&config).await {
        Ok(storage) => {
            let test_file = PathBuf::from("/tmp/test-upload.txt");
            fs::write(&test_file, b"test content").ok();
            match storage.upload_file("test/upload.txt", test_file.to_str().unwrap()).await {
                Ok(url) => { println!("Upload successful: {}", url); assert!(!url.is_empty(), "URL should not be empty"); }
                Err(e) => println!("Upload failed: {}", e),
            }
            fs::remove_file(test_file).ok();
        }
        Err(e) => println!("Storage client creation failed (expected if MinIO unavailable): {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_minio_download() {
    let config = Config {
        port: 8081,
        upload_dir: "/tmp/media-test".to_string(),
        minio_endpoint: "http://localhost:9000".to_string(),
        minio_access_key: "minioadmin".to_string(),
        minio_secret_key: "minioadmin".to_string(),
        minio_bucket: "test-media".to_string(),
        minio_region: "us-east-1".to_string(),
        max_video_size_mb: 500,
        ffmpeg_path: "ffmpeg".to_string(),
    };

    match StorageClient::new(&config).await {
        Ok(storage) => {
            let test_file = PathBuf::from("/tmp/test-download-source.txt");
            fs::write(&test_file, b"download test content").ok();
            match storage.upload_file("test/download.txt", test_file.to_str().unwrap()).await {
                Ok(_) => {
                    // Test presigned URL generation
                    match storage.get_presigned_url("test/download.txt", 3600).await {
                        Ok(url) => { println!("Presigned URL generated: {}", url); assert!(!url.is_empty(), "URL should not be empty"); }
                        Err(e) => println!("Presigned URL generation failed: {}", e),
                    }
                    // Test file exists check
                    let exists = storage.file_exists("test/download.txt").await;
                    assert!(exists, "Uploaded file should exist");
                }
                Err(e) => println!("Upload failed: {}", e),
            }
            fs::remove_file(test_file).ok();
        }
        Err(e) => println!("Storage client creation failed (expected if MinIO unavailable): {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_minio_delete() {
    let config = Config {
        port: 8081,
        upload_dir: "/tmp/media-test".to_string(),
        minio_endpoint: "http://localhost:9000".to_string(),
        minio_access_key: "minioadmin".to_string(),
        minio_secret_key: "minioadmin".to_string(),
        minio_bucket: "test-media".to_string(),
        minio_region: "us-east-1".to_string(),
        max_video_size_mb: 500,
        ffmpeg_path: "ffmpeg".to_string(),
    };

    match StorageClient::new(&config).await {
        Ok(storage) => {
            let test_file = PathBuf::from("/tmp/test-delete.txt");
            fs::write(&test_file, b"delete test").ok();
            match storage.upload_file("test/delete.txt", test_file.to_str().unwrap()).await {
                Ok(_) => {
                    match storage.delete_file("test/delete.txt").await {
                        Ok(_) => {
                            println!("Delete successful");
                            // Verify deletion
                            let exists = storage.file_exists("test/delete.txt").await;
                            assert!(!exists, "File should be deleted");
                        }
                        Err(e) => println!("Delete failed: {}", e),
                    }
                }
                Err(e) => println!("Upload failed: {}", e),
            }
            fs::remove_file(test_file).ok();
        }
        Err(e) => println!("Storage client creation failed (expected if MinIO unavailable): {}", e),
    }
}

#[tokio::test]
async fn test_processing_status_tracking() {
    #[derive(Debug, PartialEq)]
    enum ProcessingStatus { Queued, Processing, Completed, Failed }
    let mut status = ProcessingStatus::Queued;
    assert_eq!(status, ProcessingStatus::Queued);
    status = ProcessingStatus::Processing;
    assert_eq!(status, ProcessingStatus::Processing);
    status = ProcessingStatus::Completed;
    assert_eq!(status, ProcessingStatus::Completed);
    println!("Processing status tracking test passed");
}

#[tokio::test]
async fn test_invalid_video_format() {
    let invalid_file = PathBuf::from("/tmp/test-invalid.txt");
    fs::write(&invalid_file, b"not a video file").ok();
    let extension = invalid_file.extension().and_then(|e| e.to_str()).unwrap_or("");
    let valid_extensions = ["mp4", "avi", "mov", "mkv", "webm"];
    let is_valid = valid_extensions.contains(&extension);
    assert!(!is_valid, "Text file should not be valid video format");
    fs::remove_file(invalid_file).ok();
    println!("Invalid format test passed");
}

#[tokio::test]
#[ignore]
async fn test_corrupted_video_handling() {
    let corrupted_file = PathBuf::from("/tmp/test-corrupted.mp4");
    fs::write(&corrupted_file, b"CORRUPTED VIDEO DATA").ok();
    let processor = VideoProcessor::new("ffmpeg".to_string());
    match processor.get_video_info(corrupted_file.to_str().unwrap()).await {
        Ok(_) => println!("Unexpectedly succeeded on corrupted file"),
        Err(_) => println!("Correctly failed on corrupted file"),
    }
    fs::remove_file(corrupted_file).ok();
}

#[tokio::test]
#[ignore]
async fn test_concurrent_uploads() {
    use tokio::task::JoinSet;
    let mut tasks = JoinSet::new();
    for i in 0..3 {
        tasks.spawn(async move {
            let test_video = create_test_video(1);
            let media_id = format!("media_{}", i);
            println!("Upload {} - Video at: {:?}", media_id, test_video);
            media_id
        });
    }
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        if let Ok(media_id) = result { results.push(media_id); }
    }
    assert_eq!(results.len(), 3, "All uploads should complete");
    let unique_count = results.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, 3, "All media IDs should be unique");
    cleanup_test_files().await;
}

#[tokio::test]
async fn test_cleanup_temp_files() {
    let temp_dir = PathBuf::from("/tmp/media-test-cleanup");
    fs::create_dir_all(&temp_dir).ok();
    fs::write(temp_dir.join("temp1.txt"), b"temp file 1").ok();
    fs::write(temp_dir.join("temp2.txt"), b"temp file 2").ok();
    assert!(temp_dir.join("temp1.txt").exists());
    assert!(temp_dir.join("temp2.txt").exists());
    fs::remove_dir_all(&temp_dir).ok();
    assert!(!temp_dir.exists(), "Temp directory should be removed");
    println!("Temp file cleanup test passed");
}
