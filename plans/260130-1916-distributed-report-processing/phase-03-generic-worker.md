# Phase 3: Generic Worker Pattern

## Context Links
- [Plan Overview](./plan.md)
- [Phase 1: Redis Queue](./phase-01-redis-queue-infrastructure.md)
- [Phase 2: Coordinator](./phase-02-coordinator-service.md)
- [Agent Log Buffer Pattern](../../apiserver/src/services/agent_log_buffer.rs)

## Overview
- **Priority**: P1 (Core)
- **Status**: pending
- **Effort**: 5h
- **Description**: Implement background worker that polls Redis queue, processes jobs with semaphore-controlled concurrency.

## Key Insights

- Follow existing `agent_log_buffer.rs` pattern for worker lifecycle
- Semaphore limits concurrent jobs per container (default: 2)
- Route jobs by `task_type` field
- Workers are stateless - can run in any container

## Requirements

### Functional
- F1: Background tokio task polls Redis with BLPOP
- F2: Route jobs to handlers based on task_type
- F3: Store results in Redis after processing
- F4: Report completion to coordinator
- F5: Handle retries for failed jobs

### Non-Functional
- NF1: Max 2 concurrent jobs per 256MB container
- NF2: Each job completes in < 30 seconds
- NF3: Graceful shutdown (drain current job, no new jobs)
- NF4: Memory usage < 50MB per job

## Architecture

### Worker Lifecycle

```
┌────────────────────────────────────────────────────────────────────────────┐
│                          GENERIC WORKER                                      │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐         ┌─────────────────────────────────────────┐   │
│  │  Main Loop      │         │  Job Handler Pool                        │   │
│  │                 │         │                                          │   │
│  │  loop {        │         │  ┌─────────────────────────────────┐    │   │
│  │    // Acquire  │         │  │ Semaphore (permits = 2)         │    │   │
│  │    permit =    │         │  └─────────────────────────────────┘    │   │
│  │    sem.acquire │◀────────│                                          │   │
│  │                │         │  ┌───────────┐  ┌───────────┐           │   │
│  │    // Block on │         │  │  Task 1   │  │  Task 2   │           │   │
│  │    job = BLPOP │         │  │ (running) │  │ (running) │           │   │
│  │                │         │  └───────────┘  └───────────┘           │   │
│  │    // Spawn    │─────────│                                          │   │
│  │    tokio::spawn│         │                                          │   │
│  │                │         │                                          │   │
│  │    // Permit   │         │                                          │   │
│  │    released on │         │                                          │   │
│  │    task end    │         │                                          │   │
│  │  }             │         │                                          │   │
│  └─────────────────┘         └─────────────────────────────────────────┘   │
│                                                                              │
│  Graceful Shutdown:                                                          │
│  1. Stop accepting new jobs (exit BLPOP loop)                               │
│  2. Wait for in-flight tasks to complete                                    │
│  3. Release resources                                                        │
│                                                                              │
└────────────────────────────────────────────────────────────────────────────┘
```

### Job Routing

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           JOB ROUTER                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  match job.task_type {                                                       │
│      TaskType::AttendanceReport => attendance_handler(job).await,            │
│      TaskType::PayrollReport    => payroll_handler(job).await,               │
│      TaskType::MergeResults     => merge_handler(job).await,                 │
│  }                                                                           │
│                                                                               │
│  Each handler:                                                               │
│  1. Fetch required data from DB (batch only for this job's employees)       │
│  2. Compute results                                                          │
│  3. Store results in Redis                                                   │
│  4. Report completion to coordinator                                         │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `apiserver/src/worker/mod.rs` - Worker module
- `apiserver/src/worker/runner.rs` - Main worker loop
- `apiserver/src/worker/handlers/mod.rs` - Handler module
- `apiserver/src/worker/handlers/attendance.rs` - Attendance report handler
- `apiserver/src/worker/handlers/merge.rs` - Result merge handler

### Files to Modify
- `apiserver/src/lib.rs` - Export worker module
- `apiserver/src/main.rs` - Spawn worker task

## Implementation Steps

### Step 1: Define Worker Configuration

```rust
// worker/mod.rs
pub mod runner;
pub mod handlers;

pub use runner::{ReportWorker, WorkerConfig};

/// Configuration for worker behavior
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Max concurrent jobs per worker (default: 2)
    pub max_concurrent_jobs: usize,
    /// BLPOP timeout in seconds (default: 30)
    pub poll_timeout_secs: f64,
    /// Max retries per job before moving to DLQ (default: 3)
    pub max_retries: u32,
    /// Job processing timeout in seconds (default: 300)
    pub job_timeout_secs: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 2,
            poll_timeout_secs: 30.0,
            max_retries: 3,
            job_timeout_secs: 300,
        }
    }
}
```

### Step 2: Implement Worker Runner

```rust
// worker/runner.rs
use crate::queue::{RedisPool, ReportQueue, ReportJob, TaskType};
use crate::services::ReportCoordinator;
use crate::worker::handlers::{AttendanceHandler, MergeHandler};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, error};

pub struct ReportWorker {
    queue: ReportQueue,
    db: PgPool,
    config: WorkerConfig,
    coordinator: Arc<ReportCoordinator>,
}

impl ReportWorker {
    pub fn new(
        redis: RedisPool,
        db: PgPool,
        config: WorkerConfig,
        coordinator: Arc<ReportCoordinator>,
    ) -> Self {
        Self {
            queue: ReportQueue::new(redis),
            db,
            config,
            coordinator,
        }
    }

    /// Start the worker as a background task
    /// Returns a JoinHandle for graceful shutdown
    pub fn start(
        self,
        cancel_token: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.run(cancel_token).await;
        })
    }

    async fn run(self, cancel_token: CancellationToken) {
        info!(
            max_concurrent = self.config.max_concurrent_jobs,
            "Report worker started"
        );

        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_jobs));
        let worker = Arc::new(self);

        loop {
            // Check for shutdown
            if cancel_token.is_cancelled() {
                info!("Worker received shutdown signal, stopping...");
                break;
            }

            // Acquire semaphore permit (blocks if at capacity)
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    error!("Semaphore closed unexpectedly");
                    break;
                }
            };

            // Poll for job (with cancellation check)
            let job_result = tokio::select! {
                _ = cancel_token.cancelled() => {
                    drop(permit);
                    info!("Worker cancelled during poll");
                    break;
                }
                result = worker.queue.dequeue_job(worker.config.poll_timeout_secs) => result
            };

            match job_result {
                Ok(Some(job)) => {
                    let worker_clone = worker.clone();
                    let job_id = job.job_id;
                    let parent_id = job.parent_job_id;

                    // Spawn task to process job
                    tokio::spawn(async move {
                        let _permit = permit; // Hold permit until done

                        let result = timeout(
                            Duration::from_secs(worker_clone.config.job_timeout_secs),
                            worker_clone.process_job(job),
                        ).await;

                        match result {
                            Ok(Ok(())) => {
                                info!(job_id = %job_id, "Job completed successfully");
                            }
                            Ok(Err(e)) => {
                                error!(job_id = %job_id, error = %e, "Job failed");
                            }
                            Err(_) => {
                                error!(job_id = %job_id, "Job timed out");
                            }
                        }
                    });
                }
                Ok(None) => {
                    // Timeout, no job available - continue loop
                    drop(permit);
                }
                Err(e) => {
                    error!(error = %e, "Failed to dequeue job");
                    drop(permit);
                    // Brief sleep before retry
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        // Wait for in-flight tasks
        info!("Waiting for {} permits to be released...",
            self.config.max_concurrent_jobs - semaphore.available_permits());

        // Acquire all permits to ensure tasks complete
        let _all_permits = semaphore
            .acquire_many(self.config.max_concurrent_jobs as u32)
            .await;

        info!("Report worker stopped");
    }

    async fn process_job(&self, job: ReportJob) -> Result<(), String> {
        let job_id = job.job_id;
        let parent_id = job.parent_job_id;
        let batch_index = job.batch_index;

        info!(
            job_id = %job_id,
            parent_id = %parent_id,
            batch_index = batch_index,
            task_type = ?job.task_type,
            employees = job.employee_ids.len(),
            "Processing job"
        );

        // Route to appropriate handler
        let result = match job.task_type {
            TaskType::AttendanceReport => {
                AttendanceHandler::handle(&self.db, &self.queue, &job).await
            }
            TaskType::MergeResults => {
                MergeHandler::handle(&self.db, &self.queue, &job).await
            }
            TaskType::PayrollReport => {
                // Future implementation
                Err("PayrollReport not implemented".to_string())
            }
        };

        // Report completion to coordinator
        let success = result.is_ok();
        if let Err(e) = self.coordinator.on_batch_complete(parent_id, batch_index, success).await {
            error!(
                parent_id = %parent_id,
                error = ?e,
                "Failed to report completion to coordinator"
            );
        }

        result
    }
}
```

### Step 3: Implement Attendance Handler

```rust
// worker/handlers/attendance.rs
use crate::models::{AttendanceSummary, DailyAttendanceReport, PolicyRules};
use crate::queue::{ReportQueue, ReportJob, ReportType};
use crate::repositories::{employee_repository, roster_repository, time_entry_repository};
use crate::domain::policy::resolver::PolicyResolver;
use crate::services::AttendanceCalculator;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct AttendanceHandler;

impl AttendanceHandler {
    pub async fn handle(
        pool: &PgPool,
        queue: &ReportQueue,
        job: &ReportJob,
    ) -> Result<(), String> {
        let employee_ids = &job.employee_ids;
        let start_date = job.date_range.start;
        let end_date = job.date_range.end;
        let workspace_id = job.workspace_id;
        let tz: Tz = job.config.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 1. Fetch schedules for this batch's employees
        let schedules = roster_repository::get_workspace_schedules(
            pool,
            workspace_id,
            start_date,
            end_date,
            Some(employee_ids),
        )
        .await
        .map_err(|e| format!("Failed to fetch schedules: {}", e))?;

        // 2. Convert dates to UTC for time entry query
        let start_dt = tz
            .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap())
            .single()
            .unwrap()
            .with_timezone(&Utc);

        let end_dt = tz
            .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap())
            .single()
            .unwrap()
            .with_timezone(&Utc);

        // 3. Batch fetch time entries
        let entries_map = time_entry_repository::get_workspace_time_entries_batch(
            pool,
            employee_ids,
            start_dt,
            end_dt,
        )
        .await
        .map_err(|e| format!("Failed to fetch time entries: {}", e))?;

        // 4. Calculate based on report type
        let result_json = match job.config.report_type {
            ReportType::Summary => {
                Self::calculate_summary(
                    pool,
                    &schedules,
                    &entries_map,
                    workspace_id,
                    start_date,
                    end_date,
                    &job.config,
                    tz,
                ).await?
            }
            ReportType::Detail => {
                Self::calculate_detail(
                    pool,
                    &schedules,
                    &entries_map,
                    workspace_id,
                    &job.config,
                    tz,
                ).await?
            }
        };

        // 5. Store result in Redis
        queue.store_result(job.parent_job_id, job.batch_index, &result_json)
            .await
            .map_err(|e| format!("Failed to store result: {}", e))?;

        Ok(())
    }

    async fn calculate_summary(
        pool: &PgPool,
        schedules: &[EmployeeSchedule],
        entries_map: &HashMap<Uuid, Vec<TimeEntry>>,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        config: &ReportConfig,
        tz: Tz,
    ) -> Result<String, String> {
        let mut summaries: Vec<AttendanceSummary> = Vec::new();

        for emp_sched in schedules {
            let entries = entries_map.get(&emp_sched.employee_id).cloned().unwrap_or_default();
            let mut total_units = 0i32;

            for daily in &emp_sched.schedules {
                let date = daily.date;

                let day_entries: Vec<_> = entries
                    .iter()
                    .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == date)
                    .collect();

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time.with_timezone(&tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

                if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                    let actual_break_minutes: i32 = day_entries
                        .iter()
                        .filter(|e| e.entry_type == "break")
                        .map(|e| e.duration_minutes.unwrap_or(0))
                        .sum();

                    // Resolve policy (could be cached per employee for batch)
                    let rules = PolicyResolver::resolve_attendance_rules(
                        pool,
                        emp_sched.employee_id,
                        workspace_id,
                        date,
                    ).await.unwrap_or_default();

                    // Handle shift times
                    let shift_start_dt = daily.start_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());
                    let shift_end_dt = daily.end_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());

                    let (shift_start, shift_end) = if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                        if daily.end_time < daily.start_time {
                            end += chrono::Duration::days(1);
                        }
                        (Some(start), Some(end))
                    } else {
                        (None, None)
                    };

                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        punch_in,
                        punch_out,
                        shift_start,
                        shift_end,
                        daily.break_duration_minutes.unwrap_or(0),
                        actual_break_minutes,
                        config.work_unit_minutes,
                        &config.rounding_policy,
                        &rules,
                    );

                    total_units += std_units + ot_units;
                }
            }

            summaries.push(AttendanceSummary {
                employee_id: emp_sched.employee_id,
                employee_name: emp_sched.full_name.clone(),
                total_work_units: total_units,
                total_leave_days: 0, // TODO: Fetch from leave_requests
            });
        }

        serde_json::to_string(&summaries)
            .map_err(|e| format!("Failed to serialize: {}", e))
    }

    async fn calculate_detail(
        pool: &PgPool,
        schedules: &[EmployeeSchedule],
        entries_map: &HashMap<Uuid, Vec<TimeEntry>>,
        workspace_id: Uuid,
        config: &ReportConfig,
        tz: Tz,
    ) -> Result<String, String> {
        let mut reports: Vec<DailyAttendanceReport> = Vec::new();

        // Similar to calculate_summary but generates per-day records
        // (Implementation mirrors report_service.rs get_daily_attendance_report)

        serde_json::to_string(&reports)
            .map_err(|e| format!("Failed to serialize: {}", e))
    }
}
```

### Step 4: Integrate Worker in main.rs

```rust
// main.rs additions
use apiserver::worker::{ReportWorker, WorkerConfig};
use tokio_util::sync::CancellationToken;

// In main():
let worker_cancel_token = CancellationToken::new();

let worker_handle = if let Some(redis) = &redis_pool {
    let worker_config = WorkerConfig {
        max_concurrent_jobs: std::env::var("WORKER_MAX_CONCURRENT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2),
        poll_timeout_secs: 30.0,
        max_retries: 3,
        job_timeout_secs: 300,
    };

    let coordinator = Arc::new(ReportCoordinator::new(
        redis.clone(),
        pool.clone(),
    ));

    let worker = ReportWorker::new(
        redis.clone(),
        pool.clone(),
        worker_config,
        coordinator.clone(),
    );

    // Store coordinator in AppState for API endpoints
    // state.coordinator = Some(coordinator);

    Some(worker.start(worker_cancel_token.clone()))
} else {
    None
};

// In shutdown sequence:
worker_cancel_token.cancel();
if let Some(handle) = worker_handle {
    if let Err(e) = handle.await {
        tracing::error!("Worker task panicked: {:?}", e);
    }
}
```

## Todo List

- [ ] Create worker module structure
- [ ] Implement `WorkerConfig` with env var overrides
- [ ] Implement `ReportWorker` with semaphore-controlled loop
- [ ] Implement job routing by `task_type`
- [ ] Implement `AttendanceHandler` for report calculation
- [ ] Implement graceful shutdown with permit draining
- [ ] Add job timeout handling
- [ ] Add retry logic for failed jobs
- [ ] Integrate worker startup in main.rs
- [ ] Write unit tests for handler logic
- [ ] Write integration tests with Redis

## Success Criteria

- [ ] Worker starts and polls Redis queue
- [ ] Semaphore limits concurrent jobs
- [ ] Jobs routed correctly by task_type
- [ ] Results stored in Redis after processing
- [ ] Graceful shutdown completes in-flight jobs
- [ ] Memory usage < 50MB per job

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Memory spike during large batch | Limit batch size to 20 employees |
| Job stuck in processing | Timeout + heartbeat + dead letter queue |
| Redis connection lost mid-job | Retry with backoff, reconnection handling |
| Worker crash loses job | Job remains in queue until dequeued |

## Security Considerations

- Workers should validate job payload before processing
- Don't expose raw database errors to Redis results
- Sanitize any user-provided data in job config

## Next Steps

After this phase:
1. Phase 4: Application-Side Join - Optimize data fetching
2. Phase 5: Result Aggregation - Merge job handling
