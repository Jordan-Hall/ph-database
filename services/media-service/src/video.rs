use std::process::Stdio;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

pub struct VideoProcessor {
    ffmpeg_path: String,
}

impl VideoProcessor {
    pub fn new(ffmpeg_path: String) -> Self {
        Self { ffmpeg_path }
    }

    /// Generate thumbnail from video at specific timestamp
    pub async fn generate_thumbnail(
        &self,
        input_path: &str,
        output_path: &str,
        timestamp_seconds: u32,
    ) -> AppResult<()> {
        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                input_path,
                "-ss",
                &timestamp_seconds.to_string(),
                "-vframes",
                "1",
                "-q:v",
                "2",
                "-y",
                output_path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                AppError::ProcessingError(format!("Failed to execute ffmpeg: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ProcessingError(format!(
                "ffmpeg thumbnail generation failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Generate multiple thumbnails at intervals (e.g., every 5 seconds)
    pub async fn generate_thumbnail_strip(
        &self,
        input_path: &str,
        output_dir: &str,
        interval_seconds: u32,
    ) -> AppResult<Vec<String>> {
        // Get video duration first
        let duration = self.get_video_duration(input_path).await?;
        let mut thumbnails = Vec::new();

        for timestamp in (0..duration).step_by(interval_seconds as usize) {
            let output_path = format!("{}/thumb_{:04}.jpg", output_dir, timestamp);
            self.generate_thumbnail(input_path, &output_path, timestamp as u32)
                .await?;
            thumbnails.push(output_path);
        }

        Ok(thumbnails)
    }

    /// Get video duration in seconds
    pub async fn get_video_duration(&self, input_path: &str) -> AppResult<u32> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                input_path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                AppError::ProcessingError(format!("Failed to execute ffprobe: {}", e))
            })?;

        if !output.status.success() {
            return Err(AppError::ProcessingError(
                "Failed to get video duration".to_string(),
            ));
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration: f64 = duration_str.trim().parse().map_err(|_| {
            AppError::ProcessingError("Failed to parse video duration".to_string())
        })?;

        Ok(duration as u32)
    }

    /// Transcode video to web-friendly format (H.264/AAC in MP4)
    pub async fn transcode_video(
        &self,
        input_path: &str,
        output_path: &str,
    ) -> AppResult<()> {
        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                input_path,
                "-c:v",
                "libx264",
                "-preset",
                "medium",
                "-crf",
                "23",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-movflags",
                "+faststart",
                "-y",
                output_path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                AppError::ProcessingError(format!("Failed to execute ffmpeg: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ProcessingError(format!(
                "ffmpeg transcoding failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Extract a preview clip (e.g., first 15 seconds)
    pub async fn extract_preview(
        &self,
        input_path: &str,
        output_path: &str,
        duration_seconds: u32,
    ) -> AppResult<()> {
        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                input_path,
                "-t",
                &duration_seconds.to_string(),
                "-c",
                "copy",
                "-y",
                output_path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                AppError::ProcessingError(format!("Failed to execute ffmpeg: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ProcessingError(format!(
                "ffmpeg preview extraction failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Get video metadata (resolution, codec, bitrate, etc.)
    pub async fn get_video_info(&self, input_path: &str) -> AppResult<VideoInfo> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height,codec_name,bit_rate,duration",
                "-of",
                "json",
                input_path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                AppError::ProcessingError(format!("Failed to execute ffprobe: {}", e))
            })?;

        if !output.status.success() {
            return Err(AppError::ProcessingError(
                "Failed to get video info".to_string(),
            ));
        }

        // Parse JSON output
        let info_json = String::from_utf8_lossy(&output.stdout);
        // TODO: Parse JSON properly
        // For now, return placeholder
        Ok(VideoInfo {
            width: 1920,
            height: 1080,
            codec: "h264".to_string(),
            duration_seconds: 120,
            bitrate: 2000000,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub duration_seconds: u32,
    pub bitrate: u64,
}
