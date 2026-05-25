//! Agent Log Buffer Service
//!
//! Provides a buffered channel for agent attendance logs to avoid DB stress
//! and main app blocking. Logs are collected into a bounded channel and
//! processed by a background worker that batch-inserts to the database.

use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, timeout};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::{LogSource, RawAttendanceLog};
use crate::utils::id::new_id;

use crate::repositories::{
    employee::EmployeeRepository, raw_attendance_log_repository::RawAttendanceLogRepository,
};
use crate::services::pairing_service::PairingService;
use crate::utils::datetime::parse_datetime_with_tz;

/// Entry representing a raw attendance log from an agent
#[derive(Debug, Clone)]
pub struct AgentLogEntry {
    pub workspace_id: Uuid,
    pub employee_id_number: String,
    pub punch_time_raw: String,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub timezone: String,
    pub date_fmt: String,
    pub time_fmt: String,
}

pub struct AgentLogEntryArgs {
    pub workspace_id: Uuid,
    pub employee_id_number: String,
    pub punch_time_raw: String,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub timezone: String,
    pub date_fmt: String,
    pub time_fmt: String,
}

impl AgentLogEntry {
    pub fn new(args: AgentLogEntryArgs) -> Self {
        Self {
            workspace_id: args.workspace_id,
            employee_id_number: Self::sanitize_string(args.employee_id_number),
            punch_time_raw: Self::sanitize_string(args.punch_time_raw),
            device_id: args.device_id.map(Self::sanitize_string),
            raw_data: args.raw_data.map(Self::sanitize_json_value),
            timezone: args.timezone,
            date_fmt: args.date_fmt,
            time_fmt: args.time_fmt,
        }
    }

    fn sanitize_string(s: String) -> String {
        s.replace('\0', "").trim().to_string()
    }

    /// Recursively strip null bytes (\u0000) from all string values in a JSON tree.
    /// PostgreSQL jsonb columns reject \u0000 escape sequences.
    pub fn sanitize_json_value(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => serde_json::Value::String(s.replace('\0', "")),
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(Self::sanitize_json_value).collect())
            }
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.into_iter()
                    .map(|(k, v)| (k.replace('\0', ""), Self::sanitize_json_value(v)))
                    .collect(),
            ),
            other => other, // Number, Bool, Null are fine
        }
    }
}

/// Buffered channel for agent logs with bounded capacity
#[derive(Clone)]
pub struct AgentLogBuffer {
    sender: mpsc::Sender<AgentLogEntry>,
}

impl AgentLogBuffer {
    /// Create a new buffer with the given capacity
    /// Returns the buffer and the receiver for the worker
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<AgentLogEntry>) {
        let (sender, receiver) = mpsc::channel(capacity);
        (Self { sender }, receiver)
    }

    /// Send a log entry to the buffer
    /// Uses timeout to apply backpressure if buffer is full
    #[tracing::instrument(skip_all, err)]
    pub async fn send(&self, entry: AgentLogEntry) -> Result<(), crate::errors::AppError> {
        // Apply backpressure with 100ms timeout
        match timeout(Duration::from_millis(100), self.sender.send(entry)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(crate::errors::AppError::Internal("Buffer channel closed".to_string())),
            Err(_) => {
                warn!("Agent log buffer full, backpressure applied");
                Err(crate::errors::AppError::Internal("Buffer full, timeout".to_string()))
            }
        }
    }

    /// Check if the buffer is closed
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

/// Parsed log entry ready for DB insertion
struct ParsedLogEntry {
    raw_log: RawAttendanceLog,
}

/// Background worker that processes agent logs from the buffer
/// and batch-inserts them into the database
#[tracing::instrument(skip_all)]
pub async fn run_log_worker(
    employee_repo: Arc<dyn EmployeeRepository>,
    raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pairing_service: Arc<PairingService>,
    mut receiver: mpsc::Receiver<AgentLogEntry>,
    batch_size: usize,
    flush_interval_ms: u64,
) {
    info!(
        "Agent log worker started (batch_size={}, flush_interval={}ms)",
        batch_size, flush_interval_ms
    );

    let mut batch: Vec<AgentLogEntry> = Vec::with_capacity(batch_size);
    let mut flush_timer = interval(Duration::from_millis(flush_interval_ms));

    loop {
        tokio::select! {
            // Receive log entry from channel
            entry = receiver.recv() => {
                if let Some(log_entry) = entry {
                    batch.push(log_entry);
                    if batch.len() >= batch_size {
                        process_batch(&*employee_repo, &*raw_log_repo, &pairing_service, &mut batch).await;
                    }
                } else {
                    // Channel closed, drain remaining logs
                    info!("Agent log buffer closed, draining {} remaining logs", batch.len());
                    if !batch.is_empty() {
                        process_batch(&*employee_repo, &*raw_log_repo, &pairing_service, &mut batch).await;
                    }
                    break;
                }
            }
            // Flush timer tick
            _ = flush_timer.tick() => {
                if !batch.is_empty() {
                    process_batch(&*employee_repo, &*raw_log_repo, &pairing_service, &mut batch).await;
                }
            }
        }
    }

    info!("Agent log worker stopped");
}

/// Process a batch of log entries: parse, insert, and trigger pairing
async fn process_batch(
    employee_repo: &dyn EmployeeRepository,
    raw_log_repo: &dyn RawAttendanceLogRepository,
    pairing_service: &PairingService,
    batch: &mut Vec<AgentLogEntry>,
) {
    if batch.is_empty() {
        return;
    }

    let batch_len = batch.len();
    let mut parsed_logs: Vec<ParsedLogEntry> = Vec::with_capacity(batch_len);
    let mut parse_errors = 0;

    // Parse and validate each entry
    for entry in batch.drain(..) {
        match parse_log_entry(employee_repo, &entry).await {
            Ok(parsed) => parsed_logs.push(parsed),
            Err(e) => {
                parse_errors += 1;
                warn!(
                    "Failed to parse log entry for employee {}: {}",
                    entry.employee_id_number, e
                );
            }
        }
    }

    if parsed_logs.is_empty() {
        if parse_errors > 0 {
            warn!(
                "Batch had {} parse errors, no valid logs to insert",
                parse_errors
            );
        }
        return;
    }

    // Group logs by workspace_id
    use std::collections::HashMap;
    let mut logs_by_workspace: HashMap<Uuid, Vec<&RawAttendanceLog>> = HashMap::new();
    for parsed in &parsed_logs {
        logs_by_workspace
            .entry(parsed.raw_log.workspace_id)
            .or_default()
            .push(&parsed.raw_log);
    }

    // Batch insert to database per workspace
    for (workspace_id, raw_logs) in logs_by_workspace {
        match raw_log_repo
            .create_raw_logs_batch(workspace_id, &raw_logs)
            .await
        {
            Ok(inserted_count) => {
                info!(
                    "Batch insert complete for workspace {}: {}/{} logs inserted",
                    workspace_id,
                    inserted_count,
                    raw_logs.len(),
                );
            }
            Err(e) => {
                error!("Batch insert failed for workspace {}: {}", workspace_id, e);
                // Logs are lost on batch failure - acceptable for real-time logs
            }
        }
    }

    // Trigger pairing for each successfully parsed log
    for parsed in &parsed_logs {
        if let Err(e) = pairing_service.process_new_raw_log(&parsed.raw_log).await {
            error!("Pairing error for log {}: {:?}", parsed.raw_log.id, e);
        }
    }
}

/// Parse a raw log entry into a validated RawAttendanceLog
async fn parse_log_entry(
    employee_repo: &dyn EmployeeRepository,
    entry: &AgentLogEntry,
) -> Result<ParsedLogEntry, crate::errors::AppError> {
    // Resolve employee_id from employee_id_number
    let employee = match employee_repo
        .find_by_number(
            &crate::diesel_pool::TenantAuthToken::system_override(entry.workspace_id),
            &entry.employee_id_number,
        )
        .await
    {
        Ok(Some(emp)) => emp,
        Ok(None) => {
            return Err(crate::errors::AppError::Validation(format!(
                "Employee not found: {} in workspace {}",
                entry.employee_id_number, entry.workspace_id
            )));
        }
        Err(e) => {
            return Err(crate::errors::AppError::Internal(format!("Database error finding employee: {e}")));
        }
    };

    // Parse punch_time with workspace timezone
    let punch_time = parse_datetime_with_tz(
        &entry.punch_time_raw,
        &entry.timezone,
        Some(&entry.date_fmt),
        Some(&entry.time_fmt),
    )
    .map_err(|e| {
        crate::errors::AppError::Validation(format!(
            "Failed to parse timestamp '{}': {}",
            entry.punch_time_raw, e
        ))
    })?;

    let (punch, status) =
        crate::services::attendance_utils::extract_punch_status(entry.raw_data.as_ref());

    let raw_log = RawAttendanceLog {
        id: new_id(),
        workspace_id: entry.workspace_id,
        employee_id: employee.id,
        punch_time,
        source: LogSource::Device,
        device_id: entry.device_id.clone(),
        raw_data: entry.raw_data.clone(),
        punch,
        status,
        created_at: Utc::now(),
    };

    Ok(ParsedLogEntry { raw_log })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buffer_send_receive() {
        let (buffer, mut receiver) = AgentLogBuffer::new(10);

        let entry = AgentLogEntry {
            workspace_id: Uuid::new_v4(),
            employee_id_number: "EMP001".to_string(),
            punch_time_raw: "2026-01-26 09:00:00".to_string(),
            device_id: Some("D1".to_string()),
            raw_data: None,
            timezone: "UTC".to_string(),
            date_fmt: "YYYY-MM-DD".to_string(),
            time_fmt: "24h".to_string(),
        };

        buffer.send(entry.clone()).await.unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.employee_id_number, "EMP001");
    }

    #[tokio::test]
    async fn test_buffer_closes_properly() {
        let (buffer, mut receiver) = AgentLogBuffer::new(10);

        drop(buffer);

        // Receiver should return None when sender is dropped
        assert!(receiver.recv().await.is_none());
    }

    #[test]
    fn test_sanitize_agent_log_entry() {
        let entry = AgentLogEntry::new(AgentLogEntryArgs {
            workspace_id: Uuid::new_v4(),
            employee_id_number: "EMP\x00001 ".to_string(), // Null byte and trailing space
            punch_time_raw: "2026-01-26\x00 09:00:00".to_string(),
            device_id: Some("D\x001".to_string()),
            raw_data: None,
            timezone: "UTC".to_string(),
            date_fmt: "YYYY-MM-DD".to_string(),
            time_fmt: "24h".to_string(),
        });

        assert_eq!(entry.employee_id_number, "EMP001");
        assert_eq!(entry.punch_time_raw, "2026-01-26 09:00:00");
        assert_eq!(entry.device_id, Some("D1".to_string()));
    }

    #[test]
    fn test_sanitize_json_value_strips_null_bytes() {
        let raw = serde_json::json!({
            "name": "test\u{0000}value",
            "punch": 0,
            "nested": {
                "field": "hello\u{0000}world"
            },
            "list": ["a\u{0000}b", "clean"]
        });

        let sanitized = AgentLogEntry::sanitize_json_value(raw);

        assert_eq!(sanitized["name"], "testvalue");
        assert_eq!(sanitized["punch"], 0);
        assert_eq!(sanitized["nested"]["field"], "helloworld");
        assert_eq!(sanitized["list"][0], "ab");
        assert_eq!(sanitized["list"][1], "clean");
    }

    #[test]
    fn test_sanitize_raw_data_in_entry() {
        let raw = serde_json::json!({
            "user_id": "emp\u{0000}001",
            "punch": 0,
            "status": 1
        });

        let entry = AgentLogEntry::new(AgentLogEntryArgs {
            workspace_id: Uuid::new_v4(),
            employee_id_number: "EMP001".to_string(),
            punch_time_raw: "2026-01-26 09:00:00".to_string(),
            device_id: None,
            raw_data: Some(raw),
            timezone: "UTC".to_string(),
            date_fmt: "YYYY-MM-DD".to_string(),
            time_fmt: "24h".to_string(),
        });

        let data = entry.raw_data.unwrap();
        assert_eq!(data["user_id"], "emp001");
        assert_eq!(data["punch"], 0);
    }

    // =====================================================================
    // Additional edge case tests
    // =====================================================================

    #[test]
    fn test_sanitize_json_value_null_and_bool() {
        let raw = serde_json::json!({
            "flag": true,
            "nothing": null,
            "count": 42
        });
        let sanitized = AgentLogEntry::sanitize_json_value(raw);
        assert_eq!(sanitized["flag"], true);
        assert!(sanitized["nothing"].is_null());
        assert_eq!(sanitized["count"], 42);
    }

    #[test]
    fn test_sanitize_json_value_deeply_nested() {
        let raw = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": "deep\u{0000}value"
                }
            }
        });
        let sanitized = AgentLogEntry::sanitize_json_value(raw);
        assert_eq!(sanitized["level1"]["level2"]["level3"], "deepvalue");
    }

    #[test]
    fn test_sanitize_json_value_empty_string() {
        let raw = serde_json::json!({"key": ""});
        let sanitized = AgentLogEntry::sanitize_json_value(raw);
        assert_eq!(sanitized["key"], "");
    }

    #[test]
    fn test_sanitize_string_only_null_bytes() {
        let entry = AgentLogEntry::new(AgentLogEntryArgs {
            workspace_id: Uuid::new_v4(),
            employee_id_number: "\0\0\0".to_string(),
            punch_time_raw: "\0".to_string(),
            device_id: None,
            raw_data: None,
            timezone: "UTC".to_string(),
            date_fmt: "YYYY-MM-DD".to_string(),
            time_fmt: "24h".to_string(),
        });
        assert_eq!(entry.employee_id_number, "");
        assert_eq!(entry.punch_time_raw, "");
    }

    #[tokio::test]
    async fn test_buffer_capacity_one() {
        let (buffer, mut receiver) = AgentLogBuffer::new(1);

        let entry = AgentLogEntry {
            workspace_id: Uuid::new_v4(),
            employee_id_number: "EMP001".to_string(),
            punch_time_raw: "2026-01-26 09:00:00".to_string(),
            device_id: None,
            raw_data: None,
            timezone: "UTC".to_string(),
            date_fmt: "YYYY-MM-DD".to_string(),
            time_fmt: "24h".to_string(),
        };

        buffer.send(entry.clone()).await.unwrap();
        // Buffer is full now (capacity=1)

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.employee_id_number, "EMP001");
    }

    #[test]
    fn test_buffer_is_closed_after_drop() {
        let (buffer, _receiver) = AgentLogBuffer::new(10);
        assert!(!buffer.is_closed());
        // Drop receiver → sender still works technically
        // But is_closed checks if receiver is gone
        drop(_receiver);
        assert!(buffer.is_closed());
    }
}
