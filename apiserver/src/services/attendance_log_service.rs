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
use crate::services::pairing::PairingProcessor;
use crate::services::AgentLogEntry;
use crate::utils::datetime::parse_datetime_with_tz;
use crate::worker::pairing_scheduler::PairingNotifier;

#[derive(Clone)]
pub struct AttendanceLogService {
    employee_repo: Arc<dyn EmployeeRepository>,
    raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    staging_repo: Arc<dyn StagingLogRepository>,
    error_repo: Arc<dyn AttendanceLogErrorRepository>,
    /// Fire-and-forget notifications to the PairingScheduler after each INSERT.
    /// `None` in tests or when the scheduler is not configured.
    notifier: Option<Arc<PairingNotifier>>,
    /// Inline pairing for synchronous callers (punch-in/out, import).
    /// When set, `process_parsed_log` calls `process_new_raw_log` inline.
    /// `process_log` always uses deferred pairing via `PairingScheduler`.
    pairing_service: Option<Arc<dyn PairingProcessor>>,
}

pub struct AttendanceLogServiceDeps {
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub staging_repo: Arc<dyn StagingLogRepository>,
    pub error_repo: Arc<dyn AttendanceLogErrorRepository>,
    /// When set, `process_parsed_log` triggers inline pairing (needed by
    /// punch-in/out handlers that read the time entry immediately after).
    /// `process_log` always uses deferred pairing via `PairingScheduler`.
    pub pairing_service: Option<Arc<dyn PairingProcessor>>,
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
        deps: AttendanceLogServiceDeps,
        notifier: Option<Arc<PairingNotifier>>,
    ) -> Self {
        Self {
            employee_repo: deps.employee_repo,
            raw_log_repo: deps.raw_log_repo,
            staging_repo: deps.staging_repo,
            error_repo: deps.error_repo,
            notifier,
            pairing_service: deps.pairing_service,
        }
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

    /// Processes a single attendance log entry from any source (Agent or Import).
    ///
    /// Routing Logic (Dual-Layer Protection):
    /// - Case 1: Employee found → insert to raw_attendance_logs → Ok(true)
    /// - Case 2: Employee NOT found → insert to attendance_staging_logs → Ok(false)
    /// - Case 3: Parse/validation error → insert to attendance_log_errors → Ok(false)
    /// - Case 4: Database connection error → Err(msg) (agent should retry)
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn process_log(
        &self,
        args: ProcessLogArgs<'_>,
    ) -> Result<bool, crate::errors::AppError> {
        // Sanitize raw_data to strip null bytes that PostgreSQL jsonb rejects
        let raw_data = args.raw_data.map(AgentLogEntry::sanitize_json_value);
        let normalized_emp_id = crate::utils::normalize_employee_id(args.employee_id_number);
        // 1. Resolve employee_id from employee_id_number and workspace_id
        let employee = match self
            .employee_repo
            .find_by_number(
                &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                &normalized_emp_id,
            )
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
                return Err(crate::errors::AppError::Internal(msg));
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
                    .create_error_log_with_fingerprint(
                        &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
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

        let active_employment_id = employee.active_employment_id().ok_or_else(|| {
            crate::errors::AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment)
        })?;

        // 3. Create RawAttendanceLog
        let (punch, status) = Self::extract_punch_status(raw_data.as_ref());

        let raw_log = RawAttendanceLog::new(
            new_id(),
            args.workspace_id,
            active_employment_id,
            punch_time,
            args.source,
            args.device_id,
            raw_data,
            punch,
            status,
            Utc::now(),
        );

        // 4. Save to DB (Handle duplicates with ON CONFLICT DO NOTHING)
        match self
            .raw_log_repo
            .create_raw_log(args.workspace_id, &raw_log)
            .await
        {
            Ok(true) => {
                info!(
                    "Successfully saved log for employee {} at {} (UTC) from {:?}",
                    normalized_emp_id,
                    punch_time,
                    *raw_log.source()
                );

                // Notify PairingScheduler (fire-and-forget) — pairing is deferred
                // to a background batch sweep to avoid advisory-lock contention.
                if let Some(notifier) = &self.notifier {
                    notifier.notify(
                        args.workspace_id,
                        active_employment_id,
                        punch_time,
                    );
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
                Err(crate::errors::AppError::Internal(msg))
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
    ) -> Result<bool, crate::errors::AppError> {
        // Sanitize raw_data to strip null bytes that PostgreSQL jsonb rejects
        let raw_data = raw_data.map(AgentLogEntry::sanitize_json_value);
        let (punch, status) = Self::extract_punch_status(raw_data.as_ref());

        let raw_log = RawAttendanceLog::new(
            new_id(),
            workspace_id,
            employee_id,
            punch_time,
            source,
            device_id,
            raw_data,
            punch,
            status,
            Utc::now(),
        );

        match self
            .raw_log_repo
            .create_raw_log(workspace_id, &raw_log)
            .await
        {
            Ok(true) => {
                // Inline pairing for synchronous callers (punch-in/out, import)
                // that read time entries immediately after this call.
                if let Some(pairing) = &self.pairing_service {
                    if let Err(e) = pairing.process_new_raw_log(&raw_log).await {
                        tracing::error!(
                            log_id = %raw_log.id(),
                            error = %e,
                            "Pairing error in process_parsed_log"
                        );
                    }
                } else {
                    // No pairing service — notify PairingScheduler
                    if let Some(notifier) = &self.notifier {
                        notifier.notify(workspace_id, employee_id, punch_time);
                    }
                }
                Ok(true)
            }
            Ok(false) => Ok(false),
            Err(e) => Err(crate::errors::AppError::Internal(e.to_string())),
        }
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
