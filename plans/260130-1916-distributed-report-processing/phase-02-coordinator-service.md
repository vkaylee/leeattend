# Phase 2: Coordinator Service

## Context Links
- [Plan Overview](./plan.md)
- [Phase 1: Redis Queue](./phase-01-redis-queue-infrastructure.md)
- [Current Report Service](../../apiserver/src/services/report_service.rs)

## Overview
- **Priority**: P1 (Core)
- **Status**: pending
- **Effort**: 4h
- **Description**: Implement coordinator that splits large report requests into micro-jobs and tracks completion.

## Key Insights

- Coordinator is stateless - all state lives in Redis
- Batch size of 20 employees keeps RAM under 50MB per job
- For 7,000 employees: 350 micro-jobs distributed across workers
- Parent job tracks all child jobs for aggregation trigger

## Requirements

### Functional
- F1: Accept report request and return job ID immediately
- F2: Split employees into batches of 20
- F3: Enqueue all micro-jobs atomically
- F4: Create parent job metadata for tracking
- F5: Trigger merge job when all batches complete

### Non-Functional
- NF1: Job creation < 500ms for 7,000 employees
- NF2: Atomic enqueue (all-or-nothing)
- NF3: Support cancellation

## Architecture

### Coordinator Flow

```
┌────────────────────────────────────────────────────────────────────────────┐
│                         COORDINATOR SERVICE                                  │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  1. Receive Request                                                          │
│     ├─ workspace_id, date_range, report_type                                │
│     └─ Optional: employee_ids filter                                        │
│                                                                              │
│  2. Fetch Employee List (DB)                                                │
│     ├─ If filter provided: use filter                                       │
│     └─ Else: get all active employees in workspace                          │
│                                                                              │
│  3. Generate Parent Job ID                                                   │
│     └─ UUID v4                                                              │
│                                                                              │
│  4. Split into Micro-Batches                                                │
│     ├─ chunk(employees, BATCH_SIZE=20)                                      │
│     ├─ Create ReportJob for each chunk                                      │
│     └─ Each job has unique job_id + parent_job_id                           │
│                                                                              │
│  5. Create Parent Metadata in Redis                                          │
│     ├─ report:meta:{parent_id} = { status, total_batches, ... }             │
│     └─ report:counter:{parent_id} = 0                                        │
│                                                                              │
│  6. Enqueue All Jobs (Atomic)                                                │
│     └─ RPUSH report:jobs:attendance [job1, job2, ...]                       │
│                                                                              │
│  7. Return Response                                                          │
│     └─ { job_id, status: "pending", total_batches, estimated_time }         │
│                                                                              │
└────────────────────────────────────────────────────────────────────────────┘
```

### Batch Size Calculation

```
Memory Budget per Job:
┌────────────────────────────────────────────────────────┐
│ Component                    │ Size per Employee      │
├────────────────────────────────────────────────────────┤
│ Employee struct              │ ~200 bytes             │
│ Time entries (30 days avg)   │ ~3KB (30 entries)      │
│ Schedule data                │ ~1KB (30 days)         │
│ PolicyRules cache            │ ~100 bytes             │
│ Calculation buffers          │ ~500 bytes             │
│ Result serialization         │ ~500 bytes             │
├────────────────────────────────────────────────────────┤
│ TOTAL per employee           │ ~5.3KB                 │
│ Batch of 20 employees        │ ~106KB                 │
│ With 2x safety margin        │ ~212KB                 │
│ + Fixed overhead (runtime)   │ ~50MB                  │
├────────────────────────────────────────────────────────┤
│ TOTAL per job                │ ~50MB                  │
│ Safe for 256MB container     │ YES (50MB headroom)    │
└────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `apiserver/src/services/report_coordinator.rs` - Main coordinator

### Files to Modify
- `apiserver/src/services/mod.rs` - Export coordinator
- `apiserver/src/handlers/attendance.rs` - Add async export endpoint

## Implementation Steps

### Step 1: Define Constants

```rust
// report_coordinator.rs
pub const BATCH_SIZE: usize = 20;
pub const JOB_TIMEOUT_SECS: u64 = 3600;  // 1 hour max job duration
pub const ESTIMATED_SECONDS_PER_BATCH: u64 = 2;
```

### Step 2: Create Request/Response Types

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateReportJobRequest {
    pub report_type: ReportType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default)]
    pub employee_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateReportJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub total_batches: u32,
    pub estimated_seconds: u64,
    pub status_url: String,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub progress: JobProgress,
    pub result_url: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct JobProgress {
    pub total_batches: u32,
    pub completed_batches: u32,
    pub failed_batches: u32,
    pub percentage: f32,
}
```

### Step 3: Implement Coordinator Service

```rust
use crate::queue::{RedisPool, ReportQueue, ReportJob, JobMetadata, JobStatus, TaskType, DateRange, ReportConfig};
use crate::repositories::employee_repository;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

pub struct ReportCoordinator {
    queue: ReportQueue,
    db: PgPool,
}

impl ReportCoordinator {
    pub fn new(redis: RedisPool, db: PgPool) -> Self {
        Self {
            queue: ReportQueue::new(redis),
            db,
        }
    }

    /// Create a new distributed report job
    pub async fn create_report_job(
        &self,
        workspace_id: Uuid,
        requester_id: Uuid,
        request: CreateReportJobRequest,
    ) -> Result<CreateReportJobResponse, ReportError> {
        // 1. Fetch employee list
        let employees = self.fetch_employees(workspace_id, request.employee_ids.as_ref()).await?;

        if employees.is_empty() {
            return Err(ReportError::NoEmployees);
        }

        // 2. Generate parent job ID
        let parent_job_id = Uuid::new_v4();
        let now = Utc::now();

        // 3. Fetch workspace config for jobs
        let workspace = self.fetch_workspace_config(workspace_id).await?;

        // 4. Split into batches
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_batches = batches.len() as u32;

        // 5. Create micro-jobs
        let jobs: Vec<ReportJob> = batches
            .into_iter()
            .enumerate()
            .map(|(idx, emp_ids)| ReportJob {
                job_id: Uuid::new_v4(),
                parent_job_id,
                batch_index: idx as u32,
                task_type: TaskType::AttendanceReport,
                workspace_id,
                employee_ids: emp_ids,
                date_range: DateRange {
                    start: request.start_date,
                    end: request.end_date,
                },
                config: ReportConfig {
                    report_type: request.report_type.clone(),
                    timezone: workspace.timezone.clone(),
                    rounding_policy: workspace.rounding_policy.clone(),
                    work_unit_minutes: workspace.work_unit_minutes,
                },
                created_at: now,
                retry_count: 0,
                max_retries: 3,
            })
            .collect();

        // 6. Create parent metadata
        let meta = JobMetadata {
            job_id: parent_job_id,
            status: JobStatus::Pending,
            total_batches,
            completed_batches: 0,
            failed_batches: 0,
            workspace_id,
            requester_id,
            report_type: request.report_type.clone(),
            created_at: now,
            completed_at: None,
            error_message: None,
        };

        self.queue.create_job_metadata(&meta).await
            .map_err(|e| ReportError::QueueError(e.to_string()))?;

        // 7. Enqueue all jobs atomically
        self.queue.enqueue_jobs(jobs).await
            .map_err(|e| ReportError::QueueError(e.to_string()))?;

        tracing::info!(
            job_id = %parent_job_id,
            workspace_id = %workspace_id,
            total_batches = total_batches,
            "Report job created and queued"
        );

        // 8. Calculate estimated time
        let estimated_seconds = (total_batches as u64) * ESTIMATED_SECONDS_PER_BATCH;

        Ok(CreateReportJobResponse {
            job_id: parent_job_id,
            status: "pending".to_string(),
            total_batches,
            estimated_seconds,
            status_url: format!("/api/workspaces/{}/reports/jobs/{}", workspace_id, parent_job_id),
        })
    }

    /// Get job status for polling
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<JobStatusResponse, ReportError> {
        let meta = self.queue.get_job_metadata(job_id).await
            .map_err(|e| ReportError::QueueError(e.to_string()))?
            .ok_or(ReportError::JobNotFound)?;

        let percentage = if meta.total_batches > 0 {
            (meta.completed_batches as f32 / meta.total_batches as f32) * 100.0
        } else {
            0.0
        };

        // Check if file is ready
        let result_url = if meta.status == JobStatus::Completed {
            self.queue.get_result_url(job_id).await.ok().flatten()
        } else {
            None
        };

        Ok(JobStatusResponse {
            job_id: meta.job_id,
            status: format!("{:?}", meta.status).to_lowercase(),
            progress: JobProgress {
                total_batches: meta.total_batches,
                completed_batches: meta.completed_batches,
                failed_batches: meta.failed_batches,
                percentage,
            },
            result_url,
            error: meta.error_message,
            created_at: meta.created_at,
            completed_at: meta.completed_at,
        })
    }

    /// Cancel a pending job
    pub async fn cancel_job(&self, job_id: Uuid) -> Result<(), ReportError> {
        let mut meta = self.queue.get_job_metadata(job_id).await
            .map_err(|e| ReportError::QueueError(e.to_string()))?
            .ok_or(ReportError::JobNotFound)?;

        if meta.status == JobStatus::Completed || meta.status == JobStatus::Failed {
            return Err(ReportError::CannotCancel);
        }

        meta.status = JobStatus::Failed;
        meta.error_message = Some("Cancelled by user".to_string());
        meta.completed_at = Some(Utc::now());

        self.queue.update_job_metadata(&meta).await
            .map_err(|e| ReportError::QueueError(e.to_string()))?;

        tracing::info!(job_id = %job_id, "Report job cancelled");
        Ok(())
    }

    async fn fetch_employees(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, ReportError> {
        let employees = employee_repository::get_active_employee_ids(
            &self.db,
            workspace_id,
            filter_ids,
        ).await.map_err(|e| ReportError::DatabaseError(e.to_string()))?;

        Ok(employees)
    }

    async fn fetch_workspace_config(
        &self,
        workspace_id: Uuid,
    ) -> Result<WorkspaceConfig, ReportError> {
        // Fetch from workspace table
        let workspace = sqlx::query_as!(
            WorkspaceConfig,
            r#"
            SELECT timezone, rounding_policy, work_unit_minutes
            FROM workspaces
            WHERE id = $1
            "#,
            workspace_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| ReportError::DatabaseError(e.to_string()))?;

        Ok(workspace)
    }
}

#[derive(Debug)]
pub enum ReportError {
    NoEmployees,
    JobNotFound,
    CannotCancel,
    QueueError(String),
    DatabaseError(String),
}

struct WorkspaceConfig {
    timezone: String,
    rounding_policy: String,
    work_unit_minutes: i32,
}
```

### Step 4: Add Completion Handler

```rust
impl ReportCoordinator {
    /// Called by workers when a batch completes
    /// Returns true if this was the last batch (triggers merge)
    pub async fn on_batch_complete(
        &self,
        parent_job_id: Uuid,
        batch_index: u32,
        success: bool,
    ) -> Result<bool, ReportError> {
        let completed_count = if success {
            self.queue.increment_completed(parent_job_id).await
                .map_err(|e| ReportError::QueueError(e.to_string()))?
        } else {
            self.queue.increment_failed(parent_job_id).await
                .map_err(|e| ReportError::QueueError(e.to_string()))?;
            0 // Don't check for completion on failure
        };

        // Get total to check if complete
        let meta = self.queue.get_job_metadata(parent_job_id).await
            .map_err(|e| ReportError::QueueError(e.to_string()))?
            .ok_or(ReportError::JobNotFound)?;

        let all_done = completed_count >= meta.total_batches;

        if all_done {
            tracing::info!(
                parent_job_id = %parent_job_id,
                "All batches complete, triggering merge"
            );

            // Enqueue merge job
            let merge_job = ReportJob {
                job_id: Uuid::new_v4(),
                parent_job_id,
                batch_index: 0,
                task_type: TaskType::MergeResults,
                workspace_id: meta.workspace_id,
                employee_ids: vec![],  // Not needed for merge
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                },
                config: ReportConfig {
                    report_type: meta.report_type.clone(),
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
                created_at: Utc::now(),
                retry_count: 0,
                max_retries: 3,
            };

            self.queue.enqueue_jobs(vec![merge_job]).await
                .map_err(|e| ReportError::QueueError(e.to_string()))?;
        }

        Ok(all_done)
    }
}
```

## Todo List

- [ ] Create `ReportCoordinator` struct
- [ ] Implement `create_report_job` with batch splitting
- [ ] Implement `get_job_status` for polling
- [ ] Implement `cancel_job` for user cancellation
- [ ] Implement `on_batch_complete` for worker callback
- [ ] Add merge job triggering when all batches complete
- [ ] Add employee_repository::get_active_employee_ids function
- [ ] Write unit tests for batch splitting
- [ ] Write integration tests with mock Redis

## Success Criteria

- [ ] Report request returns in < 500ms
- [ ] Correct number of batches created (ceil(employees/20))
- [ ] All jobs enqueued atomically
- [ ] Status polling shows correct progress
- [ ] Merge job triggered when all complete

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Batch count miscalculation | Unit test edge cases (1, 19, 20, 21 employees) |
| Race condition on completion | Use atomic Redis increment |
| Orphaned jobs if coordinator crashes | TTL + dead letter queue processing |

## Security Considerations

- Validate requester has access to workspace
- Sanitize employee_ids filter (prevent SQL injection)
- Rate limit job creation per workspace

## Next Steps

After this phase:
1. Phase 3: Generic Worker - Process the enqueued jobs
2. Phase 5: Result Aggregation - Merge job handling
