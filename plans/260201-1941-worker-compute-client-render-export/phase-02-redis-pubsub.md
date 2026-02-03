# Phase 2: Redis Pub/Sub Infrastructure

**Priority**: P0
**Status**: Pending
**Estimated**: Medium complexity

---

## Context Links
- [Plan Overview](./plan.md)
- [Phase 1: Backend Optimization](./phase-01-backend-optimization.md)

---

## Overview

Implement Redis Pub/Sub for real-time streaming of report data from worker to WebSocket handler.

---

## Key Insights

- Redis client already supports Pub/Sub (ConnectionManager available)
- Current system uses HSET for batch results (store-and-forward)
- Pub/Sub allows immediate delivery without storage overhead
- Channel naming: `report:stream:{job_id}`

---

## Requirements

### Functional
- Workers publish JSON chunks to Redis channel
- WebSocket handlers subscribe to channel
- Message format: `{type: "CHUNK"|"DONE"|"ERROR", data, progress}`
- Channel auto-cleanup after job completion

### Non-Functional
- Pub latency < 10ms
- Support multiple subscribers per channel
- No message loss during normal operation

---

## Related Code Files

### Create
- `apiserver/src/queue/pubsub.rs` - Pub/Sub wrapper module

### Modify
- `apiserver/src/queue/mod.rs` - Export pubsub module
- `apiserver/src/worker/handlers/attendance.rs` - Publish instead of HSET

---

## Implementation Steps

### Step 1: Create Pub/Sub Module

```rust
// apiserver/src/queue/pubsub.rs

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Channel name for report streaming
pub fn stream_channel(job_id: Uuid) -> String {
    format!("report:stream:{}", job_id)
}

/// Message types for report streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamMessage {
    /// Batch of attendance data ready
    Chunk {
        data: serde_json::Value,
        batch_index: u32,
        total_batches: u32,
        progress: u8, // 0-100
    },
    /// All batches completed successfully
    Done {
        total_rows: u32,
    },
    /// Error occurred during processing
    Error {
        message: String,
        batch_index: Option<u32>,
    },
}

/// Publisher for report streaming
pub struct ReportPublisher {
    conn: ConnectionManager,
}

impl ReportPublisher {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    /// Publish a chunk to the stream channel
    pub async fn publish_chunk(
        &mut self,
        job_id: Uuid,
        data: serde_json::Value,
        batch_index: u32,
        total_batches: u32,
    ) -> Result<(), redis::RedisError> {
        let progress = ((batch_index + 1) as f32 / total_batches as f32 * 100.0) as u8;

        let msg = StreamMessage::Chunk {
            data,
            batch_index,
            total_batches,
            progress,
        };

        let channel = stream_channel(job_id);
        let payload = serde_json::to_string(&msg).unwrap();

        self.conn.publish::<_, _, ()>(&channel, &payload).await
    }

    /// Publish completion signal
    pub async fn publish_done(
        &mut self,
        job_id: Uuid,
        total_rows: u32,
    ) -> Result<(), redis::RedisError> {
        let msg = StreamMessage::Done { total_rows };
        let channel = stream_channel(job_id);
        let payload = serde_json::to_string(&msg).unwrap();

        self.conn.publish::<_, _, ()>(&channel, &payload).await
    }

    /// Publish error signal
    pub async fn publish_error(
        &mut self,
        job_id: Uuid,
        message: String,
        batch_index: Option<u32>,
    ) -> Result<(), redis::RedisError> {
        let msg = StreamMessage::Error { message, batch_index };
        let channel = stream_channel(job_id);
        let payload = serde_json::to_string(&msg).unwrap();

        self.conn.publish::<_, _, ()>(&channel, &payload).await
    }
}

/// Subscriber for report streaming (used by WebSocket handler)
pub struct ReportSubscriber {
    pubsub: redis::aio::PubSub,
}

impl ReportSubscriber {
    pub async fn new(client: &redis::Client, job_id: Uuid) -> Result<Self, redis::RedisError> {
        let mut pubsub = client.get_async_pubsub().await?;
        let channel = stream_channel(job_id);
        pubsub.subscribe(&channel).await?;
        Ok(Self { pubsub })
    }

    /// Get next message from stream
    pub async fn next_message(&mut self) -> Option<StreamMessage> {
        use futures_util::StreamExt;

        if let Some(msg) = self.pubsub.on_message().next().await {
            let payload: String = msg.get_payload().ok()?;
            serde_json::from_str(&payload).ok()
        } else {
            None
        }
    }

    /// Unsubscribe and cleanup
    pub async fn close(mut self) -> Result<(), redis::RedisError> {
        self.pubsub.unsubscribe::<_, ()>(()).await
    }
}
```

### Step 2: Export Module

```rust
// apiserver/src/queue/mod.rs
pub mod pubsub;
pub use pubsub::{ReportPublisher, ReportSubscriber, StreamMessage, stream_channel};
```

### Step 3: Integrate Publisher in Attendance Handler

```rust
// apiserver/src/worker/handlers/attendance.rs

use crate::queue::pubsub::ReportPublisher;

// In handle_attendance_batch()
async fn handle_attendance_batch(
    ctx: &WorkerContext,
    job: AttendanceJob,
) -> Result<(), WorkerError> {
    // ... existing calculation code ...

    // After calculating results
    let result_json = serde_json::to_value(&results)?;

    // NEW: Publish to stream instead of storing in Redis hash
    let mut publisher = ReportPublisher::new(ctx.redis.clone());
    publisher.publish_chunk(
        job.parent_job_id,
        result_json,
        job.batch_index,
        job.total_batches,
    ).await?;

    // OLD (remove): ctx.queue.store_result(job.parent_job_id, job.batch_index, &json_str);

    // Update completion counter (still needed for merge trigger)
    ctx.queue.increment_completed(job.parent_job_id).await?;

    Ok(())
}
```

### Step 4: Update Merge Handler for Streaming Mode

```rust
// apiserver/src/worker/handlers/merge.rs

// When all batches complete, send DONE signal
async fn handle_merge(ctx: &WorkerContext, job: MergeJob) -> Result<(), WorkerError> {
    let mut publisher = ReportPublisher::new(ctx.redis.clone());

    // Calculate total rows (approximate from batch count)
    let total_rows = job.total_batches * BATCH_SIZE as u32;

    // Send completion signal
    publisher.publish_done(job.parent_job_id, total_rows).await?;

    // Update job status
    ctx.queue.update_job_status(job.parent_job_id, JobStatus::Completed).await?;

    // Note: No Excel generation in streaming mode!
    // Client builds Excel from received chunks

    Ok(())
}
```

---

## Todo List

- [ ] Create pubsub.rs module with Publisher/Subscriber
- [ ] Define StreamMessage enum with Chunk/Done/Error variants
- [ ] Add serde serialization for messages
- [ ] Integrate ReportPublisher in attendance handler
- [ ] Update merge handler to send DONE signal
- [ ] Add unit tests for pubsub module
- [ ] Test with redis-cli SUBSCRIBE for debugging

---

## Success Criteria

- [ ] Messages published to `report:stream:{job_id}` channel
- [ ] CHUNK messages contain batch data + progress %
- [ ] DONE message sent when all batches complete
- [ ] ERROR message sent on failure
- [ ] redis-cli can subscribe and see messages

---

## Testing Commands

```bash
# Terminal 1: Subscribe to channel
redis-cli SUBSCRIBE "report:stream:test-job-id"

# Terminal 2: Trigger export job
curl -X POST http://localhost:3000/api/workspaces/{ws_id}/reports/export \
  -H "Content-Type: application/json" \
  -d '{"type": "summary", "date_range": {...}}'
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| No subscribers | Low | Messages dropped (acceptable) |
| Subscriber slow | Medium | Redis buffers; consider backpressure |
| Connection drop | Medium | Subscriber reconnect logic |
