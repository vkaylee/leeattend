use async_trait::async_trait;
use uuid::Uuid;

/// Abstract interface for Report streaming/publishing operations.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PublisherRepository: Send + Sync {
    /// Publish a chunk of progress to the stream.
    async fn publish_chunk(
        &self,
        job_id: Uuid,
        batch_index: u32,
        total_batches: u32,
    ) -> Result<(), String>;

    /// Publish a completion message.
    async fn publish_done(
        &self,
        job_id: Uuid,
        total_batches: u32,
        total_records: usize,
    ) -> Result<(), String>;

    /// Publish an error message.
    async fn publish_error(
        &self,
        job_id: Uuid,
        message: String,
        batch_index: Option<u32>,
    ) -> Result<(), String>;
}

use crate::queue::{RedisPool, ReportPublisher};

pub struct RedisPublisherRepository {
    publisher: ReportPublisher,
}

impl RedisPublisherRepository {
    pub fn new(redis: RedisPool) -> Self {
        Self {
            publisher: ReportPublisher::new(redis),
        }
    }
}

#[async_trait]
impl PublisherRepository for RedisPublisherRepository {
    async fn publish_chunk(
        &self,
        job_id: Uuid,
        batch_index: u32,
        total_batches: u32,
    ) -> Result<(), String> {
        self.publisher
            .publish_chunk(job_id, batch_index, total_batches)
            .await
            .map_err(|e| e.to_string())
    }

    async fn publish_done(
        &self,
        job_id: Uuid,
        total_batches: u32,
        total_records: usize,
    ) -> Result<(), String> {
        self.publisher
            .publish_done(job_id, total_batches, total_records)
            .await
            .map_err(|e| e.to_string())
    }

    async fn publish_error(
        &self,
        job_id: Uuid,
        message: String,
        batch_index: Option<u32>,
    ) -> Result<(), String> {
        self.publisher
            .publish_error(job_id, message, batch_index)
            .await
            .map_err(|e| e.to_string())
    }
}
