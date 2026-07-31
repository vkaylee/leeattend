use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::utils::id::new_id;

use crate::models::employment::EmploymentResolution;
use crate::models::{LogSource, RawAttendanceLog};
use crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository;
use crate::repositories::employment_repository::EmploymentRepository;
use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::staging_log_repository::{StagingLogRepository, StagingStatus};
use crate::repositories::EmployeeRepository;
use crate::services::pairing::PairingProcessor;
use crate::services::AgentLogEntry;
use crate::utils::datetime::parse_datetime_with_tz;
use crate::utils::tz::parse_workspace_tz;
use crate::worker::pairing_scheduler::PairingNotifier;

#[derive(Clone)]
pub struct AttendanceLogService {
    employee_repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn EmploymentRepository>,
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
    pub employment_repo: Arc<dyn EmploymentRepository>,
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
    pub auth_token: &'a crate::diesel_pool::TenantAuthToken,
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
    pub fn new(deps: AttendanceLogServiceDeps, notifier: Option<Arc<PairingNotifier>>) -> Self {
        Self {
            employee_repo: deps.employee_repo,
            employment_repo: deps.employment_repo,
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
    pub async fn process_log(
        &self,
        args: ProcessLogArgs<'_>,
    ) -> Result<bool, crate::errors::AppError> {
        // Destructure args to allow independent ownership of moved fields
        // (device_id, raw_data) while keeping Copy references available.
        let ProcessLogArgs {
            workspace_id,
            auth_token,
            employee_id_number,
            punch_time_raw,
            source,
            device_id,
            raw_data: raw_data_arg,
            timezone,
            date_fmt,
            time_fmt,
        } = args;

        // 0. Sanitize raw_data and normalize employee ID
        let raw_data = raw_data_arg.map(AgentLogEntry::sanitize_json_value);
        let normalized_emp_id = crate::utils::normalize_employee_id(employee_id_number);
        let fingerprint = Self::calculate_fingerprint(
            device_id.as_deref(),
            &normalized_emp_id,
            punch_time_raw,
            raw_data.as_ref(),
        );

        // 1. Resolve employee from device mapping
        let employee = match self
            .employee_repo
            .find_by_number(auth_token, &normalized_emp_id)
            .await
        {
            Ok(Some(emp)) => emp,
            Ok(None) => {
                return self
                    .stage_unknown_employee(
                        workspace_id,
                        device_id,
                        punch_time_raw,
                        timezone,
                        date_fmt,
                        time_fmt,
                        &normalized_emp_id,
                        raw_data,
                        &fingerprint,
                    )
                    .await;
            }
            Err(e) => {
                let msg = format!("Database error finding employee: {e}");
                error!("{}", msg);
                return Err(crate::errors::AppError::Internal(msg));
            }
        };

        // 2. Parse timestamp
        let punch_time = match parse_datetime_with_tz(
            punch_time_raw, timezone, date_fmt, time_fmt,
        ) {
            Ok(dt) => dt,
            Err(e) => {
                self.stage_parse_error(
                    auth_token, device_id, source, punch_time_raw,
                    raw_data, &fingerprint, &e,
                )
                .await;
                return Ok(false);
            }
        };

        // 3. Resolve employment effective on the local punch date
        let punch_tz: chrono_tz::Tz = parse_workspace_tz(timezone);
        let local_punch_date = punch_time.with_timezone(&punch_tz).date_naive();
        let Some(resolved_employment_id) = self
            .resolve_employment_or_stage(
                auth_token, workspace_id, device_id.as_deref(), source,
                &normalized_emp_id, employee.id(), local_punch_date,
                punch_time, raw_data.as_ref(), &fingerprint,
            )
            .await?
        else {
            return Ok(false); // Staged (not-found or ambiguous) — don't retry
        };

        // 4. Build and persist RawAttendanceLog
        self.build_and_persist_log(
            workspace_id, device_id, source, &normalized_emp_id,
            resolved_employment_id, punch_time, raw_data,
        )
        .await
    }

    /// Stage a log for an unknown employee (not found in the system).
    /// Returns Ok(false) — not an error, do not retry.
    async fn stage_unknown_employee(
        &self,
        workspace_id: Uuid,
        device_id: Option<String>,
        punch_time_raw: &str,
        timezone: &str,
        date_fmt: Option<&str>,
        time_fmt: Option<&str>,
        normalized_emp_id: &str,
        raw_data: Option<serde_json::Value>,
        fingerprint: &str,
    ) -> Result<bool, crate::errors::AppError> {
        let punch_time = parse_datetime_with_tz(
            punch_time_raw, timezone, date_fmt, time_fmt,
        )
        .unwrap_or_else(|_| Utc::now());

        let _ = self
            .staging_repo
            .create_staging_log(
                workspace_id,
                device_id,
                normalized_emp_id,
                punch_time,
                raw_data,
                fingerprint,
                StagingStatus::Pending,
            )
            .await;

        warn!(
            "Unknown user {} staged for workspace {} (fingerprint: {})",
            normalized_emp_id,
            workspace_id,
            &fingerprint[..8]
        );
        Ok(false)
    }

    /// Stage a parse/validation error log. Returns nothing — always Ok(false) upstream.
    async fn stage_parse_error(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        device_id: Option<String>,
        source: LogSource,
        punch_time_raw: &str,
        raw_data: Option<serde_json::Value>,
        fingerprint: &str,
        parse_error_msg: &str,
    ) {
        let msg = format!("Failed to parse timestamp '{punch_time_raw}': {parse_error_msg}");
        error!("{}", msg);

        let source_debug = format!("{source:?}");
        let _ = self
            .error_repo
            .create_error_log_with_fingerprint(
                auth_token,
                device_id,
                raw_data,
                &msg,
                &source_debug,
                fingerprint,
            )
            .await;
    }

    /// Resolve the employment record for a punch date, staging an error or
    /// ambiguous-employment record when resolution fails.
    ///
    /// Returns:
    /// - `Ok(Some(employment_id))` — resolved successfully
    /// - `Ok(None)` — staged (not-found or ambiguous); caller should return `Ok(false)`
    /// - `Err(_)` — database error; caller should propagate
    #[allow(clippy::too_many_arguments)]
    async fn resolve_employment_or_stage(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        device_id: Option<&str>,
        source: LogSource,
        normalized_emp_id: &str,
        employee_id: Uuid,
        local_punch_date: chrono::NaiveDate,
        punch_time: DateTime<Utc>,
        raw_data: Option<&serde_json::Value>,
        fingerprint: &str,
    ) -> Result<Option<Uuid>, crate::errors::AppError> {
        let resolution = match self
            .employment_repo
            .resolve_employment_on_date(auth_token, employee_id, local_punch_date)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                let msg = format!("Database error resolving employment by date: {e}");
                error!("{}", msg);
                return Err(crate::errors::AppError::Internal(msg));
            }
        };

        match resolution {
            EmploymentResolution::Resolved(emp) => Ok(Some(emp.id())),
            EmploymentResolution::NotFound => {
                let msg = format!(
                    "No employment covers punch date {punch_time} (local: {local_punch_date}) for employee {employee_id}"
                );
                warn!("{}", msg);

                let _ = self
                    .error_repo
                    .create_error_log_with_fingerprint(
                        auth_token,
                        device_id.map(String::from),
                        raw_data.cloned(),
                        &msg,
                        &format!("{source:?}"),
                        fingerprint,
                    )
                    .await;

                Ok(None)
            }
            EmploymentResolution::Ambiguous(candidates) => {
                warn!(
                    employee_id = %employee_id,
                    local_punch_date = %local_punch_date,
                    candidate_count = candidates.len(),
                    "Ambiguous employment for punch — staged for manual resolution"
                );

                self.staging_repo
                    .create_staging_log(
                        workspace_id,
                        device_id.map(String::from),
                        normalized_emp_id,
                        punch_time,
                        raw_data.cloned(),
                        fingerprint,
                        StagingStatus::Ambiguous,
                    )
                    .await?;

                Ok(None)
            }
        }
    }

    /// Build a RawAttendanceLog, persist it, and trigger post-processing
    /// (fire-and-forget pairing notification on successful insert).
    #[allow(clippy::too_many_arguments)]
    async fn build_and_persist_log(
        &self,
        workspace_id: Uuid,
        device_id: Option<String>,
        source: LogSource,
        normalized_emp_id: &str,
        employment_id: Uuid,
        punch_time: DateTime<Utc>,
        raw_data: Option<serde_json::Value>,
    ) -> Result<bool, crate::errors::AppError> {
        let (punch, status) = Self::extract_punch_status(raw_data.as_ref());
        let raw_log = RawAttendanceLog::new(
            new_id(),
            workspace_id,
            employment_id,
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
                info!(
                    "Successfully saved log for employee {} at {} (UTC) from {:?}",
                    normalized_emp_id,
                    punch_time,
                    *raw_log.source()
                );
                if let Some(notifier) = &self.notifier {
                    notifier.notify(workspace_id, employment_id, punch_time);
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
        employment_id: Uuid,
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
            employment_id,
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
                // If pairing fails, propagate the error so the caller knows the
                // time entries are not yet available and can retry.
                if let Some(pairing) = &self.pairing_service {
                    pairing.process_new_raw_log(&raw_log).await.map_err(|e| {
                        tracing::error!(
                            log_id = %raw_log.id(),
                            error = %e,
                            "Pairing error in process_parsed_log"
                        );
                        crate::errors::AppError::Internal(format!(
                            "Failed to process attendance log (id={}): {e}",
                            raw_log.id()
                        ))
                    })?;
                } else {
                    // No pairing service — notify PairingScheduler
                    if let Some(notifier) = &self.notifier {
                        notifier.notify(workspace_id, employment_id, punch_time);
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
