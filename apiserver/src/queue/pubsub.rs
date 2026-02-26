//! Redis Pub/Sub module for streaming report results.
//!
//! Enables real-time streaming of batch results to WebSocket clients
//! instead of storing and polling for completed files.

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::queue::RedisPool;

/// Channel prefix for report streams.
const STREAM_CHANNEL_PREFIX: &str = "report:stream:";

/// Message types for streaming report results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamMessage {
    /// A chunk of report data (batch completed).
    Chunk {
        batch_index: u32,
        total_batches: u32,
    },
    /// All batches complete, stream ending.
    Done {
        total_batches: u32,
        total_records: usize,
    },
    /// Error occurred during processing.
    Error {
        message: String,
        batch_index: Option<u32>,
    },
}

impl StreamMessage {
    /// Create a new chunk message.
    pub fn chunk(batch_index: u32, total_batches: u32) -> Self {
        Self::Chunk {
            batch_index,
            total_batches,
        }
    }

    /// Create a done message.
    pub fn done(total_batches: u32, total_records: usize) -> Self {
        Self::Done {
            total_batches,
            total_records,
        }
    }

    /// Create an error message.
    pub fn error(message: impl Into<String>, batch_index: Option<u32>) -> Self {
        Self::Error {
            message: message.into(),
            batch_index,
        }
    }
}

/// Subscriber for receiving streamed report results via Redis Pub/Sub.
pub struct ReportSubscriber {
    pubsub: redis::aio::PubSub,
    job_id: Uuid,
}

impl ReportSubscriber {
    /// Create a new subscriber for a job's stream channel.
    pub async fn new(pool: &RedisPool, job_id: Uuid) -> Result<Self, redis::RedisError> {
        let channel = ReportPublisher::channel_name(job_id);

        tracing::debug!(
            job_id = %job_id,
            channel = %channel,
            "Subscribing to report stream"
        );

        let mut pubsub = pool.get_pubsub_connection().await?;
        pubsub.subscribe(&channel).await?;

        Ok(Self { pubsub, job_id })
    }

    /// Get the next message from the stream.
    /// Returns None when the connection is closed.
    pub async fn next_message(&mut self) -> Option<StreamMessage> {
        use futures_util::StreamExt;

        let msg = self.pubsub.on_message().next().await?;
        let payload: String = msg.get_payload().ok()?;

        match serde_json::from_str(&payload) {
            Ok(stream_msg) => Some(stream_msg),
            Err(e) => {
                tracing::warn!(
                    job_id = %self.job_id,
                    error = %e,
                    "Failed to parse stream message"
                );
                None
            }
        }
    }

    /// Close the subscription.
    pub async fn close(mut self) -> Result<(), redis::RedisError> {
        let channel = ReportPublisher::channel_name(self.job_id);
        self.pubsub.unsubscribe(&channel).await
    }
}

/// Publisher for streaming report results via Redis Pub/Sub.
pub struct ReportPublisher {
    pool: RedisPool,
}

impl ReportPublisher {
    /// Create a new publisher with the given Redis pool.
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Get the channel name for a job.
    pub fn channel_name(job_id: Uuid) -> String {
        format!("{STREAM_CHANNEL_PREFIX}{job_id}")
    }

    /// Publish a chunk of progress to the stream.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID
    /// * `batch_index` - Current batch number (0-based)
    /// * `total_batches` - Total number of batches
    pub async fn publish_chunk(
        &self,
        job_id: Uuid,
        batch_index: u32,
        total_batches: u32,
    ) -> Result<(), redis::RedisError> {
        let msg = StreamMessage::chunk(batch_index, total_batches);
        self.publish(job_id, &msg).await
    }

    /// Publish a completion message.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID
    /// * `total_batches` - Total number of batches processed
    /// * `total_records` - Total number of records across all batches
    pub async fn publish_done(
        &self,
        job_id: Uuid,
        total_batches: u32,
        total_records: usize,
    ) -> Result<(), redis::RedisError> {
        let msg = StreamMessage::done(total_batches, total_records);
        self.publish(job_id, &msg).await
    }

    /// Publish an error message.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID
    /// * `message` - Error description
    /// * `batch_index` - Optional batch that caused the error
    pub async fn publish_error(
        &self,
        job_id: Uuid,
        message: impl Into<String>,
        batch_index: Option<u32>,
    ) -> Result<(), redis::RedisError> {
        let msg = StreamMessage::error(message, batch_index);
        self.publish(job_id, &msg).await
    }

    /// Publish a message to the job's stream channel.
    async fn publish(
        &self,
        job_id: Uuid,
        message: &StreamMessage,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let channel = Self::channel_name(job_id);
        let json =
            serde_json::to_string(message).expect("StreamMessage serialization should not fail");

        tracing::debug!(
            job_id = %job_id,
            channel = %channel,
            message_type = ?std::mem::discriminant(message),
            "Publishing stream message"
        );

        let _: () = conn.publish(&channel, json).await?;
        Ok(())
    }

    /// Get the number of subscribers for a job's channel.
    /// Useful for checking if anyone is listening before publishing.
    pub async fn subscriber_count(&self, job_id: Uuid) -> Result<usize, redis::RedisError> {
        let mut conn = self.pool.conn();
        let channel = Self::channel_name(job_id);

        // PUBSUB NUMSUB returns pairs of (channel, count)
        let result: Vec<(String, usize)> = redis::cmd("PUBSUB")
            .arg("NUMSUB")
            .arg(&channel)
            .query_async(&mut conn)
            .await?;

        Ok(result.first().map(|(_, count)| *count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_name_format() {
        let job_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let channel = ReportPublisher::channel_name(job_id);
        assert_eq!(
            channel,
            "report:stream:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_stream_message_chunk_serialization() {
        let msg = StreamMessage::chunk(0, 3);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"CHUNK\""));
        assert!(json.contains("\"batch_index\":0"));
        assert!(json.contains("\"total_batches\":3"));

        // Verify roundtrip
        let decoded: StreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            StreamMessage::Chunk {
                batch_index,
                total_batches,
                ..
            } => {
                assert_eq!(batch_index, 0);
                assert_eq!(total_batches, 3);
            }
            _ => panic!("Expected Chunk variant"),
        }
    }

    #[test]
    fn test_stream_message_done_serialization() {
        let msg = StreamMessage::done(5, 1000);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"DONE\""));
        assert!(json.contains("\"total_batches\":5"));
        assert!(json.contains("\"total_records\":1000"));

        let decoded: StreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            StreamMessage::Done {
                total_batches,
                total_records,
            } => {
                assert_eq!(total_batches, 5);
                assert_eq!(total_records, 1000);
            }
            _ => panic!("Expected Done variant"),
        }
    }

    #[test]
    fn test_stream_message_error_serialization() {
        let msg = StreamMessage::error("Database connection failed", Some(2));
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"ERROR\""));
        assert!(json.contains("\"message\":\"Database connection failed\""));
        assert!(json.contains("\"batch_index\":2"));

        let decoded: StreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            StreamMessage::Error {
                message,
                batch_index,
            } => {
                assert_eq!(message, "Database connection failed");
                assert_eq!(batch_index, Some(2));
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_stream_message_error_without_batch() {
        let msg = StreamMessage::error("General failure", None);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"batch_index\":null"));

        let decoded: StreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            StreamMessage::Error { batch_index, .. } => {
                assert_eq!(batch_index, None);
            }
            _ => panic!("Expected Error variant"),
        }
    }

    /// Optimization #5: CHUNK messages carry only progress, not batch data.
    /// This saves ~600KB per message for a 200-employee batch.
    #[test]
    fn test_chunk_message_has_no_data_field() {
        let msg = StreamMessage::chunk(2, 5);
        let json = serde_json::to_string(&msg).unwrap();

        // Must NOT contain a "data" key
        assert!(
            !json.contains("\"data\""),
            "CHUNK message should not contain 'data' field. Got: {json}"
        );

        // Must contain only type, batch_index, total_batches
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        let keys: Vec<&String> = obj.keys().collect();
        assert_eq!(keys.len(), 3, "CHUNK should have exactly 3 fields, got: {keys:?}");
        assert!(obj.contains_key("type"));
        assert!(obj.contains_key("batch_index"));
        assert!(obj.contains_key("total_batches"));
    }

    /// Verify CHUNK progress values are correctly serialized.
    #[test]
    fn test_chunk_progress_values() {
        // Last batch of a 10-batch job
        let msg = StreamMessage::chunk(9, 10);
        let json = serde_json::to_string(&msg).unwrap();

        let decoded: StreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            StreamMessage::Chunk {
                batch_index,
                total_batches,
            } => {
                assert_eq!(batch_index, 9);
                assert_eq!(total_batches, 10);
            }
            _ => panic!("Expected Chunk variant"),
        }
    }
}
