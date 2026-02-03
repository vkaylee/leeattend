# Phase 1: Redis Queue Infrastructure

## Context Links
- [Plan Overview](./plan.md)
- [Scout Report](../reports/scout-260130-1711-attendance-report-architecture.md)
- [Agent Log Buffer Pattern](../../apiserver/src/services/agent_log_buffer.rs)

## Overview
- **Priority**: P1 (Foundation)
- **Status**: pending
- **Effort**: 4h
- **Description**: Set up Redis connection pool and define queue data structures for distributed job processing.

## Key Insights

- Redis LIST with BLPOP provides simple, reliable work distribution
- STREAM offers better observability but adds complexity
- Recommend LIST for MVP, upgrade to STREAM if needed
- Job fingerprinting prevents duplicate processing

## Requirements

### Functional
- F1: Redis connection pool integrated into AppState
- F2: Job schema supports report type routing
- F3: Atomic job enqueue/dequeue operations
- F4: Job status tracking with TTL

### Non-Functional
- NF1: Connection pool max 10 per container
- NF2: BLPOP timeout 30 seconds (prevent zombie connections)
- NF3: Job TTL 24 hours (auto-cleanup)
- NF4: Reconnection with exponential backoff

## Architecture

### Queue Data Structures

```
# Job Queue (LIST)
report:jobs:attendance          # Pending attendance report jobs
report:jobs:payroll             # Future: payroll report jobs

# Job Metadata (HASH)
report:meta:{job_id}            # Job status, progress, config
  - status: "pending" | "processing" | "completed" | "failed"
  - total_jobs: 350
  - completed_jobs: 348
  - failed_jobs: 2
  - created_at: 1706614560
  - workspace_id: uuid
  - requester_id: uuid
  - report_type: "attendance_summary"
  - date_range: "2026-01-01:2026-01-31"

# Micro-Job Results (HASH)
report:results:{job_id}         # Partial results from workers
  - batch_001: {...json...}
  - batch_002: {...json...}

# Completed File URL (STRING with TTL)
report:file:{job_id}            # S3 presigned URL
  - expires in 24 hours

# Dead Letter Queue
report:dlq                      # Failed jobs for retry/inspection
```

### Job Schema (JSON)

```json
{
  "job_id": "uuid",
  "parent_job_id": "uuid",        // Link to coordinator job
  "batch_index": 0,               // 0-based batch number
  "task_type": "attendance_report",
  "workspace_id": "uuid",
  "employee_ids": ["uuid1", "uuid2", ...],  // Max 20 per job
  "date_range": {
    "start": "2026-01-01",
    "end": "2026-01-31"
  },
  "config": {
    "report_type": "summary",     // "summary" | "detail"
    "timezone": "Asia/Ho_Chi_Minh",
    "rounding_policy": "floor",
    "work_unit_minutes": 30
  },
  "created_at": 1706614560,
  "retry_count": 0,
  "max_retries": 3
}
```

## Related Code Files

### Files to Create
- `apiserver/src/queue/mod.rs` - Queue module
- `apiserver/src/queue/redis_client.rs` - Connection pool wrapper
- `apiserver/src/queue/job.rs` - Job struct and serialization
- `apiserver/src/queue/report_queue.rs` - Report-specific queue operations

### Files to Modify
- `apiserver/src/lib.rs` - Add queue module, extend AppState
- `apiserver/src/main.rs` - Initialize Redis connection
- `apiserver/Cargo.toml` - Add redis crate

## Implementation Steps

### Step 1: Add Redis Dependency
```toml
# Cargo.toml
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
```

### Step 2: Create Queue Module Structure
```
apiserver/src/queue/
├── mod.rs
├── redis_client.rs
├── job.rs
└── report_queue.rs
```

### Step 3: Implement Redis Client Wrapper

```rust
// redis_client.rs
use redis::{Client, aio::ConnectionManager};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisPool {
    conn_manager: ConnectionManager,
}

impl RedisPool {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let conn_manager = ConnectionManager::new(client).await?;
        Ok(Self { conn_manager })
    }

    pub fn conn(&self) -> ConnectionManager {
        self.conn_manager.clone()
    }
}
```

### Step 4: Define Job Types

```rust
// job.rs
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    AttendanceReport,
    PayrollReport,  // Future
    MergeResults,   // Aggregation job
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Summary,
    Detail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportJob {
    pub job_id: Uuid,
    pub parent_job_id: Uuid,
    pub batch_index: u32,
    pub task_type: TaskType,
    pub workspace_id: Uuid,
    pub employee_ids: Vec<Uuid>,
    pub date_range: DateRange,
    pub config: ReportConfig,
    pub created_at: DateTime<Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub report_type: ReportType,
    pub timezone: String,
    pub rounding_policy: String,
    pub work_unit_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub total_batches: u32,
    pub completed_batches: u32,
    pub failed_batches: u32,
    pub workspace_id: Uuid,
    pub requester_id: Uuid,
    pub report_type: ReportType,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}
```

### Step 5: Implement Queue Operations

```rust
// report_queue.rs
use redis::AsyncCommands;
use crate::queue::{RedisPool, ReportJob, JobMetadata, JobStatus};

const QUEUE_KEY: &str = "report:jobs:attendance";
const META_PREFIX: &str = "report:meta:";
const RESULTS_PREFIX: &str = "report:results:";
const JOB_TTL_SECS: i64 = 86400; // 24 hours

pub struct ReportQueue {
    pool: RedisPool,
}

impl ReportQueue {
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Enqueue a batch of jobs atomically
    pub async fn enqueue_jobs(&self, jobs: Vec<ReportJob>) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let serialized: Vec<String> = jobs
            .iter()
            .map(|j| serde_json::to_string(j).unwrap())
            .collect();

        // RPUSH for FIFO ordering
        conn.rpush(QUEUE_KEY, &serialized).await
    }

    /// Blocking pop with timeout (for workers)
    pub async fn dequeue_job(&self, timeout_secs: f64) -> Result<Option<ReportJob>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let result: Option<(String, String)> = conn.blpop(QUEUE_KEY, timeout_secs).await?;

        match result {
            Some((_, json)) => {
                let job: ReportJob = serde_json::from_str(&json)
                    .map_err(|e| redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "JSON parse error",
                        e.to_string()
                    )))?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Create job metadata (coordinator calls this)
    pub async fn create_job_metadata(&self, meta: &JobMetadata) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", META_PREFIX, meta.job_id);
        let json = serde_json::to_string(meta).unwrap();

        redis::pipe()
            .set(&key, json)
            .expire(&key, JOB_TTL_SECS)
            .query_async(&mut conn)
            .await
    }

    /// Get job metadata (status polling)
    pub async fn get_job_metadata(&self, job_id: Uuid) -> Result<Option<JobMetadata>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", META_PREFIX, job_id);
        let json: Option<String> = conn.get(&key).await?;

        Ok(json.map(|s| serde_json::from_str(&s).unwrap()))
    }

    /// Increment completed batch count (workers call this)
    pub async fn increment_completed(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", META_PREFIX, parent_job_id);

        // Get and increment atomically would need Lua script
        // For simplicity, use HINCRBY on hash field
        // But our meta is JSON string, so we need to redesign to use HASH

        // Alternative: Use separate counter key
        let counter_key = format!("report:counter:{}", parent_job_id);
        let count: u32 = conn.incr(&counter_key, 1).await?;
        Ok(count)
    }

    /// Store batch result
    pub async fn store_result(&self, parent_job_id: Uuid, batch_index: u32, result: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", RESULTS_PREFIX, parent_job_id);
        let field = format!("batch_{:05}", batch_index);

        redis::pipe()
            .hset(&key, &field, result)
            .expire(&key, JOB_TTL_SECS)
            .query_async(&mut conn)
            .await
    }

    /// Get all results for aggregation
    pub async fn get_all_results(&self, parent_job_id: Uuid) -> Result<Vec<(String, String)>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", RESULTS_PREFIX, parent_job_id);
        conn.hgetall(&key).await
    }
}
```

### Step 6: Extend AppState

```rust
// lib.rs additions
use crate::queue::RedisPool;

pub struct AppState {
    pub agent_manager: AgentManager,
    pub db: PgPool,
    pub redis: Option<RedisPool>,  // Optional for graceful degradation
    // ... existing fields
}
```

### Step 7: Initialize in main.rs

```rust
// main.rs additions
let redis_url = std::env::var("REDIS_URL").ok();
let redis_pool = match redis_url {
    Some(url) => {
        match RedisPool::new(&url).await {
            Ok(pool) => {
                tracing::info!("Redis connected for distributed processing");
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("Redis unavailable, falling back to sync processing: {}", e);
                None
            }
        }
    }
    None => {
        tracing::info!("REDIS_URL not set, distributed processing disabled");
        None
    }
};
```

## Todo List

- [ ] Add redis crate to Cargo.toml
- [ ] Create `queue/mod.rs` with module exports
- [ ] Implement `RedisPool` with connection manager
- [ ] Define job types and serialization
- [ ] Implement `ReportQueue` with enqueue/dequeue
- [ ] Add metadata and result storage operations
- [ ] Extend AppState with optional Redis pool
- [ ] Add Redis initialization to main.rs
- [ ] Write unit tests for job serialization
- [ ] Write integration tests with Redis container

## Success Criteria

- [ ] Redis connection established with health check
- [ ] Jobs can be enqueued and dequeued correctly
- [ ] Job metadata persists with TTL
- [ ] Batch results stored and retrieved
- [ ] Graceful degradation when Redis unavailable

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Redis connection failure at startup | Make Redis optional, fallback to sync |
| Job serialization errors | Comprehensive unit tests |
| Memory leak from uncleaned keys | TTL on all keys, periodic cleanup |

## Security Considerations

- Redis connection string should be in environment variable, not code
- Consider Redis AUTH for production
- Use TLS for Redis connections in production
- Sanitize job payloads to prevent injection

## Next Steps

After this phase:
1. Phase 2: Coordinator Service - Job splitting and submission
2. Phase 3: Generic Worker - Consume and process jobs
