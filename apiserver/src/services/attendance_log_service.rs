use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::{LogSource, RawAttendanceLog};
use crate::repositories::{employee_repository, raw_attendance_log_repository, attendance_log_error_repository};
use crate::services::pairing_service::PairingService;
use crate::utils::datetime::parse_datetime_with_tz;
use crate::services::{AgentLogBuffer, AgentLogEntry, agent_log_buffer};

#[derive(Clone)]
pub struct AttendanceLogService {
    #[allow(dead_code)]
    pool: PgPool,
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

    pub async fn ingest_agent_log(&self, entry: AgentLogEntry) -> Result<(), String> {
        self.log_buffer.send(entry).await
    }

    /// Processes a single attendance log entry from any source (Agent or Import).
    /// Handles employee mapping, timezone-aware parsing (if needed), and triggers pairing logic.
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
                let msg = format!("Employee not found: {} in workspace {}", employee_id_number, workspace_id);
                warn!("{}", msg);
                let _ = attendance_log_error_repository::create_error_log(
                    pool,
                    workspace_id,
                    device_id.clone(),
                    raw_data,
                    &msg,
                    &format!("{:?}", source),
                ).await;
                return Err(msg);
            }
            Err(e) => {
                let msg = format!("Database error finding employee: {}", e);
                error!("{}", msg);
                return Err(msg);
            }
        };

        // 2. Parse punch_time with workspace configuration
        let punch_time = match parse_datetime_with_tz(punch_time_raw, timezone, date_fmt, time_fmt) {
            Ok(dt) => dt,
            Err(e) => {
                let msg = format!("Failed to parse timestamp '{}': {}", punch_time_raw, e);
                error!("{}", msg);
                let _ = attendance_log_error_repository::create_error_log(
                    pool,
                    workspace_id,
                    device_id.clone(),
                    raw_data,
                    &msg,
                    &format!("{:?}", source),
                ).await;
                return Err(msg);
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
                let msg = format!("Failed to save raw attendance log: {}", e);
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
}
