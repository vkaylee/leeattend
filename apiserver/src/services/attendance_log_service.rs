use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::utils::id::new_id;

use crate::models::{LogSource, RawAttendanceLog};
use crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository;
use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::staging_log_repository::StagingLogRepository;
use crate::repositories::EmployeeRepository;
use crate::services::pairing_service::PairingService;
use crate::services::{agent_log_buffer, AgentLogBuffer, AgentLogEntry};
use crate::utils::datetime::parse_datetime_with_tz;

#[derive(Clone)]
pub struct AttendanceLogService {
    employee_repo: Arc<dyn EmployeeRepository>,
    raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    staging_repo: Arc<dyn StagingLogRepository>,
    error_repo: Arc<dyn AttendanceLogErrorRepository>,
    pairing_service: Arc<PairingService>,
    #[allow(dead_code)] // Held to keep mpsc Sender alive — dropping closes the worker channel
    log_buffer: Arc<AgentLogBuffer>,
}

pub struct AttendanceLogServiceConfig {
    pub buffer_capacity: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

pub struct AttendanceLogServiceDeps {
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub staging_repo: Arc<dyn StagingLogRepository>,
    pub error_repo: Arc<dyn AttendanceLogErrorRepository>,
}

pub struct ProcessLogArgs<'a> {
    pub workspace_id: Uuid,
    pub employee_id_number: &'a str,
    pub punch_time_raw: &'a str,
    pub source: LogSource,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub timezone: &'a str,
    pub date_fmt: Option<&'a str>,
    pub time_fmt: Option<&'a str>,
}

impl AttendanceLogService {
    pub fn new(
        pairing_service: Arc<PairingService>,
        config: AttendanceLogServiceConfig,
        deps: AttendanceLogServiceDeps,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (log_buffer, log_receiver) = AgentLogBuffer::new(config.buffer_capacity);
        let log_buffer = Arc::new(log_buffer);

        let pairing_clone = Arc::clone(&pairing_service);
        let employee_repo_clone = Arc::clone(&deps.employee_repo);
        let raw_log_repo_clone = Arc::clone(&deps.raw_log_repo);
        let worker_handle = tokio::spawn(async move {
            agent_log_buffer::run_log_worker(
                employee_repo_clone,
                raw_log_repo_clone,
                pairing_clone,
                log_receiver,
                config.batch_size,
                config.flush_interval_ms,
            )
            .await;
        });

        let service = Self {
            employee_repo: deps.employee_repo,
            raw_log_repo: deps.raw_log_repo,
            staging_repo: deps.staging_repo,
            error_repo: deps.error_repo,
            pairing_service,
            log_buffer,
        };

        (service, worker_handle)
    }

    /// Extract punch/status from raw_data JSON, defaulting to -1.
    /// Delegates to shared [`crate::services::attendance_utils::extract_punch_status`].
    fn extract_punch_status(raw_data: Option<&serde_json::Value>) -> (i32, i32) {
        crate::services::attendance_utils::extract_punch_status(raw_data)
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
            if let Some(punch) = data.get("punch").and_then(serde_json::Value::as_i64) {
                hasher.update(punch.to_string());
            }
            hasher.update("|");
            if let Some(status) = data.get("status").and_then(serde_json::Value::as_i64) {
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
    #[tracing::instrument(skip_all, err)]
    pub async fn ingest_agent_log(&self, entry: AgentLogEntry) -> Result<(), String> {
        self.process_log(ProcessLogArgs {
            workspace_id: entry.workspace_id,
            employee_id_number: &entry.employee_id_number,
            punch_time_raw: &entry.punch_time_raw,
            source: LogSource::Device,
            device_id: entry.device_id,
            raw_data: entry.raw_data,
            timezone: &entry.timezone,
            date_fmt: Some(&entry.date_fmt),
            time_fmt: Some(&entry.time_fmt),
        })
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
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn process_log(&self, args: ProcessLogArgs<'_>) -> Result<bool, String> {
        // Sanitize raw_data to strip null bytes that PostgreSQL jsonb rejects
        let raw_data = args.raw_data.map(AgentLogEntry::sanitize_json_value);
        let normalized_emp_id = crate::utils::normalize_employee_id(args.employee_id_number);
        // 1. Resolve employee_id from employee_id_number and workspace_id
        let employee = match self
            .employee_repo
            .find_by_number(&crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id), &normalized_emp_id)
            .await
        {
            Ok(Some(emp)) => emp,
            Ok(None) => {
                // Case 2: Unknown user → Stage for later review (NOT an error)
                let fingerprint = Self::calculate_fingerprint(
                    args.device_id.as_deref(),
                    &normalized_emp_id,
                    args.punch_time_raw,
                    raw_data.as_ref(),
                );

                // Parse timestamp for staging (best effort)
                let punch_time = parse_datetime_with_tz(
                    args.punch_time_raw,
                    args.timezone,
                    args.date_fmt,
                    args.time_fmt,
                )
                .unwrap_or_else(|_| Utc::now());

                let _ = self
                    .staging_repo
                    .create_staging_log(
                        args.workspace_id,
                        args.device_id.clone(),
                        &normalized_emp_id,
                        punch_time,
                        raw_data,
                        &fingerprint,
                    )
                    .await;

                warn!(
                    "Unknown user {} staged for workspace {} (fingerprint: {})",
                    normalized_emp_id,
                    args.workspace_id,
                    &fingerprint[..8]
                );
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
        let punch_time = match parse_datetime_with_tz(
            args.punch_time_raw,
            args.timezone,
            args.date_fmt,
            args.time_fmt,
        ) {
            Ok(dt) => dt,
            Err(e) => {
                // Case 3: Validation error → Log with fingerprint (NOT retry)
                let msg = format!("Failed to parse timestamp '{}': {}", args.punch_time_raw, e);
                error!("{}", msg);

                let fingerprint = Self::calculate_fingerprint(
                    args.device_id.as_deref(),
                    &normalized_emp_id,
                    args.punch_time_raw,
                    raw_data.as_ref(),
                );

                let _ = self
                    .error_repo
                    .create_error_log_with_fingerprint(&crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                        args.device_id.clone(),
                        raw_data,
                        &msg,
                        &format!("{:?}", args.source),
                        &fingerprint,
                    )
                    .await;

                return Ok(false); // SUCCESS - don't retry
            }
        };

        // 3. Create RawAttendanceLog
        let (punch, status) = Self::extract_punch_status(raw_data.as_ref());

        let raw_log = RawAttendanceLog {
            id: new_id(),
            workspace_id: args.workspace_id,
            employee_id: employee.id,
            punch_time,
            source: args.source,
            device_id: args.device_id,
            raw_data,
            punch,
            status,
            created_at: Utc::now(),
        };

        // 4. Save to DB (Handle duplicates with ON CONFLICT DO NOTHING)
        match self
            .raw_log_repo
            .create_raw_log(args.workspace_id, &raw_log)
            .await
        {
            Ok(true) => {
                info!(
                    "Successfully saved log for {} ({}) at {} (UTC) from {:?}",
                    employee.full_name, normalized_emp_id, punch_time, raw_log.source
                );

                // 5. Trigger Pairing Logic
                if let Err(e) = self.pairing_service.process_new_raw_log(&raw_log).await {
                    error!("Pairing error for log {}: {:?}", raw_log.id, e);
                    // We return Ok(true) because the log WAS saved, pairing is secondary/async-ish
                }
                Ok(true)
            }
            Ok(false) => {
                info!(
                    "Duplicate log skipped for {} at {}",
                    normalized_emp_id, punch_time
                );
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
    #[tracing::instrument(skip_all, err)]
    pub async fn process_parsed_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        punch_time: DateTime<Utc>,
        source: LogSource,
        device_id: Option<String>,
        raw_data: Option<serde_json::Value>,
    ) -> Result<bool, String> {
        // Sanitize raw_data to strip null bytes that PostgreSQL jsonb rejects
        let raw_data = raw_data.map(AgentLogEntry::sanitize_json_value);
        let (punch, status) = Self::extract_punch_status(raw_data.as_ref());

        let raw_log = RawAttendanceLog {
            id: new_id(),
            workspace_id,
            employee_id,
            punch_time,
            source,
            device_id,
            raw_data,
            punch,
            status,
            created_at: Utc::now(),
        };

        match self
            .raw_log_repo
            .create_raw_log(workspace_id, &raw_log)
            .await
        {
            Ok(true) => {
                let _ = self.pairing_service.process_new_raw_log(&raw_log).await;
                Ok(true)
            }
            Ok(false) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Claim staged logs for a newly created/updated employee.
    /// Finds all pending staging logs matching the employee_id_number and moves them to raw_attendance_logs.
    /// Returns the number of logs claimed.
    #[tracing::instrument(skip_all, err)]
    pub async fn claim_staged_logs(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        employee_id_number: &str,
    ) -> Result<u32, String> {
        // 1. Find all pending staging logs for this employee_id_number
        let staged_logs = self
            .staging_repo
            .get_pending_staged_logs(workspace_id, employee_id_number)
            .await
            .map_err(|e| format!("Failed to fetch staged logs: {e}"))?;

        if staged_logs.is_empty() {
            return Ok(0);
        }

        let mut claimed_count = 0u32;
        let mut dates_to_reprocess = std::collections::HashSet::new();

        // 2. We skip true transactions for now and do best-effort, or we rely on the repos
        // Since we are decoupling from raw SQLx transactions, we'll iterate and update sequentially
        for staged in &staged_logs {
            let (punch, status) = Self::extract_punch_status(staged.raw_data.as_ref());

            let raw_log = RawAttendanceLog {
                id: new_id(),
                workspace_id,
                employee_id,
                punch_time: staged.punch_time,
                source: LogSource::Device,
                device_id: staged.device_id.clone(),
                raw_data: staged.raw_data.clone(),
                punch,
                status,
                created_at: Utc::now(),
            };

            // 3. Insert to raw_attendance_logs using repo
            match self
                .raw_log_repo
                .create_raw_log(workspace_id, &raw_log)
                .await
            {
                Ok(true) => {
                    claimed_count += 1;
                    dates_to_reprocess.insert(staged.punch_time.date_naive());

                    // Mark staging log as processed
                    let _ = self
                        .staging_repo
                        .update_staging_status(staged.id, workspace_id, "processed")
                        .await;
                }
                Ok(false) => {
                    // Duplicate - still mark as processed
                    let _ = self
                        .staging_repo
                        .update_staging_status(staged.id, workspace_id, "processed")
                        .await;
                }
                Err(e) => {
                    error!("Failed to claim staged log {}: {}", staged.id, e);
                    return Err(format!("Aborted due to DB error: {e}"));
                }
            }
        }

        // 4. Trigger pairing for all affected dates
        // Ideally we'd get tz from workspace repo, default to UTC for now
        let tz = chrono_tz::UTC;

        for date in dates_to_reprocess {
            info!(
                "Reprocessing attendance for employee {} on {}",
                employee_id, date
            );
            if let Err(e) = self
                .pairing_service
                .reprocess_daily_attendance(workspace_id, employee_id, date, tz)
                .await
            {
                error!(
                    "Pairing error for employee {} on {}: {:?}",
                    employee_id, date, e
                );
            }
        }

        info!(
            "Claimed {} staged logs for employee {}",
            claimed_count, employee_id_number
        );
        Ok(claimed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_deterministic() {
        let fp1 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_fingerprint_different_device() {
        let fp1 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some("DEV2"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_no_device() {
        let fp1 =
            AttendanceLogService::calculate_fingerprint(None, "U001", "2026-01-01 08:00:00", None);
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some(""),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        // None and "" should produce the same fingerprint
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_with_raw_data() {
        let raw = serde_json::json!({"punch": 1, "status": 2});
        let fp1 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw),
        );
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_different_punch() {
        let raw1 = serde_json::json!({"punch": 1, "status": 0});
        let raw2 = serde_json::json!({"punch": 2, "status": 0});
        let fp1 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw1),
        );
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw2),
        );
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_raw_data_no_punch_status() {
        let raw = serde_json::json!({"other": "data"});
        let fp = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw),
        );
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_fingerprint_different_user() {
        let fp1 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U002",
            "2026-01-01 08:00:00",
            None,
        );
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_different_timestamp() {
        let fp1 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 09:00:00",
            None,
        );
        assert_ne!(fp1, fp2);
    }

    // === extract_punch_status tests ===

    #[test]
    fn test_extract_punch_status_with_values() {
        let raw = serde_json::json!({"punch": 1, "status": 2});
        let (punch, status) = AttendanceLogService::extract_punch_status(Some(&raw));
        assert_eq!(punch, 1);
        assert_eq!(status, 2);
    }

    #[test]
    fn test_extract_punch_status_none() {
        let (punch, status) = AttendanceLogService::extract_punch_status(None);
        assert_eq!(punch, -1);
        assert_eq!(status, -1);
    }

    #[test]
    fn test_extract_punch_status_missing_keys() {
        let raw = serde_json::json!({"other": "data"});
        let (punch, status) = AttendanceLogService::extract_punch_status(Some(&raw));
        assert_eq!(punch, -1);
        assert_eq!(status, -1);
    }

    #[test]
    fn test_extract_punch_status_partial() {
        let raw = serde_json::json!({"punch": 3});
        let (punch, status) = AttendanceLogService::extract_punch_status(Some(&raw));
        assert_eq!(punch, 3);
        assert_eq!(status, -1);
    }

    #[test]
    fn test_extract_punch_status_zero_values() {
        let raw = serde_json::json!({"punch": 0, "status": 0});
        let (punch, status) = AttendanceLogService::extract_punch_status(Some(&raw));
        assert_eq!(punch, 0);
        assert_eq!(status, 0);
    }

    #[test]
    fn test_extract_punch_status_string_values_ignored() {
        let raw = serde_json::json!({"punch": "not_a_number", "status": "invalid"});
        let (punch, status) = AttendanceLogService::extract_punch_status(Some(&raw));
        assert_eq!(punch, -1);
        assert_eq!(status, -1);
    }

    // =====================================================================
    // Additional fingerprint edge cases
    // =====================================================================

    #[test]
    fn test_fingerprint_empty_strings() {
        let fp = AttendanceLogService::calculate_fingerprint(Some(""), "", "", None);
        assert_eq!(fp.len(), 64);
        // Should be deterministic
        let fp2 = AttendanceLogService::calculate_fingerprint(Some(""), "", "", None);
        assert_eq!(fp, fp2);
    }

    #[test]
    fn test_fingerprint_raw_data_with_only_status() {
        let raw = serde_json::json!({"status": 5});
        let fp = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw),
        );
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_fingerprint_raw_data_with_negative_values() {
        let raw = serde_json::json!({"punch": -1, "status": -1});
        let fp = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw),
        );
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_fingerprint_unicode_user_id() {
        let fp = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "員工001",
            "2026-01-01 08:00:00",
            None,
        );
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_fingerprint_different_raw_status_changes_hash() {
        let raw1 = serde_json::json!({"punch": 1, "status": 0});
        let raw2 = serde_json::json!({"punch": 1, "status": 1});
        let fp1 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw1),
        );
        let fp2 = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw2),
        );
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_none_vs_empty_raw_data() {
        let raw_empty = serde_json::json!({});
        let fp_none = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            None,
        );
        let fp_empty = AttendanceLogService::calculate_fingerprint(
            Some("DEV1"),
            "U001",
            "2026-01-01 08:00:00",
            Some(&raw_empty),
        );
        // None and {} both contribute empty string to the hash, so fingerprints are equal
        assert_eq!(fp_none, fp_empty);
    }
}
