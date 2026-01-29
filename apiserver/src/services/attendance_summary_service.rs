use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AttendanceDailySummary, OtStatus, PolicyRules};
use crate::repositories::{
    roster_repository, time_entry_repository,
    SqlxWorkspaceRepository, workspace_repository::WorkspaceRepository,
};
use crate::domain::policy::resolver::PolicyResolver;
use crate::services::AttendanceCalculator;

pub struct AttendanceSummaryService;

impl AttendanceSummaryService {
    /// Recalculates and persists the daily summary for a specific employee and date.
    /// This should be called whenever time entries or schedules change.
    pub async fn recalculate_daily_summary(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), String> {
        // 1. Get Workspace Settings
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Workspace not found")?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 2. Get Effective Schedule for this specific date
        // We use the existing repo method but filter for just one employee/date implicitly
        // Optimization: Create a more specific repo method later if needed.
        let schedules = roster_repository::get_workspace_schedules(
            pool, 
            workspace_id, 
            date, 
            date, 
            Some(&vec![employee_id])
        )
        .await
        .map_err(|e| e.to_string())?;

        let emp_sched = schedules.first().ok_or("No schedule found for employee")?;
        let daily_schedule = emp_sched.schedules.iter().find(|s| s.date == date).ok_or("No schedule for date")?;

        // 3. Get Time Entries for the date (in UTC range)
        let start_dt_utc = tz.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
            .single().unwrap().with_timezone(&Utc);
        let end_dt_utc = tz.from_local_datetime(&date.and_hms_opt(23, 59, 59).unwrap())
            .single().unwrap().with_timezone(&Utc);

        let entries = time_entry_repository::get_employee_time_entries(
            pool,
            employee_id,
            start_dt_utc,
            end_dt_utc,
        )
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

        let actual_punch_in = day_entries.first().map(|e| e.punch_in_time.with_timezone(&tz));
        let actual_punch_out = day_entries.last().and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

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
            if let (Some(sched_start_time), Some(sched_end_time)) = (daily_schedule.start_time, daily_schedule.end_time) {
                let start_dt = tz.from_local_datetime(&date.and_time(sched_start_time)).single();
                let end_dt = tz.from_local_datetime(&date.and_time(sched_end_time)).single();
                
                if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                    if sched_end_time < sched_start_time {
                         end += chrono::Duration::days(1);
                    }
                    
                    // Resolve Policy Rules
                    let mut rules = PolicyResolver::resolve_attendance_rules(pool, employee_id, workspace_id, date)
                        .await
                        .unwrap_or(PolicyRules {
                            grace_period_minutes: daily_schedule.early_punch_in_grace_period_minutes.unwrap_or(0),
                            authorized_early_leave_minutes: 0,
                            is_early_leave_paid: false,
                            early_arrival_strategy: workspace.early_arrival_strategy,
                            is_break_auto_deduct: daily_schedule.is_break_auto_deduct.unwrap_or(false),
                            is_break_overtime: false,
                            allow_early_ot: daily_schedule.allow_early_ot.unwrap_or(false),
                        });
                    
                    if let Some(gp) = daily_schedule.early_punch_in_grace_period_minutes { rules.grace_period_minutes = gp; }
                    if let Some(bad) = daily_schedule.is_break_auto_deduct { rules.is_break_auto_deduct = bad; }
                    if let Some(aeo) = daily_schedule.allow_early_ot { rules.allow_early_ot = aeo; }

                    let buckets = AttendanceCalculator::calculate_buckets(
                        punch_in,
                        punch_out,
                        Some(start),
                        Some(end),
                        daily_schedule.break_duration_minutes.unwrap_or(0),
                        actual_break_minutes,
                        &rules,
                    );
                    
                    // System Detected OT = Early OT + Late OT
                    system_detected_ot_minutes = buckets.early_ot_minutes + buckets.late_ot_minutes;
                }
            }
        }

        // 5. Upsert into DB
        let existing = sqlx::query_as!(
            AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
            employee_id, date
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let new_status = match existing {
            Some(ref s) => {
                if s.ot_status == OtStatus::None && system_detected_ot_minutes > 0 {
                    OtStatus::PendingEmployee
                } else {
                    s.ot_status
                }
            },
            None => {
                if system_detected_ot_minutes > 0 {
                    OtStatus::PendingEmployee
                } else {
                    OtStatus::None
                }
            }
        };

        sqlx::query!(
            r#"
            INSERT INTO attendance_daily_summaries (
                workspace_id, employee_id, date, 
                actual_work_minutes, system_detected_ot_minutes, 
                ot_status
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (employee_id, date) DO UPDATE SET
                actual_work_minutes = EXCLUDED.actual_work_minutes,
                system_detected_ot_minutes = EXCLUDED.system_detected_ot_minutes,
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
            new_status as OtStatus
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn claim_ot(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            UPDATE attendance_daily_summaries
            SET 
                claimed_ot_minutes = $1,
                ot_reason = $2,
                ot_status = 'pending_manager',
                updated_at = CURRENT_TIMESTAMP
            WHERE employee_id = $3 AND date = $4
            "#,
            claimed_minutes,
            reason,
            employee_id,
            date
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            return Err("Record not found".to_string());
        }
        Ok(())
    }

    pub async fn approve_ot(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus // Approved or Rejected
    ) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            UPDATE attendance_daily_summaries
            SET
                approved_ot_minutes = $1,
                manager_note = $2,
                ot_status = $3,
                updated_at = CURRENT_TIMESTAMP
            WHERE employee_id = $4 AND date = $5
            "#,
            approved_minutes,
            manager_note,
            status as OtStatus,
            employee_id,
            date
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            return Err("Record not found".to_string());
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn enter_paper_ot(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: String,
        entered_by_user_id: Uuid,
        photo_url: Option<String>,
        allow_exceed: bool,
    ) -> Result<AttendanceDailySummary, String> {
        // 1. Check existing summary
        let existing = sqlx::query_as!(
            AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
            employee_id, date
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

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
        let result = sqlx::query_as!(
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
            RETURNING id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as "ot_status: OtStatus", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at
            "#,
            employee_id, date, claimed_minutes, reason,
            entered_by_user_id, now, photo_url
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result)
    }
}
