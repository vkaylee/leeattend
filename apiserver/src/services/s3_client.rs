//! S3 Client Wrapper
//!
//! Provides simplified interface for S3 operations:
//! - Upload report files
//! - Generate presigned download URLs

use aws_config::timeout::TimeoutConfig;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use std::time::Duration;

/// S3 client wrapper for report storage operations.
pub struct S3Client {
    client: Client,
    bucket: String,
}

/// Error type for S3 operations.
#[derive(Debug)]
pub struct S3Error(pub String);

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for S3Error {}

impl S3Client {
    /// Create a new S3Client from environment configuration.
    ///
    /// Reads AWS credentials from environment/config files.
    /// Requires `S3_BUCKET` environment variable to be set.
    ///
    /// Configuration via environment variables:
    /// - `S3_BUCKET`: Required, S3 bucket name
    /// - `S3_UPLOAD_TIMEOUT_SECS`: Optional, upload timeout in seconds (default: 180 = 3 minutes)
    ///
    /// # Errors
    /// Returns error if S3_BUCKET is not set.
    pub async fn new() -> Result<Self, S3Error> {
        // Get upload timeout from environment or use 3 minutes default
        let upload_timeout_secs: u64 = std::env::var("S3_UPLOAD_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(180); // 3 minutes default

        let timeout_config = TimeoutConfig::builder()
            .operation_timeout(Duration::from_secs(upload_timeout_secs))
            .operation_attempt_timeout(Duration::from_secs(upload_timeout_secs))
            .build();

        let config = aws_config::defaults(BehaviorVersion::latest())
            .timeout_config(timeout_config)
            .load()
            .await;

        let s3_config = S3ConfigBuilder::from(&config).build();
        let client = Client::from_conf(s3_config);

        let bucket =
            std::env::var("S3_BUCKET").map_err(|_| S3Error("S3_BUCKET not set".to_string()))?;

        tracing::info!(
            bucket = %bucket,
            upload_timeout_secs = upload_timeout_secs,
            "S3 client initialized with custom timeout"
        );

        Ok(Self { client, bucket })
    }

    /// Create S3Client with explicit bucket name (for testing).
    #[allow(dead_code)]
    pub async fn with_bucket(bucket: String) -> Result<Self, S3Error> {
        let upload_timeout_secs: u64 = std::env::var("S3_UPLOAD_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(180);

        let timeout_config = TimeoutConfig::builder()
            .operation_timeout(Duration::from_secs(upload_timeout_secs))
            .operation_attempt_timeout(Duration::from_secs(upload_timeout_secs))
            .build();

        let config = aws_config::defaults(BehaviorVersion::latest())
            .timeout_config(timeout_config)
            .load()
            .await;

        let s3_config = S3ConfigBuilder::from(&config).build();
        let client = Client::from_conf(s3_config);

        Ok(Self { client, bucket })
    }

    /// Upload a report file to S3.
    ///
    /// # Arguments
    /// * `key` - S3 object key (path within bucket)
    /// * `data` - File content as bytes
    /// * `content_type` - MIME type of the file
    ///
    /// # Returns
    /// S3 object key on success.
    pub async fn upload_report(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, S3Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| S3Error(format!("S3 upload failed: {}", e)))?;

        tracing::info!(
            bucket = %self.bucket,
            key = %key,
            "Report uploaded to S3"
        );

        Ok(key.to_string())
    }

    /// Generate a presigned URL for downloading a report.
    ///
    /// # Arguments
    /// * `key` - S3 object key
    /// * `expires_in` - URL validity duration
    ///
    /// # Returns
    /// Presigned URL string that allows temporary access to the object.
    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, S3Error> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(expires_in)
            .build()
            .map_err(|e| S3Error(format!("Presign config error: {}", e)))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| S3Error(format!("Presign failed: {}", e)))?;

        Ok(presigned.uri().to_string())
    }

    /// Get the bucket name.
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_error_display() {
        let err = S3Error("test error".to_string());
        assert_eq!(format!("{}", err), "test error");
    }
}
