use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::{LogSource, RawAttendanceLog};
use crate::repositories::{employee_repository, raw_attendance_log_repository, attendance_log_error_repository, staging_log_repository};
use crate::services::pairing_service::PairingService;
use crate::utils::datetime::parse_datetime_with_tz;
use crate::services::{AgentLogBuffer, AgentLogEntry, agent_log_buffer};

#[derive(Clone)]
pub struct AttendanceLogService {
    #[allow(dead_code)]
    pool: PgPool,
    #[allow(dead_code)]  // Buffer kept for potential batch operations
    log_buffer: Arc<AgentLogBuffer>,
}

impl AttendanceLogService {
    pub fn new(
        #[allow(dead_code)]
    pool: PgPool,
        buffer_capacity: usize,
        batch_size: usize,
        flush_interval_ms: u64,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (log_buffer, log_receiver) = AgentLogBuffer::new(buffer_capacity);
        let log_buffer = Arc::new(log_buffer);

        let pool_clone = pool.clone();
        let worker_handle = tokio::spawn(async move {
            agent_log_buffer::run_log_worker(
                pool_clone,
                log_receiver,
                batch_size,
                flush_interval_ms,
            )
            .await;
        });

        let service = Self {
            pool,
            log_buffer,
        };

        (service, worker_handle)
    }

    /// Calculate fingerprint for deduplication.
    /// Format: SHA256(device_id|raw_user_id|punch_time_raw|punch|status)
    fn calculate_fingerprint(
        device_id: Option<&str>,
        raw_user_id: &str,
        punch_time_raw: &str,
        raw_data: Option<&serde_json::Value>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(device_id.unwrap_or(""));
        hasher.update("|");
        hasher.update(raw_user_id);
        hasher.update("|");
        hasher.update(punch_time_raw);
        hasher.update("|");
        // Extract punch and status from raw_data if available
        if let Some(data) = raw_data {
            if let Some(punch) = data.get("punch").and_then(|p| p.as_i64()) {
                hasher.update(punch.to_string());
            }
            hasher.update("|");
            if let Some(status) = data.get("status").and_then(|s| s.as_i64()) {
                hasher.update(status.to_string());
            }
        } else {
            hasher.update("|");
        }
        format!("{:x}", hasher.finalize())
    }

    /// Process agent log synchronously to ensure ACK reflects actual parsing/DB result.
    /// Returns Ok(()) for ALL cases except database connection errors.
    /// Unknown users and validation errors are swallowed with 200 OK to prevent agent retries.
    pub async fn ingest_agent_log(&self, entry: AgentLogEntry) -> Result<(), String> {
        Self::process_log(
            &self.pool,
            entry.workspace_id,
            &entry.employee_id_number,
            &entry.punch_time_raw,
            LogSource::Device,
            entry.device_id,
            entry.raw_data,
            &entry.timezone,
            Some(&entry.date_fmt),
            Some(&entry.time_fmt),
        )
        .await
        .map(|_| ()) // Convert Result<bool, String> to Result<(), String>
    }

    /// Processes a single attendance log entry from any source (Agent or Import).
    ///
    /// Routing Logic (Dual-Layer Protection):
    /// - Case 1: Employee found → insert to raw_attendance_logs → Ok(true)
    /// - Case 2: Employee NOT found → insert to attendance_staging_logs → Ok(false)
    /// - Case 3: Parse/validation error → insert to attendance_log_errors → Ok(false)
    /// - Case 4: Database connection error → Err(msg) (agent should retry)
    #[allow(clippy::too_many_arguments)]
        pub async fn process_log(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id_number: &str,
        punch_time_raw: &str,
        source: LogSource,
        device_id: Option<String>,
        raw_data: Option<serde_json::Value>,
        // Workspace config for naive parsing
        timezone: &str,
        date_fmt: Option<&str>,
        time_fmt: Option<&str>,
    ) -> Result<bool, String> {
        // 1. Resolve employee_id from employee_id_number and workspace_id
        let employee = match employee_repository::find_by_number(pool, workspace_id, employee_id_number).await {
            Ok(Some(emp)) => emp,
            Ok(None) => {
                // Case 2: Unknown user → Stage for later review (NOT an error)
                let fingerprint = Self::calculate_fingerprint(
                    device_id.as_deref(),
                    employee_id_number,
                    punch_time_raw,
                    raw_data.as_ref(),
                );

                // Parse timestamp for staging (best effort)
                let punch_time = parse_datetime_with_tz(punch_time_raw, timezone, date_fmt, time_fmt)
                    .unwrap_or_else(|_| Utc::now());

                let _ = staging_log_repository::create_staging_log(
                    pool,
                    workspace_id,
                    device_id,
                    employee_id_number,
                    punch_time,
                    raw_data,
                    &fingerprint,
                ).await;

                warn!("Unknown user {} staged for workspace {} (fingerprint: {})",
                    employee_id_number, workspace_id, &fingerprint[..8]);
                return Ok(false); // SUCCESS - don't retry
            }
            Err(e) => {
                // Case 4: Database error → Agent should retry
                let msg = format!("Database error finding employee: {e}");
                error!("{}", msg);
                return Err(msg);
            }
        };

        // 2. Parse punch_time with workspace configuration
        let punch_time = match parse_datetime_with_tz(punch_time_raw, timezone, date_fmt, time_fmt) {
            Ok(dt) => dt,
            Err(e) => {
                // Case 3: Validation error → Log with fingerprint (NOT retry)
                let msg = format!("Failed to parse timestamp '{punch_time_raw}': {e}");
                error!("{}", msg);

                let fingerprint = Self::calculate_fingerprint(
                    device_id.as_deref(),
                    employee_id_number,
                    punch_time_raw,
                    raw_data.as_ref(),
                );

                let _ = attendance_log_error_repository::create_error_log_with_fingerprint(
                    pool,
                    workspace_id,
                    device_id,
                    raw_data,
                    &msg,
                    &format!("{source:?}"),
                    &fingerprint,
                ).await;

                return Ok(false); // SUCCESS - don't retry
            }
        };

        // 3. Create RawAttendanceLog
        let raw_log = RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id,
            employee_id: employee.id,
            punch_time,
            source,
            device_id,
            raw_data,
            created_at: Utc::now(),
        };

        // 4. Save to DB (Handle duplicates with ON CONFLICT DO NOTHING)
        match raw_attendance_log_repository::create_raw_log(pool, &raw_log).await {
            Ok(true) => {
                info!(
                    "Successfully saved log for {} ({}) at {} (UTC) from {:?}",
                    employee.full_name, employee_id_number, punch_time, raw_log.source
                );

                // 5. Trigger Pairing Logic
                if let Err(e) = PairingService::process_new_raw_log(pool, &raw_log).await {
                    error!("Pairing error for log {}: {:?}", raw_log.id, e);
                    // We return Ok(true) because the log WAS saved, pairing is secondary/async-ish
                }
                Ok(true)
            }
            Ok(false) => {
                info!("Duplicate log skipped for {} at {}", employee_id_number, punch_time);
                Ok(false)
            }
            Err(e) => {
                let msg = format!("Failed to save raw attendance log: {e}");
                error!("{}", msg);
                Err(msg)
            }
        }
    }

    /// Optimized version for batch processing where employee and workspace config are already known/cached.
    pub async fn process_parsed_log(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        punch_time: DateTime<Utc>,
        source: LogSource,
        device_id: Option<String>,
        raw_data: Option<serde_json::Value>,
    ) -> Result<bool, String> {
        let raw_log = RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id,
            employee_id,
            punch_time,
            source,
            device_id,
            raw_data,
            created_at: Utc::now(),
        };

        match raw_attendance_log_repository::create_raw_log(pool, &raw_log).await {
            Ok(true) => {
                let _ = PairingService::process_new_raw_log(pool, &raw_log).await;
                Ok(true)
            }
            Ok(false) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Claim staged logs for a newly created/updated employee.
    /// Finds all pending staging logs matching the employee_id_number and moves them to raw_attendance_logs.
    /// Returns the number of logs claimed.
    pub async fn claim_staged_logs(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        employee_id_number: &str,
    ) -> Result<u32, String> {
        // 1. Find all pending staging logs for this employee_id_number
        let staged_logs = sqlx::query_as::<_, staging_log_repository::StagingLog>(
            r#"
            SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at
            FROM attendance_staging_logs
            WHERE workspace_id = $1 AND raw_user_id = $2 AND status = 'pending'
            ORDER BY punch_time ASC
            "#
        )
        .bind(workspace_id)
        .bind(employee_id_number)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch staged logs: {e}"))?;

        if staged_logs.is_empty() {
            return Ok(0);
        }

        let mut claimed_count = 0u32;
        let mut dates_to_reprocess = std::collections::HashSet::new();

        for staged in &staged_logs {
            // 2. Create RawAttendanceLog from staging
            let raw_log = RawAttendanceLog {
                id: Uuid::new_v4(),
                workspace_id,
                employee_id,
                punch_time: staged.punch_time,
                source: LogSource::Device,
                device_id: staged.device_id.clone(),
                raw_data: staged.raw_data.clone(),
                created_at: Utc::now(),
            };

            // 3. Insert to raw_attendance_logs
            match raw_attendance_log_repository::create_raw_log(pool, &raw_log).await {
                Ok(true) => {
                    claimed_count += 1;
                    dates_to_reprocess.insert(staged.punch_time.date_naive());

                    // Mark staging log as processed
                    let _ = staging_log_repository::update_staging_status(pool, staged.id, staged.workspace_id, "processed").await;
                }
                Ok(false) => {
                    // Duplicate - still mark as processed
                    let _ = staging_log_repository::update_staging_status(pool, staged.id, staged.workspace_id, "processed").await;
                }
                Err(e) => {
                    error!("Failed to claim staged log {}: {}", staged.id, e);
                }
            }
        }

        // 4. Trigger pairing for all affected dates
        // Get workspace timezone for proper reprocessing
        let ws = crate::repositories::workspace_repository::get_workspace_by_id(pool, workspace_id)
            .await
            .ok()
            .flatten();
        let tz_str = ws.as_ref().map(|w| w.timezone.as_str()).unwrap_or("UTC");
        let tz: chrono_tz::Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);
        let algorithm = ws.as_ref().map(|w| w.pairing_algorithm.as_str()).unwrap_or("anchor_first");

        for date in dates_to_reprocess {
            info!("Reprocessing attendance for employee {} on {}", employee_id, date);
            if let Err(e) = PairingService::reprocess_daily_attendance(pool, employee_id, date, tz, algorithm).await {
                error!("Pairing error for employee {} on {}: {:?}", employee_id, date, e);
            }
        }

        info!("Claimed {} staged logs for employee {}", claimed_count, employee_id_number);
        Ok(claimed_count)
    }
}
