use crate::errors::AppResult;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::time::Duration;

pub mod file_service;
pub mod key_generator;
pub mod resolver;
pub mod s3_storage;
pub mod scanner;

pub use file_service::FileService;
pub use resolver::StorageResolver;
pub use s3_storage::{S3Config, S3Storage};
pub use scanner::{ScanProvider, ScanResult, VirusScanner};

#[derive(Debug, Clone)]
pub struct ObjectMeta {
    pub content_type: Option<String>,
    pub content_length: i64,
    pub last_modified: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put_object(&self, key: &str, data: Bytes, content_type: &str) -> AppResult<()>;
    async fn get_object(&self, key: &str) -> AppResult<Bytes>;
    async fn delete_object(&self, key: &str) -> AppResult<()>;
    async fn presigned_get_url(&self, key: &str, ttl: Duration) -> AppResult<String>;
    async fn presigned_put_url(
        &self,
        key: &str,
        content_type: &str,
        ttl: Duration,
    ) -> AppResult<String>;
    async fn head_object(&self, key: &str) -> AppResult<ObjectMeta>;
    async fn health_check(&self) -> AppResult<()>;
}
