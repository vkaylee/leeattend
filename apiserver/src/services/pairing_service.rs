use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate, TimeZone, NaiveTime, DateTime};
use tracing::info;
use chrono_tz::Tz;

use crate::models::RawAttendanceLog;
use crate::services::pairing_algo::{self, PairingShiftConfig};
use crate::repositories::roster_repository;

pub struct PairingService;

impl PairingService {
    pub async fn process_new_raw_log(
        pool: &PgPool,
        log: &RawAttendanceLog,
    ) -> Result<(), sqlx::Error> {
        // 1. Deduplication (Same as before)
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
            info!("Deduplicating log {} for employee {}. Found another log within 1m window.", log.id, log.employee_id);
            return Ok(());
        }

        // 2. Fetch Workspace Timezone
        // We assume the log is already inserted by the handler/caller.
        // We need workspace info.
        let workspace_info = sqlx::query!(
            "SELECT timezone FROM workspaces WHERE id = $1",
            log.workspace_id
        )
        .fetch_optional(pool)
        .await?;

        let timezone_str = workspace_info.map(|w| w.timezone).unwrap_or_else(|| "UTC".to_string());
        let tz: Tz = timezone_str.parse().unwrap_or(chrono_tz::UTC);

        // 3. Determine Local Date
        let local_dt = log.punch_time.with_timezone(&tz);
        let local_date = local_dt.date_naive();

        // 4. Reprocess that day
        Self::reprocess_daily_attendance(pool, log.employee_id, local_date, tz).await?;

        Ok(())
    }

    pub async fn reprocess_daily_attendance(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Result<(), sqlx::Error> {
        info!("Reprocessing attendance for emp {} on date {}", employee_id, date);

        // 1. Fetch effective schedule
        // We need to construct PairingShiftConfig.
        // We reuse roster_repository logic.
        let shift_id_opt = roster_repository::get_effective_shift(pool, employee_id, date).await?;

        let mut shift_config = None;
        if let Some(shift_id) = shift_id_opt {
             let shift = sqlx::query!(
                "SELECT start_time, end_time, break_start, break_end FROM shifts WHERE id = $1",
                shift_id
             )
             .fetch_optional(pool)
             .await?;

             if let Some(s) = shift {
                 shift_config = Some(PairingShiftConfig {
                     start_time: s.start_time,
                     end_time: s.end_time,
                     break_start: s.break_start,
                     break_end: s.break_end,
                 });
             }
        }

        // 2. Fetch all raw logs for that day (in Local Time perspective)
        // To be safe, we query UTC range corresponding to that day in that TZ.
        let start_of_day = tz.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).unwrap().with_timezone(&Utc);
        let end_of_day = tz.from_local_datetime(&date.and_hms_opt(23, 59, 59).unwrap()).unwrap().with_timezone(&Utc);

        let logs = sqlx::query!(
            "SELECT punch_time FROM raw_attendance_logs WHERE employee_id = $1 AND punch_time >= $2 AND punch_time <= $3 ORDER BY punch_time ASC",
            employee_id,
            start_of_day,
            end_of_day
        )
        .fetch_all(pool)
        .await?;

        if logs.is_empty() {
            // Nothing to process, maybe delete existing auto entries?
            // Yes, if no logs, no entries.
            Self::clear_auto_entries(pool, employee_id, start_of_day, end_of_day).await?;
            return Ok(());
        }

        let log_times: Vec<NaiveTime> = logs.iter()
            .map(|l| l.punch_time.with_timezone(&tz).time())
            .collect();

        // 3. Run Algorithm
        let sessions = if let Some(config) = shift_config {
            pairing_algo::pair_logs_min_cost(&log_times, &config)
        } else {
            // No shift (Day Off or Unassigned). Use simple pairing (Flip-Flop)
            // Or treat as "Continuous" 00:00-23:59?
            // Let's use simple chronological pairing.
            Self::pair_chronological(&log_times)
        };

        // 4. Update Database
        // Start Transaction
        let mut tx = pool.begin().await?;

        // A. Delete existing entries (Caution: we might want to preserve manual notes, but for now we overwrite auto-generated ones)
        // Ideally we should distinguish "Manual" vs "Auto".
        // Assuming we overwrite all for that day.
        sqlx::query!(
            "DELETE FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time <= $3",
            employee_id,
            start_of_day,
            end_of_day
        )
        .execute(&mut *tx)
        .await?;

        // B. Insert new entries
        for session in sessions {
            // Convert NaiveTime back to UTC DateTime for that specific date
            // Handle day overflow? The algo doesn't output day overflow yet (all NaiveTime).
            // We assume everything is on 'date'.
            
            let start_dt = tz.from_local_datetime(&date.and_time(session.start)).unwrap().with_timezone(&Utc);
            
            let end_dt = session.end.map(|t| {
                 tz.from_local_datetime(&date.and_time(t)).unwrap().with_timezone(&Utc)
            });

            let duration = if let Some(end) = end_dt {
                Some((end - start_dt).num_minutes() as i32)
            } else {
                None
            };

            sqlx::query!(
                r#"
                INSERT INTO time_entries 
                (id, employee_id, shift_id, punch_in_time, punch_out_time, duration_minutes, entry_type, note, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
                "#,
                Uuid::new_v4(),
                employee_id,
                shift_id_opt,
                start_dt,
                end_dt,
                duration,
                session.entry_type,
                session.note
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn clear_auto_entries(pool: &PgPool, employee_id: Uuid, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time <= $3",
            employee_id, start, end
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    fn pair_chronological(logs: &[NaiveTime]) -> Vec<pairing_algo::PairedSession> {
        let mut sessions = Vec::new();
        let mut chunks = logs.chunks_exact(2);
        
        while let Some(chunk) = chunks.next() {
            sessions.push(pairing_algo::PairedSession {
                start: chunk[0],
                end: Some(chunk[1]),
                entry_type: "work".to_string(),
                note: Some("No Shift - Auto Paired".to_string()),
            });
        }
        
        // Handle remainder
        if let Some(last) = chunks.remainder().first() {
             sessions.push(pairing_algo::PairedSession {
                start: *last,
                end: None,
                entry_type: "work".to_string(),
                note: Some("No Shift - Missing Out".to_string()),
            });
        }
        
        sessions
    }
}
