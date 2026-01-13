use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use tracing::info;

use crate::models::{RawAttendanceLog, TimeEntry};
use crate::repositories::{time_entry_repository, raw_attendance_log_repository};

pub struct PairingService;

impl PairingService {
    pub async fn process_new_raw_log(
        pool: &PgPool,
        log: &RawAttendanceLog,
    ) -> Result<(), sqlx::Error> {
        // 1. Deduplication: Check latest raw log for this employee
        if let Ok(Some(_last_raw)) = raw_attendance_log_repository::get_latest_log(pool, log.employee_id).await {
            // If the log we are processing IS the one we just saved, we might need to be careful
            // But usually this is called AFTER saving the new log, so we should look for the one BEFORE it.
            // Actually, the simplest way is to check if there's any log within 2 minutes EXCLUDING the current log.
            // However, process_new_raw_log is called with the log that was JUST created.
            
            // Let's just find logs in the last 2 minutes for this employee
            let recent_logs = raw_attendance_log_repository::get_paginated_raw_logs(
                pool, 
                log.workspace_id, 
                Some(log.employee_id), 
                Some(log.punch_time - Duration::minutes(2)), 
                Some(log.punch_time - Duration::seconds(1)), // Exclude current log by time
                2, 
                0
            ).await?;

            if !recent_logs.is_empty() {
                info!("Deduplicating log for employee {} at {}. Found recent log.", log.employee_id, log.punch_time);
                return Ok(());
            }
        }

        // 2. Find open time entry
        let open_entry = time_entry_repository::get_latest_active_entry(pool, log.employee_id).await?;

        if let Some(mut entry) = open_entry {
            let diff = log.punch_time - entry.punch_in_time;
            
            // If open entry is within 14 hours, treat this log as a Punch Out
            if diff > Duration::zero() && diff < Duration::hours(14) {
                info!("Pairing log as PUNCH OUT for employee {} (In: {}, Out: {})", log.employee_id, entry.punch_in_time, log.punch_time);
                entry.punch_out_time = Some(log.punch_time);
                entry.duration_minutes = Some(diff.num_minutes() as i32);
                entry.updated_at = Utc::now();
                time_entry_repository::update_time_entry(pool, &entry).await?;
                return Ok(());
            }
        }

        // 3. Otherwise, create a new time entry (Punch In)
        info!("Creating new PUNCH IN entry for employee {} at {}", log.employee_id, log.punch_time);
        let new_entry = TimeEntry {
            id: Uuid::new_v4(),
            employee_id: log.employee_id,
            shift_id: None,
            punch_in_time: log.punch_time,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: "work".to_string(),
            note: Some(format!("Auto-paired from {} log", log.source)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        time_entry_repository::create_time_entry(pool, &new_entry).await?;
        
        Ok(())
    }
}
