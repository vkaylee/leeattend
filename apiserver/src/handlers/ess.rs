use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{Datelike, NaiveDate};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::EmployeeContext;
use crate::models::{
    CreateEmergencyContactRequest, Employee, EmployeeDocument, EmployeeEmergencyContact,
    EmployeeProfile, MyProfileResponse, TimeEntry, UpdateEmployeeProfileRequest,
};
use crate::AppState;

// --- Profile Endpoints ---

/// Get the current employee's complete profile
pub async fn get_my_profile(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<MyProfileResponse>> {
    // Get employee record
    let emp = sqlx::query_as::<_, Employee>(
        r#"
        SELECT e.*,
               ep.hire_date as "hire_date",
               ep.termination_date as "termination_date"
        FROM employees e
        LEFT JOIN employment_periods ep ON ep.employee_id = e.id
            AND ep.id = (
                SELECT id FROM employment_periods
                WHERE employee_id = e.id
                ORDER BY hire_date DESC LIMIT 1
            )
        WHERE e.id = $1
        "#,
    )
    .bind(employee.employee_id)
    .fetch_one(state.db.api())
    .await?;

    // Get profile (may not exist yet)
    let profile = sqlx::query_as::<_, EmployeeProfile>(
        "SELECT * FROM employee_profiles WHERE employee_id = $1",
    )
    .bind(employee.employee_id)
    .fetch_optional(state.db.api())
    .await?;

    // Get emergency contacts
    let contacts = sqlx::query_as::<_, EmployeeEmergencyContact>(
        "SELECT * FROM employee_emergency_contacts WHERE employee_id = $1 ORDER BY is_primary DESC, created_at",
    )
    .bind(employee.employee_id)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(MyProfileResponse {
        employee: emp,
        profile,
        emergency_contacts: contacts,
    }))
}

/// Update the current employee's profile
pub async fn update_my_profile(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Json(req): Json<UpdateEmployeeProfileRequest>,
) -> AppResult<Json<EmployeeProfile>> {
    // Upsert profile (create if not exists, update if exists)
    let profile = sqlx::query_as::<_, EmployeeProfile>(
        r#"
        INSERT INTO employee_profiles (employee_id, phone, address, date_of_birth, gender, national_id, bank_name, bank_account_number, bank_branch)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (employee_id) DO UPDATE SET
            phone = COALESCE(EXCLUDED.phone, employee_profiles.phone),
            address = COALESCE(EXCLUDED.address, employee_profiles.address),
            date_of_birth = COALESCE(EXCLUDED.date_of_birth, employee_profiles.date_of_birth),
            gender = COALESCE(EXCLUDED.gender, employee_profiles.gender),
            national_id = COALESCE(EXCLUDED.national_id, employee_profiles.national_id),
            bank_name = COALESCE(EXCLUDED.bank_name, employee_profiles.bank_name),
            bank_account_number = COALESCE(EXCLUDED.bank_account_number, employee_profiles.bank_account_number),
            bank_branch = COALESCE(EXCLUDED.bank_branch, employee_profiles.bank_branch),
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(employee.employee_id)
    .bind(&req.phone)
    .bind(&req.address)
    .bind(req.date_of_birth)
    .bind(&req.gender)
    .bind(&req.national_id)
    .bind(&req.bank_name)
    .bind(&req.bank_account_number)
    .bind(&req.bank_branch)
    .fetch_one(state.db.api())
    .await?;

    Ok(Json(profile))
}

// --- Emergency Contacts Endpoints ---

/// Get all emergency contacts for the current employee
pub async fn get_my_emergency_contacts(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<EmployeeEmergencyContact>>> {
    let contacts = sqlx::query_as::<_, EmployeeEmergencyContact>(
        "SELECT * FROM employee_emergency_contacts WHERE employee_id = $1 ORDER BY is_primary DESC, created_at",
    )
    .bind(employee.employee_id)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(contacts))
}

/// Add a new emergency contact
pub async fn add_emergency_contact(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Json(req): Json<CreateEmergencyContactRequest>,
) -> AppResult<impl IntoResponse> {
    let is_primary = req.is_primary.unwrap_or(false);

    // If setting as primary, unset other primary contacts first
    if is_primary {
        sqlx::query(
            "UPDATE employee_emergency_contacts SET is_primary = false WHERE employee_id = $1",
        )
        .bind(employee.employee_id)
        .execute(state.db.api())
        .await?;
    }

    let contact = sqlx::query_as::<_, EmployeeEmergencyContact>(
        r#"
        INSERT INTO employee_emergency_contacts (employee_id, contact_name, relationship, phone, is_primary)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(employee.employee_id)
    .bind(&req.contact_name)
    .bind(&req.relationship)
    .bind(&req.phone)
    .bind(is_primary)
    .fetch_one(state.db.api())
    .await?;

    Ok((StatusCode::CREATED, Json(contact)))
}

/// Delete an emergency contact
pub async fn delete_emergency_contact(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Path(contact_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Verify ownership and delete
    let result =
        sqlx::query("DELETE FROM employee_emergency_contacts WHERE id = $1 AND employee_id = $2")
            .bind(contact_id)
            .bind(employee.employee_id)
            .execute(state.db.api())
            .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Contact not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// --- Documents Endpoints ---

/// Get all documents for the current employee
pub async fn get_my_documents(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<EmployeeDocument>>> {
    let docs = sqlx::query_as::<_, EmployeeDocument>(
        "SELECT * FROM employee_documents WHERE employee_id = $1 AND is_active = true ORDER BY created_at DESC",
    )
    .bind(employee.employee_id)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(docs))
}

// --- Payslips Endpoints ---

/// Get all payslips for the current employee
pub async fn get_my_payslips(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<crate::models::Payslip>>> {
    let payslips = sqlx::query_as::<_, crate::models::Payslip>(
        "SELECT * FROM payslips WHERE employee_id = $1 ORDER BY period_start DESC",
    )
    .bind(employee.employee_id)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(payslips))
}

// --- Attendance Endpoints ---

#[derive(serde::Deserialize)]
pub struct AttendanceQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

/// Get attendance history for the current employee
pub async fn get_my_attendance(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    axum::extract::Query(query): axum::extract::Query<AttendanceQuery>,
) -> AppResult<Json<Vec<TimeEntry>>> {
    let start = query
        .start_date
        .unwrap_or_else(|| chrono::Utc::now().date_naive() - chrono::Duration::days(30));
    let end = query
        .end_date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let entries = sqlx::query_as::<_, TimeEntry>(
        r#"
        SELECT * FROM time_entries
        WHERE employee_id = $1
          AND punch_in_time >= $2::date
          AND punch_in_time < ($3::date + INTERVAL '1 day')
        ORDER BY punch_in_time DESC
        "#,
    )
    .bind(employee.employee_id)
    .bind(start)
    .bind(end)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(entries))
}

// --- Leave Endpoints ---

/// ESS Leave Balance Response
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct EssLeaveBalance {
    pub leave_type: String,
    pub entitlement: i32,
    pub accrued_amount: f64,
    pub carried_over: i32,
    pub used: i64,
    pub remaining: f64,
}

/// Get leave balances for the current employee
pub async fn get_my_leave_balances(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<EssLeaveBalance>>> {
    let year = chrono::Utc::now().year();

    let balances = sqlx::query_as::<_, EssLeaveBalance>(
        r#"
        SELECT
            lt.code as leave_type,
            lb.entitlement,
            CAST(lb.accrued_amount AS FLOAT8) as accrued_amount,
            lb.carried_over,
            COALESCE(
                (SELECT COUNT(*) FROM leave_requests lr
                 WHERE lr.employee_id = $1
                   AND lr.leave_type_id = lb.leave_type_id
                   AND lr.status = 'approved'
                   AND CAST(EXTRACT(YEAR FROM lr.start_date) AS INTEGER) = $2),
                0
            ) as used,
            CAST(lb.entitlement + lb.accrued_amount + lb.carried_over AS FLOAT8) - COALESCE(
                (SELECT COUNT(*) FROM leave_requests lr
                 WHERE lr.employee_id = $1
                   AND lr.leave_type_id = lb.leave_type_id
                   AND lr.status = 'approved'
                   AND CAST(EXTRACT(YEAR FROM lr.start_date) AS INTEGER) = $2),
                0
            ) as remaining
        FROM leave_balances lb
        JOIN leave_types lt ON lb.leave_type_id = lt.id
        WHERE lb.employee_id = $1 AND lb.year = $2
        "#,
    )
    .bind(employee.employee_id)
    .bind(year)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(balances))
}

/// Get leave requests for the current employee (ESS version - avoids name conflict)
pub async fn ess_get_my_leave_requests(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<crate::models::LeaveRequest>>> {
    let requests = sqlx::query_as::<_, crate::models::LeaveRequest>(
        "SELECT * FROM leave_requests WHERE employee_id = $1 ORDER BY created_at DESC",
    )
    .bind(employee.employee_id)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(requests))
}

// --- Shift Swap ESS Endpoints (Phase 2) ---

use crate::models::shift_swap::{
    EssCreateShiftSwapRequest, RespondShiftSwapRequest, ShiftSwapRequest,
    ShiftSwapRequestWithNames,
};


/// GET /ess/shift-swaps — list swap requests where I am requester or target
pub async fn ess_get_my_shift_swaps(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<ShiftSwapRequestWithNames>>> {
    let swaps = state.shift_swap_service.get_employee_swaps(employee.employee_id, employee.workspace_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to list shift swaps".to_string())))?;

    Ok(Json(swaps))
}

/// POST /ess/shift-swaps — employee creates a swap request (status: pending_target)
pub async fn ess_create_shift_swap(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Json(req): Json<EssCreateShiftSwapRequest>,
) -> AppResult<(StatusCode, Json<ShiftSwapRequest>)> {
    let swap = state.shift_swap_service.create_ess_swap(
        employee.workspace_id,
        employee.employee_id,
        employee.user_id,
        req,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to create shift swap".to_string())))?;

    Ok((StatusCode::CREATED, Json(swap)))
}

/// PUT /ess/shift-swaps/:id/respond — target employee accepts or rejects
pub async fn ess_respond_to_swap(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Path(id): Path<Uuid>,
    Json(req): Json<RespondShiftSwapRequest>,
) -> AppResult<StatusCode> {
    state.shift_swap_service.respond_to_swap(employee.workspace_id, employee.employee_id, id, req.accept)
        .await
        .map_err(|s| AppError::from((s, "Failed to respond to shift swap".to_string())))?;

    Ok(StatusCode::OK)
}

/// DELETE /ess/shift-swaps/:id — requester cancels their swap request
pub async fn ess_cancel_swap(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    state.shift_swap_service.cancel_swap(employee.workspace_id, employee.employee_id, id)
        .await
        .map_err(|s| AppError::from((s, "Failed to cancel shift swap".to_string())))?;

    Ok(StatusCode::NO_CONTENT)
}
