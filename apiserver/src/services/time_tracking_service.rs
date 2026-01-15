use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{PunchInRequest, PunchOutRequest, TimeEntry, UpdateTimeEntryRequest};
use crate::repositories::employee_repository;
use crate::repositories::time_entry_repository;
use crate::repositories::roster_repository;
use crate::repositories::shift_repository;
use crate::repositories::workspace_repository::{WorkspaceRepository, SqlxWorkspaceRepository};

pub struct TimeTrackingService;

impl TimeTrackingService {
    pub async fn punch_in(
        pool: &PgPool,
        workspace_id: Uuid,
        req: PunchInRequest,
    ) -> Result<TimeEntry, StatusCode> {
        // Verify employee belongs to workspace
        let employee = employee_repository::get_employee_by_id(pool, req.employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        if !employee.is_active {
            return Err(StatusCode::FORBIDDEN);
        }

        // Get Workspace Timezone
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo.get_workspace_by_id(workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?; // Should allow not found? No, foreign key usually enforces it, but safe to error.

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // Check if already punched in
        if let Some(_) = time_entry_repository::get_latest_active_entry(pool, req.employee_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            return Err(StatusCode::CONFLICT); // Already punched in
        }

        let now = Utc::now();
        let local_now = now.with_timezone(&tz);
        let today = local_now.date_naive();

        // 1. Identify effective shift
        let shift_id = roster_repository::get_effective_shift(pool, req.employee_id, today)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut final_is_break_overtime = req.is_break_overtime.unwrap_or(false);

        if let Some(sid) = shift_id {
            let shift = shift_repository::get_shift_by_id(pool, sid)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            let entry_type = req.entry_type.clone().unwrap_or_else(|| "work".to_string());
            if entry_type == "work" {
                let start_time = shift.start_time;
                let grace_minutes = shift.early_punch_in_grace_period_minutes as u32;
                
                let shift_start = today.and_time(start_time);
                let earliest_punch = shift_start - chrono::Duration::minutes(grace_minutes as i64);

                if local_now.naive_local() < earliest_punch {
                    return Err(StatusCode::BAD_REQUEST);
                }
            } else if entry_type == "break" && req.is_break_overtime.is_none() {
                final_is_break_overtime = shift.break_is_overtime;
            }
        }

        let entry = TimeEntry {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            shift_id,
            punch_in_time: now,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: final_is_break_overtime,
            entry_type: req.entry_type.unwrap_or_else(|| "work".to_string()),
            note: req.note,
            created_at: now,
            updated_at: now,
        };

        time_entry_repository::create_time_entry(pool, &entry)
            .await
            .map_err(|e| {
                eprintln!("Error punching in: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(entry)
    }

    pub async fn punch_out(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        req: PunchOutRequest,
    ) -> Result<TimeEntry, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let mut entry = time_entry_repository::get_latest_active_entry(pool, employee_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?; // Not punched in

        let punch_out_time = Utc::now();
        let duration = punch_out_time.signed_duration_since(entry.punch_in_time);

        entry.punch_out_time = Some(punch_out_time);
        entry.duration_minutes = Some(duration.num_minutes() as i32);
        if let Some(note) = req.note {
            entry.note = Some(
                format!("{}\n{}", entry.note.unwrap_or_default(), note)
                    .trim()
                    .to_string(),
            );
        }
        entry.updated_at = Utc::now();

        time_entry_repository::update_time_entry(pool, &entry)
            .await
            .map_err(|e| {
                eprintln!("Error punching out: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(entry)
    }

    pub async fn get_employee_history(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        time_entry_repository::get_employee_time_entries(pool, employee_id, start_date, end_date)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn update_time_entry(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
        req: UpdateTimeEntryRequest,
    ) -> Result<TimeEntry, StatusCode> {
        let mut entry = time_entry_repository::get_time_entry_by_id(pool, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Verify employee of this entry belongs to the workspace
        let _ = employee_repository::get_employee_by_id(pool, entry.employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::FORBIDDEN)?;

        if let Some(t) = req.punch_in_time {
            entry.punch_in_time = t;
        }
        if let Some(t) = req.punch_out_time {
            entry.punch_out_time = Some(t);
        }
        if let Some(et) = req.entry_type {
            entry.entry_type = et;
        }
        if let Some(n) = req.note {
            entry.note = Some(n);
        }

        if let Some(po) = entry.punch_out_time {
            let duration = po.signed_duration_since(entry.punch_in_time);
            entry.duration_minutes = Some(duration.num_minutes() as i32);
        }

        time_entry_repository::update_time_entry(pool, &entry)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(entry)
    }

    pub async fn delete_time_entry(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<(), StatusCode> {
        let entry = time_entry_repository::get_time_entry_by_id(pool, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Verify employee of this entry belongs to the workspace
        let _ = employee_repository::get_employee_by_id(pool, entry.employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::FORBIDDEN)?;

        time_entry_repository::delete_time_entry(pool, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }
}
