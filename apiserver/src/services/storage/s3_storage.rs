use crate::errors::{AppError, AppResult};
use crate::services::storage::{ObjectMeta, ObjectStorage};
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use aws_sdk_s3::Client;
use bytes::Bytes;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint_url: String,
    pub region: String,
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_path_style: bool,
}

#[derive(Clone)]
pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    /// Build S3 storage from the centralized env registry.
    pub async fn from_registry(
        registry: &crate::utils::env_registry::EnvRegistry,
    ) -> AppResult<Self> {
        let endpoint_url = registry
            .get("S3_ENDPOINT_URL")
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::Internal("S3_ENDPOINT_URL env var not set".to_string()))?;
        let bucket_name = registry
            .get("S3_BUCKET_NAME")
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::Internal("S3_BUCKET_NAME env var not set".to_string()))?;
        let access_key = registry
            .get("S3_ACCESS_KEY")
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::Internal("S3_ACCESS_KEY env var not set".to_string()))?;
        let secret_key = registry
            .get("S3_SECRET_KEY")
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::Internal("S3_SECRET_KEY env var not set".to_string()))?;
        let region = registry
            .get("S3_REGION")
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "us-east-1".to_string());
        let use_path_style = registry
            .get_parsed::<bool>("S3_USE_PATH_STYLE")
            .unwrap_or(true);

        let config = S3Config {
            endpoint_url,
            region,
            bucket_name,
            access_key,
            secret_key,
            use_path_style,
        };

        Self::new(config).await
    }

    #[allow(clippy::unused_async)]
    pub async fn new(config: S3Config) -> AppResult<Self> {
        use aws_credential_types::Credentials;
        use aws_sdk_s3::config::Region;

        let credentials = Credentials::new(
            config.access_key.clone(),
            config.secret_key.clone(),
            None,
            None,
            "static",
        );

        let builder = aws_sdk_s3::Config::builder()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Some(Region::new(config.region)))
            .endpoint_url(config.endpoint_url)
            .force_path_style(config.use_path_style);

        let client = Client::from_conf(builder.build());

        // Ensure the bucket exists (or try to create it, or let the health check fail if it doesn't)
        // Since central fallback buckets should already exist, we don't automatically create
        // but we verify health in health_check.

        Ok(Self {
            client,
            bucket: config.bucket_name,
        })
    }
}

#[async_trait]
impl ObjectStorage for S3Storage {
    async fn put_object(&self, key: &str, data: Bytes, content_type: &str) -> AppResult<()> {
        use aws_sdk_s3::primitives::ByteStream;
        let body = ByteStream::from(data);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 put_object error: {e}")))?;
        Ok(())
    }

    async fn get_object(&self, key: &str) -> AppResult<Bytes> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("NotFound")
                    || err_str.contains("NoSuchKey")
                    || err_str.contains("404")
                {
                    AppError::not_found_args(
                        I18nKey::ErrorFileNotFound,
                        [("details", format!("File not found in S3: {key}"))],
                    )
                } else {
                    AppError::Internal(format!("S3 get_object error: {err_str}"))
                }
            })?;

        let bytes = resp
            .body
            .collect()
            .await
            .map_err(|e| AppError::Internal(format!("S3 read bytes error: {e}")))?
            .into_bytes();

        Ok(bytes)
    }

    async fn delete_object(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 delete_object error: {e}")))?;
        Ok(())
    }

    async fn presigned_get_url(&self, key: &str, ttl: Duration) -> AppResult<String> {
        use aws_sdk_s3::presigning::PresigningConfig;

        let presigning_config = PresigningConfig::builder()
            .expires_in(ttl)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build presigning config: {e}")))?;

        let presigned_req = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::Internal(format!("S3 presigned get URL error: {e}")))?;

        Ok(presigned_req.uri().to_string())
    }

    async fn presigned_put_url(
        &self,
        key: &str,
        content_type: &str,
        ttl: Duration,
    ) -> AppResult<String> {
        use aws_sdk_s3::presigning::PresigningConfig;

        let presigning_config = PresigningConfig::builder()
            .expires_in(ttl)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build presigning config: {e}")))?;

        let presigned_req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::Internal(format!("S3 presigned put URL error: {e}")))?;

        Ok(presigned_req.uri().to_string())
    }

    async fn head_object(&self, key: &str) -> AppResult<ObjectMeta> {
        let resp = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await;

        match resp {
            Ok(output) => {
                let last_modified = output.last_modified().map(|lm| {
                    chrono::DateTime::<chrono::Utc>::from_timestamp(lm.secs(), lm.subsec_nanos())
                        .unwrap_or_else(chrono::Utc::now)
                });
                Ok(ObjectMeta {
                    content_type: output.content_type().map(std::string::ToString::to_string),
                    content_length: output.content_length().unwrap_or(0),
                    last_modified,
                })
            }
            Err(err) => {
                let err_str = err.to_string();
                if err_str.contains("NotFound")
                    || err_str.contains("NoSuchKey")
                    || err_str.contains("404")
                {
                    Err(AppError::not_found_args(
                        I18nKey::ErrorFileNotFound,
                        [("details", format!("File not found in S3: {key}"))],
                    ))
                } else {
                    Err(AppError::Internal(format!(
                        "S3 head_object error: {err_str}"
                    )))
                }
            }
        }
    }

    async fn health_check(&self) -> AppResult<()> {
        self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .max_keys(1)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 health check failed: {e}")))?;
        Ok(())
    }
}
