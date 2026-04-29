use std::sync::Arc;

use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::policy::defaults::DEFAULT_WORK_MINUTES_PER_DAY;

use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{AttendanceDailySummary, OtStatus};
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::{
    workspace_repository::WorkspaceRepository, SqlxWorkspaceRepository, TimeEntryRepository,
};
use crate::services::AttendanceCalculator;

pub struct AttendanceSummaryService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    pool: PgPool,
}

impl AttendanceSummaryService {
    pub fn new(
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        pool: PgPool,
    ) -> Self {
        Self {
            time_entry_repo,
            roster_repo,
            pool,
        }
    }

    /// Recalculates and persists the daily summary for a specific employee and date.
    /// This should be called whenever time entries or schedules change.
    #[tracing::instrument(skip_all, err)]
    pub async fn recalculate_daily_summary(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), String> {
        // 1. Get Workspace Settings
        let workspace_repo = SqlxWorkspaceRepository {
            pool: self.pool.clone(),
        };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Workspace not found")?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);

        // 2. Get Effective Schedule for this specific date
        let schedules = self
            .roster_repo
            .get_workspace_schedules(None, workspace_id, date, date, Some(&vec![employee_id]))
            .await
            .map_err(|e| e.to_string())?;

        let emp_sched = schedules.first().ok_or("No schedule found for employee")?;
        let daily_schedule = emp_sched
            .schedules
            .iter()
            .find(|s| s.date == date)
            .ok_or("No schedule for date")?;

        // 3. Get Time Entries for the date (in UTC range)
        let start_dt_utc = local_midnight_to_utc(&tz, date, 0, 0, 0)?;
        let end_dt_utc = local_midnight_to_utc(&tz, date, 23, 59, 59)?;

        let entries = self
            .time_entry_repo
            .get_employee_time_entries(workspace_id, employee_id, start_dt_utc, end_dt_utc)
            .await
            .map_err(|e| e.to_string())?;

        // Filter entries that actually belong to this date in local time
        let day_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == date)
            .collect();

        // 4. Calculate Stats
        let mut actual_work_minutes = 0;
        let mut system_detected_ot_minutes = 0;

        let actual_punch_in = day_entries
            .first()
            .map(|e| e.punch_in_time.with_timezone(&tz));
        let actual_punch_out = day_entries
            .last()
            .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

        if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
            let actual_break_minutes: i32 = day_entries
                .iter()
                .filter(|e| e.entry_type == "break")
                .map(|e| e.duration_minutes.unwrap_or(0))
                .sum();

            // Calculate raw duration for "actual_work_minutes"
            let raw_duration = punch_out.signed_duration_since(punch_in).num_minutes() as i32;
            actual_work_minutes = i32::max(0, raw_duration - actual_break_minutes);

            // Calculate OT
            if let (Some(sched_start_time), Some(sched_end_time)) =
                (daily_schedule.start_time, daily_schedule.end_time)
            {
                let start_dt = tz
                    .from_local_datetime(&date.and_time(sched_start_time))
                    .single();
                let end_dt = tz
                    .from_local_datetime(&date.and_time(sched_end_time))
                    .single();

                if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                    if sched_end_time < sched_start_time {
                        end += chrono::Duration::days(1);
                    }

                    let rules =
                        crate::domain::policy::resolver::PolicyResolver::resolve_attendance_rules(
                            &self.pool,
                            employee_id,
                            workspace_id,
                            date,
                        )
                        .await
                        .map_err(|e| e.to_string())?;

                    // Resolve Policy Rules
                    let policy_rules = PolicyResolver::resolve_with_shift_overrides(
                        &self.pool,
                        employee_id,
                        workspace_id,
                        date,
                        &crate::domain::policy::resolver::ShiftPolicyOverrides {
                            grace_period_minutes: daily_schedule
                                .early_punch_in_grace_period_minutes,
                            is_break_auto_deduct: daily_schedule.is_break_auto_deduct,
                            allow_early_ot: daily_schedule.allow_early_ot,
                            early_arrival_strategy: rules.early_arrival_strategy,
                            is_break_overtime: false,
                        },
                    )
                    .await;

                    let buckets = AttendanceCalculator::calculate_buckets(
                        punch_in,
                        punch_out,
                        Some(start),
                        Some(end),
                        daily_schedule.break_duration_minutes.unwrap_or(0),
                        actual_break_minutes,
                        &policy_rules,
                        None,
                    );

                    // System Detected OT = Early OT + Late OT
                    system_detected_ot_minutes = buckets.early_ot_minutes + buckets.late_ot_minutes;
                }
            }
        }

        // 4b. Check if this date is a holiday
        let holiday = HolidayRepository::is_holiday(&self.pool, workspace_id, date)
            .await
            .unwrap_or(None);
        let is_holiday = holiday.is_some();
        let holiday_name = holiday.as_ref().map(|h| h.name.clone());

        // 4c. Holiday leave conflict detection
        if is_holiday && actual_work_minutes > 0 {
            if let Some(ref hol) = holiday {
                let system_leave: Option<(Uuid, Uuid, String)> = sqlx::query_as(
                    r#"SELECT lr.id, lr.leave_type_id, lt.code
                       FROM leave_requests lr
                       JOIN leave_types lt ON lt.id = lr.leave_type_id
                       JOIN employees e ON e.id = lr.employee_id
                       WHERE lr.employee_id = $1
                         AND e.workspace_id = $3
                         AND lr.status IN ('approved'::leave_status_type, 'pre_approved'::leave_status_type)
                         AND lr.start_date <= $2
                         AND lr.end_date >= $2
                         AND lr.requested_by_user_id IS NULL"#,
                )
                .bind(employee_id)
                .bind(date)
                .bind(workspace_id)
                .fetch_optional(&self.pool)
                .await
                .unwrap_or(None);

                if let Some((leave_request_id, _leave_type_id, _leave_code)) = system_leave {
                    let shift_minutes = daily_schedule
                        .start_time
                        .and_then(|st| {
                            daily_schedule.end_time.map(|et| {
                                let diff = et.signed_duration_since(st).num_minutes() as i32;
                                let break_mins = daily_schedule.break_duration_minutes.unwrap_or(0);
                                if diff > 0 {
                                    diff - break_mins
                                } else {
                                    diff + 1440 - break_mins
                                }
                            })
                        })
                        .unwrap_or(DEFAULT_WORK_MINUTES_PER_DAY);

                    let threshold = hol.work_threshold_percent.unwrap_or(50);
                    let work_percent = if shift_minutes > 0 {
                        (actual_work_minutes * 100) / shift_minutes
                    } else {
                        100
                    };

                    if work_percent >= threshold {
                        let _ = sqlx::query(
                            "UPDATE leave_requests SET status = 'cancelled'::leave_status_type, updated_at = NOW() WHERE id = $1"
                        )
                        .bind(leave_request_id)
                        .execute(&self.pool)
                        .await;

                        tracing::info!(
                            employee_id = %employee_id,
                            date = %date,
                            work_percent = work_percent,
                            threshold = threshold,
                            "Holiday leave reversed: employee worked {}% (>= {}% threshold)",
                            work_percent, threshold
                        );
                    } else {
                        tracing::info!(
                            employee_id = %employee_id,
                            date = %date,
                            work_percent = work_percent,
                            threshold = threshold,
                            "Holiday leave kept: employee worked {}% (< {}% threshold)",
                            work_percent, threshold
                        );
                    }
                }
            }
        }

        // 5. Upsert into DB
        let existing: Option<AttendanceDailySummary> = sqlx::query_as!(
            AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND workspace_id = $2 AND date = $3",
            employee_id, workspace_id, date
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        let new_status = match existing {
            Some(ref s) => {
                if s.ot_status == OtStatus::None && system_detected_ot_minutes > 0 {
                    OtStatus::PendingEmployee
                } else {
                    s.ot_status
                }
            }
            None => {
                if system_detected_ot_minutes > 0 {
                    OtStatus::PendingEmployee
                } else {
                    OtStatus::None
                }
            }
        };

        let _result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            INSERT INTO attendance_daily_summaries (
                workspace_id, employee_id, date, 
                actual_work_minutes, system_detected_ot_minutes, 
                ot_status, is_holiday, holiday_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (employee_id, date) DO UPDATE SET
                actual_work_minutes = EXCLUDED.actual_work_minutes,
                system_detected_ot_minutes = EXCLUDED.system_detected_ot_minutes,
                is_holiday = EXCLUDED.is_holiday,
                holiday_name = EXCLUDED.holiday_name,
                ot_status = CASE 
                    WHEN attendance_daily_summaries.ot_status = 'none'::ot_status_type AND EXCLUDED.system_detected_ot_minutes > 0 THEN 'pending_employee'::ot_status_type
                    ELSE attendance_daily_summaries.ot_status 
                END,
                updated_at = CURRENT_TIMESTAMP
            "#,
            workspace_id,
            employee_id,
            date,
            actual_work_minutes,
            system_detected_ot_minutes,
            new_status as OtStatus,
            is_holiday,
            holiday_name as Option<String>
        )
        .execute(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn claim_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<(), String> {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            UPDATE attendance_daily_summaries
            SET 
                claimed_ot_minutes = $1,
                ot_reason = $2,
                ot_status = 'pending_manager',
                updated_at = CURRENT_TIMESTAMP
            WHERE employee_id = $3 AND workspace_id = $4 AND date = $5
            "#,
            claimed_minutes,
            reason,
            employee_id,
            workspace_id,
            date
        )
        .execute(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        if result.rows_affected() == 0 {
            return Err("Record not found".to_string());
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus, // Approved or Rejected
    ) -> Result<(), String> {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            UPDATE attendance_daily_summaries
            SET
                approved_ot_minutes = $1,
                manager_note = $2,
                ot_status = $3,
                updated_at = CURRENT_TIMESTAMP
            WHERE employee_id = $4 AND workspace_id = $5 AND date = $6
            "#,
            approved_minutes,
            manager_note,
            status as OtStatus,
            employee_id,
            workspace_id,
            date
        )
        .execute(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        if result.rows_affected() == 0 {
            return Err("Record not found".to_string());
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip_all, err)]
    pub async fn enter_paper_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: String,
        entered_by_user_id: Uuid,
        photo_url: Option<String>,
        allow_exceed: bool,
    ) -> Result<AttendanceDailySummary, String> {
        // 1. Check existing summary
        let existing: Option<AttendanceDailySummary> = sqlx::query_as!(
            AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND workspace_id = $2 AND date = $3",
            employee_id, workspace_id, date
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        // 2. Validate against system detected (if not allowing exceed)
        if !allow_exceed {
            if let Some(ref ex) = existing {
                if claimed_minutes > ex.system_detected_ot_minutes {
                    return Err(format!(
                        "Claimed {} minutes exceeds system detected {} minutes",
                        claimed_minutes, ex.system_detected_ot_minutes
                    ));
                }
            }
        }

        let now = Utc::now();

        // 3. Upsert with paper form metadata
        let result: AttendanceDailySummary = sqlx::query_as!(
            AttendanceDailySummary,
            r#"
            INSERT INTO attendance_daily_summaries (
                workspace_id, employee_id, date,
                claimed_ot_minutes, ot_reason, ot_status,
                claim_entered_by_user_id, claim_entered_at,
                paper_form_photo_url, entered_via
            )
            SELECT
                e.workspace_id, $1, $2,
                $3, $4, 'pending_manager',
                $5, $6, $7, 'paper_form'
            FROM employees e WHERE e.id = $1
            ON CONFLICT (employee_id, date) DO UPDATE SET
                claimed_ot_minutes = EXCLUDED.claimed_ot_minutes,
                ot_reason = EXCLUDED.ot_reason,
                ot_status = 'pending_manager',
                claim_entered_by_user_id = EXCLUDED.claim_entered_by_user_id,
                claim_entered_at = EXCLUDED.claim_entered_at,
                paper_form_photo_url = EXCLUDED.paper_form_photo_url,
                entered_via = 'paper_form',
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as "ot_status: OtStatus", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at
            "#,
            employee_id, date, claimed_minutes, reason,
            entered_by_user_id, now, photo_url
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        Ok(result)
    }
}

use crate::utils::chrono_helpers::local_to_utc as local_midnight_to_utc;
