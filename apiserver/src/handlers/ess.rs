#![allow(unused_variables)]
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
    CreateEmergencyContactRequest, EmployeeDocument, EmployeeEmergencyContact, EmployeeProfile,
    MyProfileResponse, TimeEntry, UpdateEmployeeProfileRequest,
};
use crate::AppState;

// --- Profile Endpoints ---

/// Get the current employee's complete profile
pub async fn get_my_profile(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<MyProfileResponse>> {
    // Get employee record
    let emp = state
        .employees
        .repo
        .get_employee_by_id(employee.employee_id, &crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Employee not found".to_string()))?;

    // Get profile (may not exist yet)
    let profile = state
        .employees
        .repo
        .get_employee_profile(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id)
        .await?;

    // Get emergency contacts
    let contacts = state
        .employees
        .repo
        .get_emergency_contacts(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id)
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
    // Upsert profile
    let profile = state
        .employees
        .repo
        .upsert_employee_profile(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id, &req)
        .await?;

    Ok(Json(profile))
}

// --- Emergency Contacts Endpoints ---

/// Get all emergency contacts for the current employee
pub async fn get_my_emergency_contacts(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<EmployeeEmergencyContact>>> {
    let contacts = state
        .employees
        .repo
        .get_emergency_contacts(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id)
        .await?;

    Ok(Json(contacts))
}

/// Add a new emergency contact
pub async fn add_emergency_contact(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Json(req): Json<CreateEmergencyContactRequest>,
) -> AppResult<impl IntoResponse> {
    let contact = state
        .employees
        .repo
        .upsert_emergency_contact(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id, &req)
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
    let affected = state
        .employees
        .repo
        .delete_emergency_contact(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id, contact_id)
        .await?;

    if affected == 0 {
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
    let docs = state
        .employees
        .repo
        .get_employee_documents(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id)
        .await?;

    Ok(Json(docs))
}

// --- Payslips Endpoints ---

/// Get all payslips for the current employee
pub async fn get_my_payslips(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<crate::models::Payslip>>> {
    let payslips = state
        .payroll
        .repo
        .get_payslips_by_employee(employee.workspace_id, employee.employee_id)
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

    let from = start
        .and_hms_opt(0, 0, 0)
        .map(|t| t.and_utc())
        .ok_or_else(|| AppError::Internal("Invalid start date".to_string()))?;
    let to = end
        .and_hms_opt(23, 59, 59)
        .map(|t| t.and_utc())
        .ok_or_else(|| AppError::Internal("Invalid end date".to_string()))?;

    let entries = state
        .attendance
        .time_entry_repo
        .get_employee_time_entries(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id, from, to)
        .await?;

    Ok(Json(entries))
}

// --- Time Clock Endpoints ---

#[derive(serde::Deserialize)]
pub struct EssPunchRequest {
    pub note: Option<String>,
}

pub async fn ess_punch(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Json(req): Json<EssPunchRequest>,
) -> AppResult<(StatusCode, Json<TimeEntry>)> {
    let entry = state
        .attendance
        .time_tracking_service
        .punch(
            state.attendance.log_service.as_ref(),
            employee.workspace_id,
            employee.employee_id,
            req.note,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(entry)))
}

#[derive(serde::Deserialize)]
pub struct EssTimeEntriesQuery {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_my_time_entries(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    axum::extract::Query(query): axum::extract::Query<EssTimeEntriesQuery>,
) -> AppResult<Json<Vec<TimeEntry>>> {
    let start = query
        .start_date
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let end = query.end_date.unwrap_or_else(chrono::Utc::now);

    let entries = state
        .attendance
        .time_tracking_service
        .get_employee_history(employee.workspace_id, employee.employee_id, start, end)
        .await?;

    Ok(Json(entries))
}

// --- Leave Endpoints ---

/// ESS Leave Balance Response
#[derive(Debug, serde::Serialize)]
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

    let balances = state
        .leaves
        .balance_repo
        .get_employee_balances(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id, year)
        .await?;

    let ess_balances = balances
        .into_iter()
        .map(|b| EssLeaveBalance {
            leave_type: b.leave_type_code,
            entitlement: b.entitlement,
            accrued_amount: b.accrued_amount,
            carried_over: b.carried_over,
            used: i64::from(b.used),
            remaining: b.remaining,
        })
        .collect();

    Ok(Json(ess_balances))
}

/// Get leave requests for the current employee (ESS version - avoids name conflict)
pub async fn ess_get_my_leave_requests(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<crate::models::LeaveRequest>>> {
    let requests = state
        .leaves
        .request_repo
        .get_employee_leave_requests(&crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id), employee.employee_id)
        .await?;

    Ok(Json(requests))
}

// --- Shift Swap ESS Endpoints (Phase 2) ---

use crate::models::shift_swap::{
    EssCreateShiftSwapRequest, RespondShiftSwapRequest, ShiftSwapRequest, ShiftSwapRequestWithNames,
};

/// GET /ess/shift-swaps — list swap requests where I am requester or target
pub async fn ess_get_my_shift_swaps(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
) -> AppResult<Json<Vec<ShiftSwapRequestWithNames>>> {
    let swaps = state
        .shifts
        .swap_service
        .get_employee_swaps(employee.employee_id, employee.workspace_id)
        .await?;

    Ok(Json(swaps))
}

/// POST /ess/shift-swaps — employee creates a swap request (status: pending_target)
pub async fn ess_create_shift_swap(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Json(req): Json<EssCreateShiftSwapRequest>,
) -> AppResult<(StatusCode, Json<ShiftSwapRequest>)> {
    let swap = state
        .shifts
        .swap_service
        .create_ess_swap(
            employee.workspace_id,
            employee.employee_id,
            employee.user_id,
            req,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(swap)))
}

/// PUT /ess/shift-swaps/:id/respond — target employee accepts or rejects
pub async fn ess_respond_to_swap(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Path(id): Path<Uuid>,
    Json(req): Json<RespondShiftSwapRequest>,
) -> AppResult<StatusCode> {
    state
        .shifts
        .swap_service
        .respond_to_swap(employee.workspace_id, employee.employee_id, id, req.accept)
        .await?;

    Ok(StatusCode::OK)
}

/// DELETE /ess/shift-swaps/:id — requester cancels their swap request
pub async fn ess_cancel_swap(
    State(state): State<Arc<AppState>>,
    employee: EmployeeContext,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    state
        .shifts
        .swap_service
        .cancel_swap(employee.workspace_id, employee.employee_id, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
