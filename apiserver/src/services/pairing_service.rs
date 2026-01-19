use chrono::{DateTime, Duration, NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::models::RawAttendanceLog;
use crate::repositories::roster_repository;
use crate::services::pairing_algo::{self, PairingShiftConfig};

pub struct PairingService;

impl PairingService {
    pub async fn process_new_raw_log(
        pool: &PgPool,
        log: &RawAttendanceLog,
    ) -> Result<(), sqlx::Error> {
        // 1. Deduplication (exact match check)
        // We only skip if there is a log at the EXACT same timestamp to avoid database unique constraint or redundant work.
        let is_duplicate = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM raw_attendance_logs 
                WHERE employee_id = $1 
                AND id != $2
                AND punch_time = $3
                LIMIT 1
            )
            "#,
            log.employee_id,
            log.id,
            log.punch_time
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);

        if is_duplicate {
            info!(
                "Deduplicating log {} for employee {}. Found another log with identical timestamp.",
                log.id, log.employee_id
            );
            return Ok(());
        }

        // 2. Fetch Workspace Config
        let workspace_info = sqlx::query!(
            "SELECT timezone, pairing_algorithm FROM workspaces WHERE id = $1",
            log.workspace_id
        )
        .fetch_optional(pool)
        .await?;

        let timezone_str = workspace_info
            .as_ref()
            .map(|w| w.timezone.clone())
            .unwrap_or_else(|| "UTC".to_string());
        let algorithm = workspace_info
            .map(|w| w.pairing_algorithm)
            .unwrap_or_else(|| "anchor_first".to_string());
        let tz: Tz = timezone_str.parse().unwrap_or(chrono_tz::UTC);

        // 3. Determine Local Date
        let local_dt = log.punch_time.with_timezone(&tz);
        let local_date = local_dt.date_naive();

        // 4. Reprocess that day
        Self::reprocess_daily_attendance(pool, log.employee_id, local_date, tz, &algorithm).await?;

        // 5. Reprocess previous day if early in the morning (Overnight checkouts)
        if local_dt.hour() < 12 {
            let prev_date = local_date - Duration::days(1);
            info!(
                "Log is early in day ({}), reprocessing previous day {} for overnight checkouts",
                local_dt, prev_date
            );
            Self::reprocess_daily_attendance(pool, log.employee_id, prev_date, tz, &algorithm)
                .await?;
        }

        Ok(())
    }

    pub async fn reprocess_daily_attendance(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
        tz: Tz,
        algorithm: &str,
    ) -> Result<(), sqlx::Error> {
        info!(
            "Reprocessing attendance for emp {} on date {} using {}",
            employee_id, date, algorithm
        );

        // 1. Fetch effective schedule
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
                let start_m = s.start_time.hour() * 60 + s.start_time.minute();
                let mut end_m = s.end_time.hour() * 60 + s.end_time.minute();

                // Handle Overnight: if end < start, it's next day
                if s.end_time < s.start_time {
                    end_m += 1440;
                }

                let mut bs_m = s.break_start.map(|t| t.hour() * 60 + t.minute());
                let mut be_m = s.break_end.map(|t| t.hour() * 60 + t.minute());

                // Handle break overnight transition if needed
                if let (Some(bs), Some(be)) = (bs_m, be_m) {
                    if bs < start_m {
                        bs_m = Some(bs + 1440);
                    }
                    if be < bs {
                        be_m = Some(be + 1440);
                    }
                }

                shift_config = Some(PairingShiftConfig {
                    start_minutes: start_m as i32,
                    end_minutes: end_m as i32,
                    break_start_minutes: bs_m.map(|m| m as i32),
                    break_end_minutes: be_m.map(|m| m as i32),
                });
            }
        }

        // 2. Fetch raw logs
        // Search window: strictly bounded by shift hours + buffer to avoid cross-day stealing.
        let start_of_day_local = tz
            .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let start_of_day_utc = start_of_day_local.with_timezone(&Utc);
        
        let (window_start_utc, window_end_utc) = if let Some(s) = &shift_config {
            // Use shift start/end with a generous 6-hour buffer
            let buffer_mins = 360; // 6 hours
            let start_mins = s.start_minutes - buffer_mins;
            let end_mins = s.end_minutes + buffer_mins;
            
            let s_utc = (start_of_day_local + Duration::minutes(start_mins as i64)).with_timezone(&Utc);
            let e_utc = (start_of_day_local + Duration::minutes(end_mins as i64)).with_timezone(&Utc);
            (s_utc, e_utc)
        } else {
            // Fallback to strict 24h if no shift
            let s_utc = start_of_day_local.with_timezone(&Utc);
            let e_utc = s_utc + Duration::hours(24);
            (s_utc, e_utc)
        };

        let next_day_utc = (start_of_day_local + Duration::days(1)).with_timezone(&Utc);

        let logs = sqlx::query!(
            "SELECT id, punch_time, (raw_data->'punch')::text as punch_str FROM raw_attendance_logs WHERE employee_id = $1 AND punch_time >= $2 AND punch_time < $3 ORDER BY punch_time ASC",
            employee_id,
            window_start_utc,
            window_end_utc
        )
        .fetch_all(pool)
        .await?;

        // 3. Filter and Linearize Logs
        let mut filtered_log_events = Vec::new();
        if !logs.is_empty() {
            let mut last_minute: Option<i64> = None;

            for log in logs {
                // Deduplicate by minute
                let current_minute = log.punch_time.timestamp() / 60;
                if let Some(lm) = last_minute {
                    if current_minute == lm {
                        continue;
                    }
                }

                // Linearize: minutes since start_of_day_local
                let log_local = log.punch_time.with_timezone(&tz);
                let duration_since_start = log_local.signed_duration_since(start_of_day_local);

                // Extract punch type from metadata if present
                let punch_type = log.punch_str.and_then(|s| s.parse::<i32>().ok());

                filtered_log_events.push(pairing_algo::LogEvent {
                    id: log.id,
                    minutes: duration_since_start.num_minutes() as i32,
                    punch_type,
                });

                last_minute = Some(current_minute);
            }
        }

        if filtered_log_events.is_empty() {
            Self::clear_auto_entries(
                pool,
                employee_id,
                start_of_day_utc,
                next_day_utc,
            )
            .await?;
            return Ok(());
        }

        // 4. Run Algorithm
        let strategy = pairing_algo::get_strategy(algorithm);
        let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

        // 5. Update Database
        let mut tx = pool.begin().await?;

        // Delete existing entries starting on THIS day only (24h window)
        // This prevents reprocessing Day N from accidentally deleting morning shifts of Day N+1
        sqlx::query!(
            "DELETE FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time < $3",
            employee_id,
            start_of_day_utc,
            next_day_utc
        )
        .execute(&mut *tx)
        .await?;

        for session in sessions {
            // Calculate timestamps based on local day start to ensure accuracy
            let start_dt_local = tz.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).unwrap()
                + Duration::minutes(session.start_minutes as i64);
            let start_dt_utc = start_dt_local.with_timezone(&Utc);

            // Important: Only save sessions that actually START on the target date.
            if start_dt_local.date_naive() != date {
                continue;
            }

            let end_dt_utc = session.end_minutes.map(|m| {
                let end_dt_local = tz.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).unwrap()
                    + Duration::minutes(m as i64);
                end_dt_local.with_timezone(&Utc)
            });

            let duration = end_dt_utc.map(|end| (end - start_dt_utc).num_minutes() as i32);

            sqlx::query!(
                r#"
                INSERT INTO time_entries 
                (id, employee_id, shift_id, punch_in_time, punch_out_time, duration_minutes, entry_type, note, generation_metadata, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
                "#,
                Uuid::new_v4(),
                employee_id,
                shift_id_opt,
                start_dt_utc,
                end_dt_utc,
                duration,
                session.entry_type,
                session.note,
                session.metadata
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn clear_auto_entries(
        pool: &PgPool,
        employee_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time < $3",
            employee_id, start, end
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
