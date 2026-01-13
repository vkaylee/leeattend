use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use tracing::info;

use crate::models::TimeEntry;
use crate::models::RawAttendanceLog;

pub struct PairingService;

impl PairingService {
    pub async fn process_new_raw_log(
        pool: &PgPool,
        log: &RawAttendanceLog,
    ) -> Result<(), sqlx::Error> {
        // 1. Deduplication: Check for any OTHER log within 1 minute
        let start_window = log.punch_time - Duration::minutes(1);
        let end_window = log.punch_time + Duration::minutes(1);

        let is_duplicate = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM raw_attendance_logs 
                WHERE employee_id = $1 
                AND id != $2
                AND punch_time >= $3
                AND punch_time <= $4
                LIMIT 1
            )
            "#,
            log.employee_id,
            log.id,
            start_window,
            end_window
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);

        if is_duplicate {
            info!("Deduplicating log {} for employee {} at {}. Found another log within 1m window.", log.id, log.employee_id, log.punch_time);
            return Ok(());
        }

        // 2. Find latest open time entry
        // Using regular query_as to be more flexible with nullable fields
        let open_entry = sqlx::query_as::<_, TimeEntry>(
            "SELECT * FROM time_entries WHERE employee_id = $1 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1"
        )
        .bind(log.employee_id)
        .fetch_optional(pool)
        .await?;

        if let Some(mut entry) = open_entry {
            let diff = log.punch_time - entry.punch_in_time;
            
            // If open entry is within 14 hours, treat this log as a Punch Out
            if diff > Duration::zero() && diff < Duration::hours(14) {
                info!("Pairing log as PUNCH OUT for employee {} (In: {}, Out: {})", log.employee_id, entry.punch_in_time, log.punch_time);
                entry.punch_out_time = Some(log.punch_time);
                entry.duration_minutes = Some(diff.num_minutes() as i32);
                entry.updated_at = Utc::now();
                
                sqlx::query(
                    "UPDATE time_entries SET punch_out_time = $1, duration_minutes = $2, updated_at = $3 WHERE id = $4"
                )
                .bind(entry.punch_out_time)
                .bind(entry.duration_minutes)
                .bind(entry.updated_at)
                .bind(entry.id)
                .execute(pool)
                .await?;

                return Ok(());
            }
        }

        // 3. Otherwise, create a new time entry (Punch In)
        info!("Creating new PUNCH IN entry for employee {} at {}", log.employee_id, log.punch_time);
        
        sqlx::query(
            r#"
            INSERT INTO time_entries 
            (id, employee_id, punch_in_time, entry_type, note, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(log.employee_id)
        .bind(log.punch_time)
        .bind("work")
        .bind(format!("Auto-paired from {} log", log.source))
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(pool)
        .await?;
        
        Ok(())
    }
}