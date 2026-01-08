use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, DateTime};
use axum::http::StatusCode;

use crate::models::{TimeEntry, PunchInRequest, PunchOutRequest, UpdateTimeEntryRequest};
use crate::repositories::time_entry_repository;
use crate::repositories::employee_repository;

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

        // Check if already punched in
        if let Some(_) = time_entry_repository::get_latest_active_entry(pool, req.employee_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
             return Err(StatusCode::CONFLICT); // Already punched in
        }

        let entry = TimeEntry {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            punch_in_time: Utc::now(),
            punch_out_time: None,
            duration_minutes: None,
            entry_type: req.entry_type.unwrap_or_else(|| "work".to_string()),
            note: req.note,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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
            entry.note = Some(format!("{}\n{}", entry.note.unwrap_or_default(), note).trim().to_string());
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

        if let Some(t) = req.punch_in_time { entry.punch_in_time = t; }
        if let Some(t) = req.punch_out_time { entry.punch_out_time = Some(t); }
        if let Some(et) = req.entry_type { entry.entry_type = et; }
        if let Some(n) = req.note { entry.note = Some(n); }

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
