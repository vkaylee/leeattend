//! Agent Log Buffer Service
//!
//! Provides a buffered channel for agent attendance logs to avoid DB stress
//! and main app blocking. Logs are collected into a bounded channel and
//! processed by a background worker that batch-inserts to the database.

use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, timeout};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::{LogSource, RawAttendanceLog};
use crate::repositories::{employee_repository, raw_attendance_log_repository};
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
    pub async fn send(&self, entry: AgentLogEntry) -> Result<(), String> {
        // Apply backpressure with 100ms timeout
        match timeout(Duration::from_millis(100), self.sender.send(entry)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err("Buffer channel closed".to_string()),
            Err(_) => {
                warn!("Agent log buffer full, backpressure applied");
                Err("Buffer full, timeout".to_string())
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
pub async fn run_log_worker(
    pool: PgPool,
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
                match entry {
                    Some(log_entry) => {
                        batch.push(log_entry);
                        if batch.len() >= batch_size {
                            process_batch(&pool, &mut batch).await;
                        }
                    }
                    None => {
                        // Channel closed, drain remaining logs
                        info!("Agent log buffer closed, draining {} remaining logs", batch.len());
                        if !batch.is_empty() {
                            process_batch(&pool, &mut batch).await;
                        }
                        break;
                    }
                }
            }
            // Flush timer tick
            _ = flush_timer.tick() => {
                if !batch.is_empty() {
                    process_batch(&pool, &mut batch).await;
                }
            }
        }
    }

    info!("Agent log worker stopped");
}

/// Process a batch of log entries: parse, insert, and trigger pairing
async fn process_batch(pool: &PgPool, batch: &mut Vec<AgentLogEntry>) {
    if batch.is_empty() {
        return;
    }

    let batch_len = batch.len();
    let mut parsed_logs: Vec<ParsedLogEntry> = Vec::with_capacity(batch_len);
    let mut parse_errors = 0;

    // Parse and validate each entry
    for entry in batch.drain(..) {
        match parse_log_entry(pool, &entry).await {
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
            warn!("Batch had {} parse errors, no valid logs to insert", parse_errors);
        }
        return;
    }

    // Extract raw logs for batch insert
    let raw_logs: Vec<&RawAttendanceLog> = parsed_logs.iter().map(|p| &p.raw_log).collect();

    // Batch insert to database
    match raw_attendance_log_repository::create_raw_logs_batch(pool, &raw_logs).await {
        Ok(inserted_count) => {
            info!(
                "Batch insert complete: {}/{} logs inserted ({} duplicates skipped, {} parse errors)",
                inserted_count,
                batch_len,
                raw_logs.len() - inserted_count,
                parse_errors
            );

            // Trigger pairing for each successfully inserted log
            for parsed in &parsed_logs {
                if let Err(e) = PairingService::process_new_raw_log(pool, &parsed.raw_log).await {
                    error!("Pairing error for log {}: {:?}", parsed.raw_log.id, e);
                }
            }
        }
        Err(e) => {
            error!("Batch insert failed: {}", e);
            // Logs are lost on batch failure - acceptable for real-time logs
            // Agent can retry if needed
        }
    }
}

/// Parse a raw log entry into a validated RawAttendanceLog
async fn parse_log_entry(pool: &PgPool, entry: &AgentLogEntry) -> Result<ParsedLogEntry, String> {
    // Resolve employee_id from employee_id_number
    let employee = match employee_repository::find_by_number(
        pool,
        entry.workspace_id,
        &entry.employee_id_number,
    )
    .await
    {
        Ok(Some(emp)) => emp,
        Ok(None) => {
            return Err(format!(
                "Employee not found: {} in workspace {}",
                entry.employee_id_number, entry.workspace_id
            ));
        }
        Err(e) => {
            return Err(format!("Database error finding employee: {}", e));
        }
    };

    // Parse punch_time with workspace timezone
    let punch_time = parse_datetime_with_tz(
        &entry.punch_time_raw,
        &entry.timezone,
        Some(&entry.date_fmt),
        Some(&entry.time_fmt),
    )
    .map_err(|e| format!("Failed to parse timestamp '{}': {}", entry.punch_time_raw, e))?;

    let raw_log = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: entry.workspace_id,
        employee_id: employee.id,
        punch_time,
        source: LogSource::Device,
        device_id: entry.device_id.clone(),
        raw_data: entry.raw_data.clone(),
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
}
