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
        // 1. Deduplication (1 minute window)
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
            info!(
                "Deduplicating log {} for employee {}. Found another log within 1m window.",
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
            .unwrap_or_else(|| "dp_pattern".to_string());
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
        // Search window: Date 00:00 to Date+1 12:00 (36 hours) to catch overnight checkouts
        let start_of_day_local = tz
            .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let start_of_day_utc = start_of_day_local.with_timezone(&Utc);
        let end_of_window_utc = start_of_day_utc + Duration::hours(36);

        let logs = sqlx::query!(
            "SELECT punch_time, (raw_data->'punch')::text as punch_str FROM raw_attendance_logs WHERE employee_id = $1 AND punch_time >= $2 AND punch_time < $3 ORDER BY punch_time ASC",
            employee_id,
            start_of_day_utc,
            end_of_window_utc
        )
        .fetch_all(pool)
        .await?;

        // 3. Filter and Linearize Logs
        let mut filtered_log_events = Vec::new();
        if !logs.is_empty() {
            let mut last_time: Option<DateTime<Utc>> = None;

            for log in logs {
                if let Some(lt) = last_time {
                    if log.punch_time.signed_duration_since(lt) <= Duration::minutes(1) {
                        continue;
                    }
                }

                // Linearize: minutes since start_of_day_local
                let log_local = log.punch_time.with_timezone(&tz);
                let duration_since_start = log_local.signed_duration_since(start_of_day_local);

                // Extract punch type from metadata if present
                let punch_type = log.punch_str.and_then(|s| s.parse::<i32>().ok());

                filtered_log_events.push(pairing_algo::LogEvent {
                    minutes: duration_since_start.num_minutes() as i32,
                    punch_type,
                });

                last_time = Some(log.punch_time);
            }
        }

        if filtered_log_events.is_empty() {
            Self::clear_auto_entries(
                pool,
                employee_id,
                start_of_day_utc,
                start_of_day_utc + Duration::hours(24),
            )
            .await?;
            return Ok(());
        }

        // 4. Run Algorithm
        let sessions = if algorithm == "chronological"
            || (algorithm == "dp_pattern" && shift_config.is_none())
        {
            Self::pair_chronological_events(&filtered_log_events)
        } else {
            let strategy = pairing_algo::get_strategy(algorithm);
            strategy.pair(&filtered_log_events, shift_config.as_ref())
        };

        // 5. Update Database
        let mut tx = pool.begin().await?;

        // Delete existing auto entries for the search window to prevent duplicates when reprocessing
        sqlx::query!(
            "DELETE FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time < $3",
            employee_id,
            start_of_day_utc,
            end_of_window_utc
        )
        .execute(&mut *tx)
        .await?;

        for session in sessions {
            let start_dt = start_of_day_utc + Duration::minutes(session.start_minutes as i64);
            let end_dt = session
                .end_minutes
                .map(|m| start_of_day_utc + Duration::minutes(m as i64));

            let duration = end_dt.map(|end| (end - start_dt).num_minutes() as i32);

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

    fn pair_chronological_events(
        logs: &[pairing_algo::LogEvent],
    ) -> Vec<pairing_algo::PairedSession> {
        let mut sessions = Vec::new();

        let mut chunks = logs.chunks_exact(2);

        for chunk in chunks.by_ref() {
            sessions.push(pairing_algo::PairedSession {
                start_minutes: chunk[0].minutes,

                end_minutes: Some(chunk[1].minutes),

                entry_type: "work".to_string(),

                note: Some("No Shift - Auto Paired".to_string()),
            });
        }

        if let Some(last) = chunks.remainder().first() {
            sessions.push(pairing_algo::PairedSession {
                start_minutes: last.minutes,

                end_minutes: None,

                entry_type: "work".to_string(),

                note: Some("No Shift - Missing Out".to_string()),
            });
        }

        sessions
    }
}
