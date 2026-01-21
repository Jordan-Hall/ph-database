use crate::error::{AppError, AppResult};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct VirusScanner {
    clamd_host: String,
    clamd_port: u16,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub is_infected: bool,
    pub virus_name: Option<String>,
    pub scan_time_ms: u64,
}

impl VirusScanner {
    pub fn new(clamd_host: String, clamd_port: u16) -> Self {
        Self {
            clamd_host,
            clamd_port,
        }
    }

    /// Scan a file for viruses using ClamAV daemon
    pub async fn scan_file(&self, file_path: &Path) -> AppResult<ScanResult> {
        let start_time = std::time::Instant::now();

        // Read file contents
        let file_data = tokio::fs::read(file_path).await.map_err(|e| {
            tracing::error!("Failed to read file for scanning: {}", e);
            AppError::ProcessingError(format!("Failed to read file: {}", e))
        })?;

        // Connect to ClamAV daemon
        let addr = format!("{}:{}", self.clamd_host, self.clamd_port);
        let mut stream = TcpStream::connect(&addr).await.map_err(|e| {
            tracing::error!("Failed to connect to ClamAV daemon at {}: {}", addr, e);
            AppError::StorageError(format!("Virus scanner unavailable: {}", e))
        })?;

        tracing::debug!("Connected to ClamAV daemon at {}", addr);

        // Send INSTREAM command
        stream
            .write_all(b"zINSTREAM\0")
            .await
            .map_err(|e| {
                tracing::error!("Failed to send INSTREAM command: {}", e);
                AppError::ProcessingError(format!("Scanner communication error: {}", e))
            })?;

        // Send file data in chunks (ClamAV protocol: 4-byte length + data)
        const CHUNK_SIZE: usize = 2048;
        for chunk in file_data.chunks(CHUNK_SIZE) {
            let chunk_len = chunk.len() as u32;
            stream
                .write_all(&chunk_len.to_be_bytes())
                .await
                .map_err(|e| {
                    tracing::error!("Failed to send chunk length: {}", e);
                    AppError::ProcessingError(format!("Scanner communication error: {}", e))
                })?;

            stream.write_all(chunk).await.map_err(|e| {
                tracing::error!("Failed to send chunk data: {}", e);
                AppError::ProcessingError(format!("Scanner communication error: {}", e))
            })?;
        }

        // Send terminator (0-length chunk)
        stream
            .write_all(&[0u8, 0, 0, 0])
            .await
            .map_err(|e| {
                tracing::error!("Failed to send terminator: {}", e);
                AppError::ProcessingError(format!("Scanner communication error: {}", e))
            })?;

        // Read response
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(|e| {
                tracing::error!("Failed to read scan response: {}", e);
                AppError::ProcessingError(format!("Scanner communication error: {}", e))
            })?;

        let response_str = String::from_utf8_lossy(&response);
        let scan_time_ms = start_time.elapsed().as_millis() as u64;

        tracing::debug!("ClamAV response: {}", response_str.trim());

        // Parse response
        // Format: "stream: OK" for clean files
        // Format: "stream: Eicar-Test-Signature FOUND" for infected files
        let is_infected = response_str.contains("FOUND");
        let virus_name = if is_infected {
            // Extract virus name from response
            response_str
                .split(':')
                .nth(1)
                .and_then(|part| part.trim().strip_suffix(" FOUND"))
                .map(|name| name.trim().to_string())
        } else {
            None
        };

        let result = ScanResult {
            is_infected,
            virus_name,
            scan_time_ms,
        };

        if is_infected {
            tracing::warn!(
                "VIRUS DETECTED in {:?}: {}",
                file_path,
                result.virus_name.as_deref().unwrap_or("Unknown")
            );
        } else {
            tracing::info!(
                "File {:?} scanned clean in {}ms",
                file_path,
                scan_time_ms
            );
        }

        Ok(result)
    }

    /// Check if ClamAV daemon is available
    pub async fn ping(&self) -> bool {
        let addr = format!("{}:{}", self.clamd_host, self.clamd_port);

        match TcpStream::connect(&addr).await {
            Ok(mut stream) => {
                // Send PING command
                if stream.write_all(b"zPING\0").await.is_err() {
                    return false;
                }

                // Read response (should be "PONG")
                let mut response = [0u8; 16];
                match stream.read(&mut response).await {
                    Ok(n) if n > 0 => {
                        let response_str = String::from_utf8_lossy(&response[..n]);
                        response_str.contains("PONG")
                    }
                    _ => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Get ClamAV version information
    pub async fn version(&self) -> AppResult<String> {
        let addr = format!("{}:{}", self.clamd_host, self.clamd_port);
        let mut stream = TcpStream::connect(&addr).await.map_err(|e| {
            AppError::StorageError(format!("ClamAV daemon unavailable: {}", e))
        })?;

        // Send VERSION command
        stream
            .write_all(b"zVERSION\0")
            .await
            .map_err(|e| AppError::ProcessingError(format!("Failed to send command: {}", e)))?;

        // Read response
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(|e| AppError::ProcessingError(format!("Failed to read response: {}", e)))?;

        Ok(String::from_utf8_lossy(&response).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    #[ignore] // Requires ClamAV daemon running
    async fn test_scan_clean_file() {
        let scanner = VirusScanner::new("localhost".to_string(), 3310);

        // Create temporary clean file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "This is a clean test file").expect("Failed to write");

        let result = scanner
            .scan_file(temp_file.path())
            .await
            .expect("Scan should succeed");

        assert!(!result.is_infected, "Clean file should not be infected");
        assert!(result.virus_name.is_none());
        assert!(result.scan_time_ms > 0);
    }

    #[tokio::test]
    #[ignore] // Requires ClamAV daemon running
    async fn test_scan_eicar_file() {
        let scanner = VirusScanner::new("localhost".to_string(), 3310);

        // Create EICAR test file (standard anti-virus test signature)
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        // EICAR test string (safe, not real malware)
        writeln!(
            temp_file,
            "X5O!P%@AP[4\\PZX54(P^)7CC)7}}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*"
        )
        .expect("Failed to write");

        let result = scanner
            .scan_file(temp_file.path())
            .await
            .expect("Scan should succeed");

        assert!(result.is_infected, "EICAR file should be detected");
        assert!(result.virus_name.is_some());
        assert!(
            result
                .virus_name
                .unwrap()
                .to_lowercase()
                .contains("eicar"),
            "Should detect EICAR signature"
        );
    }

    #[tokio::test]
    #[ignore] // Requires ClamAV daemon running
    async fn test_ping() {
        let scanner = VirusScanner::new("localhost".to_string(), 3310);
        let is_available = scanner.ping().await;
        assert!(is_available, "ClamAV daemon should respond to PING");
    }

    #[tokio::test]
    #[ignore] // Requires ClamAV daemon running
    async fn test_version() {
        let scanner = VirusScanner::new("localhost".to_string(), 3310);
        let version = scanner.version().await.expect("Should get version");
        assert!(!version.is_empty());
        println!("ClamAV version: {}", version);
    }
}
