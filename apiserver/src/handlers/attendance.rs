use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{
    AdjustmentStatus, ApproveAttendanceAdjustmentRequest, BulkOtEntryError, BulkOtUploadResult,
    CreateAttendanceAdjustmentRequest, MemberRole, PunchInRequest, PunchOutRequest,
    UpdateTimeEntryRequest,
};
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::{AppState, UserId};

// --- Time Tracking Handlers ---

pub async fn punch_in(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<PunchInRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match crate::services::time_tracking_service::TimeTrackingService::punch_in(
        &state.db,
        workspace_id,
        req,
    )
    .await
    {
        Ok(entry) => Ok((StatusCode::CREATED, Json(entry))),
        Err(status) => Err(status),
    }
}

pub async fn punch_out(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<PunchOutRequest>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::punch_out(
        &state.db,
        workspace_id,
        employee_id,
        req,
    )
    .await
    {
        Ok(entry) => (StatusCode::OK, Json(entry)).into_response(),
        Err(status) => status.into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub async fn get_time_history(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::get_employee_history(
        &state.db,
        workspace_id,
        employee_id,
        query.start,
        query.end,
    )
    .await
    {
        Ok(entries) => Json(entries).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn update_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<UpdateTimeEntryRequest>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::update_time_entry(
        &state.db,
        workspace_id,
        id,
        req,
    )
    .await
    {
        Ok(entry) => Json(entry).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn delete_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::delete_time_entry(
        &state.db,
        workspace_id,
        id,
    )
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(status) => status.into_response(),
    }
}

// --- Attendance Import Handlers ---

pub async fn upload_attendance_excel(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    // RBAC: Only Admin/Owner
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::FORBIDDEN)?;

    if role != MemberRole::Admin && role != MemberRole::Owner {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or(StatusCode::BAD_REQUEST)?;

    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "adjustment".to_string()
    ).await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_attendance_logs(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    // RBAC: Only Admin/Owner
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::FORBIDDEN)?;

    if role != MemberRole::Admin && role != MemberRole::Owner {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or(StatusCode::BAD_REQUEST)?;

    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "log".to_string()
    ).await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_employee_excel(

    State(state): State<Arc<AppState>>,

    Path(ws_id): Path<Uuid>,

    MemberContext { user_id, .. }: MemberContext,

    mut multipart: Multipart,

) -> Result<impl IntoResponse, StatusCode> {

    // RBAC: Only Admin/Owner

    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {

        pool: state.db.clone(),

    }

    .find_primary_role(ws_id, user_id)

    .await

    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?

    .ok_or(StatusCode::FORBIDDEN)?;



    if role != MemberRole::Admin && role != MemberRole::Owner {

        return Err(StatusCode::FORBIDDEN);

    }



    let mut file_path = None;

    while let Some(field) = multipart

        .next_field()

        .await

        .map_err(|_| StatusCode::BAD_REQUEST)?

    {

        let name = field.name().unwrap_or("").to_string();

        if name == "file" {

            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();

            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;



            let dir = "storage/imports";

            tokio::fs::create_dir_all(dir)

                .await

                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);

            tokio::fs::write(&path, data)

                .await

                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            file_path = Some(path);

        }

    }



    let path = file_path.ok_or(StatusCode::BAD_REQUEST)?;
    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "employee".to_string()
    ).await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

#[derive(Deserialize)]
pub struct ImportJobQuery {
    pub target_type: Option<String>,
}

pub async fn get_latest_import_job(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<ImportJobQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    match crate::repositories::import_job_repository::get_latest_job(
        &state.db,
        workspace_id,
        query.target_type,
    )
    .await
    {
        Ok(job) => Ok(Json(job)),
        Err(e) => {
            error!("GET_LATEST_JOB_ERROR: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct AttendanceLogQuery {
    pub employee_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_attendance_logs(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = query.limit.unwrap_or(20);
    let page = query.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    let logs = crate::repositories::time_entry_repository::get_paginated_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        error!("GET_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = crate::repositories::time_entry_repository::count_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
    )
    .await
    .map_err(|e| {
        error!("COUNT_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

pub async fn get_raw_attendance_logs(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = query.limit.unwrap_or(20);
    let page = query.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    let logs = crate::repositories::raw_attendance_log_repository::get_paginated_raw_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        error!("GET_RAW_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = crate::repositories::raw_attendance_log_repository::count_raw_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
    )
    .await
    .map_err(|e| {
        error!("COUNT_RAW_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

pub async fn get_import_job_status(
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let job = crate::repositories::import_job_repository::get_job_by_id(&state.db, job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(job))
}

#[derive(Deserialize)]
pub struct ReprocessQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

pub async fn force_reprocess_attendance(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Query(query): Query<ReprocessQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }

    let workspace = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .get_workspace_by_id(ctx.workspace_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let tz: chrono_tz::Tz = workspace
        .timezone
        .parse()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut curr_date = query.start_date;
    while curr_date <= query.end_date {
        crate::services::pairing_service::PairingService::reprocess_daily_attendance(
            &state.db,
            employee_id,
            curr_date,
            tz,
            &workspace.pairing_algorithm,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(next) = curr_date.succ_opt() {
            curr_date = next;
        } else {
            break;
        }
    }

    Ok(StatusCode::OK)
}

pub async fn submit_attendance_adjustment(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<CreateAttendanceAdjustmentRequest>,
) -> impl IntoResponse {
    println!(
        "SUBMIT_ADJUSTMENT_HANDLER_REACHED: ws_id={}, user_id={}",
        workspace_id, user_id
    );
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::submit_request(
        &state.db,
        workspace_id,
        user_id,
        req,
    )
    .await
    {
        Ok(adjustment) => Ok((StatusCode::CREATED, Json(adjustment))),
        Err(e) => Err(e),
    }
}

pub async fn approve_attendance_adjustment(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveAttendanceAdjustmentRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::approve_request(
        &state.db,
        ctx.workspace_id,
        user_id,
        id,
        req.status,
    )
    .await
    {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(e) => Err(e),
    }
}

pub async fn get_my_attendance_adjustments(
    Path(id): Path<Uuid>,
    UserId(user_id): UserId,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("GET_MY_ADJUSTMENTS_REACHED: id={}, user_id={}", id, user_id);
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_my_requests(
        &state.db, id,
    )
    .await {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}

#[derive(Deserialize)]
pub struct AdjustmentStatusQuery {
    pub status: Option<AdjustmentStatus>,
}

pub async fn get_workspace_attendance_adjustments(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AdjustmentStatusQuery>,
) -> impl IntoResponse {
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_workspace_requests(&state.db, workspace_id, query.status).await {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_attendance_report(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    MemberContext { .. }: MemberContext,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let start_date_str = params.get("start_date").ok_or(StatusCode::BAD_REQUEST)?;
    let end_date_str = params.get("end_date").ok_or(StatusCode::BAD_REQUEST)?;
    let report_type = params.get("type").map(|s| s.as_str()).unwrap_or("summary");

    let page = params.get("page").and_then(|s| s.parse::<i64>().ok());
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok());
    let sort = params.get("sort").and_then(|s| {
        serde_json::from_value::<crate::models::AttendanceReportSort>(serde_json::Value::String(
            s.to_string(),
        ))
        .ok()
    });
    let employee_ids = params.get("employee_ids").map(|s| {
        s.split(',')
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect::<Vec<Uuid>>()
    });

    let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let end_date =
        NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d").map_err(|_| StatusCode::BAD_REQUEST)?;

    if report_type == "daily" {
        match crate::services::ReportService::get_daily_attendance_report(
            &state.db,
            workspace_id,
            start_date,
            end_date,
            page,
            limit,
            employee_ids,
        )
        .await
        {
            Ok(report) => Ok(Json(report).into_response()),
            Err(e) => Err(e),
        }
    } else {
        match crate::services::ReportService::get_workspace_attendance_summary(
            &state.db,
            workspace_id,
            start_date,
            end_date,
            sort,
            page,
            limit,
            employee_ids,
        )
        .await
        {
            Ok(summary) => Ok(Json(summary).into_response()),
            Err(e) => Err(e),
        }
    }
}


// --- Attendance Summary & OT Handlers ---

#[derive(Deserialize)]
pub struct ClaimOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: Option<String>,
}

pub async fn claim_ot(
    State(state): State<Arc<AppState>>,
    Path((ws_id, employee_id)): Path<(Uuid, Uuid)>,
    MemberContext { user_id, .. }: MemberContext,
    Json(req): Json<ClaimOtRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. Verify ownership (Employee can only claim for themselves) or Admin
    let employee = crate::repositories::employee_repository::get_employee_by_id(&state.db, employee_id, ws_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(uid) = employee.user_id {
        if uid != user_id {
             // If not the user, check if Admin/Owner/Manager?
             let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository { pool: state.db.clone() }
                .find_primary_role(ws_id, user_id).await.unwrap_or(None);
             
             if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
                 return Err(StatusCode::FORBIDDEN);
             }
        }
    } else {
         let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository { pool: state.db.clone() }
            .find_primary_role(ws_id, user_id).await.unwrap_or(None);
         
         if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
             return Err(StatusCode::FORBIDDEN);
         }
    }

    crate::services::attendance_summary_service::AttendanceSummaryService::claim_ot(
        &state.db,
        employee_id,
        req.date,
        req.claimed_minutes,
        req.reason,
    )
    .await
    .map_err(|e| {
        error!("Claim OT Error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct ApproveOtRequest {
    pub date: NaiveDate,
    pub approved_minutes: i32,
    pub manager_note: Option<String>,
    pub status: String, // "approved" or "rejected"
}

pub async fn approve_ot(
    State(state): State<Arc<AppState>>,
    Path((ws_id, employee_id)): Path<(Uuid, Uuid)>,
    MemberContext { user_id, .. }: MemberContext,
    Json(req): Json<ApproveOtRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // RBAC: Only Admin/Owner/Manager
     let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository { pool: state.db.clone() }
        .find_primary_role(ws_id, user_id).await.unwrap_or(None);
     
     if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
         return Err(StatusCode::FORBIDDEN);
     }

    let status_enum = match req.status.as_str() {
        "approved" => crate::models::OtStatus::Approved,
        "rejected" => crate::models::OtStatus::Rejected,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    crate::services::attendance_summary_service::AttendanceSummaryService::approve_ot(
        &state.db,
        employee_id,
        req.date,
        req.approved_minutes,
        req.manager_note,
        status_enum,
    )
    .await
    .map_err(|e| {
        error!("Approve OT Error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub employee_id: Option<Uuid>,
}

pub async fn get_attendance_summaries(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    Query(query): Query<SummaryQuery>,
) -> Result<impl IntoResponse, StatusCode> {

    // RBAC and ID Resolution
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository { pool: state.db.clone() }
        .find_primary_role(ws_id, user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::FORBIDDEN)?;

    let is_admin = matches!(role, MemberRole::Admin | MemberRole::Owner);

    let target_employee_id = if is_admin {
        query.employee_id
    } else {
        // Find my employee ID
        let emp_record = sqlx::query!("SELECT id FROM employees WHERE workspace_id = $1 AND user_id = $2", ws_id, user_id)
            .fetch_optional(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let my_id = emp_record.ok_or(StatusCode::FORBIDDEN)?.id;

        if let Some(req_id) = query.employee_id {
            if req_id != my_id {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        Some(my_id)
    };

    let summaries = if let Some(emp_id) = target_employee_id {
        sqlx::query_as!(
            crate::models::AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: crate::models::OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE workspace_id = $1 AND employee_id = $2 AND date >= $3 AND date <= $4 ORDER BY date ASC",
            ws_id, emp_id, query.start_date, query.end_date
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as!(
            crate::models::AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: crate::models::OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE workspace_id = $1 AND date >= $2 AND date <= $3 ORDER BY date ASC, employee_id ASC",
            ws_id, query.start_date, query.end_date
        )
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| {
        error!("Get Summaries Error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(summaries))
}

// --- Paper Form OT Entry ---

#[derive(Deserialize)]
pub struct EnterPaperOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: String,
    #[serde(default)]
    pub allow_exceed_system_detected: bool,
    pub paper_form_photo_url: Option<String>,
}

pub async fn enter_paper_ot(
    State(state): State<Arc<AppState>>,
    Path((ws_id, employee_id)): Path<(Uuid, Uuid)>,
    MemberContext { user_id, .. }: MemberContext,
    Json(req): Json<EnterPaperOtRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. RBAC: Admin/Owner only
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository { pool: state.db.clone() }
        .find_primary_role(ws_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    if !matches!(role, MemberRole::Admin | MemberRole::Owner) {
        return Err(StatusCode::FORBIDDEN);
    }

    // 2. Validate employee exists in workspace
    let _employee = crate::repositories::employee_repository::get_employee_by_id(&state.db, employee_id, ws_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 3. Validate minutes
    if req.claimed_minutes <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 4. Call service
    let result = crate::services::attendance_summary_service::AttendanceSummaryService::enter_paper_ot(
        &state.db,
        employee_id,
        req.date,
        req.claimed_minutes,
        req.reason,
        user_id,
        req.paper_form_photo_url,
        req.allow_exceed_system_detected,
    )
    .await
    .map_err(|e| {
        error!("Enter Paper OT Error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(Json(result))
}

// --- Bulk CSV Upload for Paper OT ---

const MAX_BULK_UPLOAD_ROWS: usize = 10_000;
const MAX_BULK_UPLOAD_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub async fn bulk_upload_paper_ot(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. RBAC: Admin/Owner only
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::FORBIDDEN)?;

    if !matches!(role, MemberRole::Admin | MemberRole::Owner) {
        return Err(StatusCode::FORBIDDEN);
    }

    // 2. Extract CSV file from multipart
    let mut csv_data: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            // File size limit check
            if data.len() > MAX_BULK_UPLOAD_FILE_SIZE {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }

            csv_data = Some(data.to_vec());
            break;
        }
    }

    let csv_bytes = csv_data.ok_or(StatusCode::BAD_REQUEST)?;

    // 3. Parse CSV
    let csv_str = String::from_utf8(csv_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());

    // Collect all rows first to validate count
    let mut rows: Vec<(usize, csv::StringRecord)> = Vec::new();
    for (idx, result) in rdr.records().enumerate() {
        let record = result.map_err(|_| StatusCode::BAD_REQUEST)?;
        rows.push((idx + 2, record)); // +2 because CSV line 1 is header, records start at line 2

        if rows.len() > MAX_BULK_UPLOAD_ROWS {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
    }

    let total_rows = rows.len();
    let mut errors: Vec<BulkOtEntryError> = Vec::new();
    let mut success_count = 0;

    // Track duplicates within this upload
    let mut seen_keys: std::collections::HashSet<(String, NaiveDate)> = std::collections::HashSet::new();

    // 4. Process each row
    for (line_number, record) in rows {
        // Parse fields
        let employee_id_str = record.get(0).unwrap_or("").trim().to_string();
        let date_str = record.get(1).unwrap_or("").trim();
        let minutes_str = record.get(2).unwrap_or("").trim();
        let reason = record.get(3).unwrap_or("").trim().to_string();

        // Validation: Check for empty fields
        if employee_id_str.is_empty() {
            errors.push(BulkOtEntryError {
                line_number,
                employee_id: employee_id_str,
                error_message: "Employee ID is required".to_string(),
            });
            continue;
        }

        // Parse date
        let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str,
                    error_message: format!("Invalid date format: {}. Expected YYYY-MM-DD", date_str),
                });
                continue;
            }
        };

        // Check future date
        let today = chrono::Utc::now().date_naive();
        if date > today {
            errors.push(BulkOtEntryError {
                line_number,
                employee_id: employee_id_str.clone(),
                error_message: "Date cannot be in the future".to_string(),
            });
            continue;
        }

        // Parse minutes
        let minutes: i32 = match minutes_str.parse() {
            Ok(m) => m,
            Err(_) => {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str,
                    error_message: format!("Invalid minutes value: {}", minutes_str),
                });
                continue;
            }
        };

        // Validate minutes
        if minutes <= 0 {
            errors.push(BulkOtEntryError {
                line_number,
                employee_id: employee_id_str,
                error_message: "Minutes must be greater than 0".to_string(),
            });
            continue;
        }

        // Validate reason
        if reason.is_empty() {
            errors.push(BulkOtEntryError {
                line_number,
                employee_id: employee_id_str,
                error_message: "Reason is required".to_string(),
            });
            continue;
        }

        // Check for duplicate in this upload
        let key = (employee_id_str.clone(), date);
        if seen_keys.contains(&key) {
            errors.push(BulkOtEntryError {
                line_number,
                employee_id: employee_id_str,
                error_message: "Duplicate entry in this upload".to_string(),
            });
            continue;
        }
        seen_keys.insert(key);

        // Find employee by employee_id_number
        let employee = match crate::repositories::employee_repository::get_employee_by_id_number(
            &state.db,
            &employee_id_str,
            ws_id,
        )
        .await
        {
            Ok(Some(emp)) => emp,
            Ok(None) => {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str,
                    error_message: "Employee not found in workspace".to_string(),
                });
                continue;
            }
            Err(e) => {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str,
                    error_message: format!("Database error: {}", e),
                });
                continue;
            }
        };

        // Validate against system_detected_ot_minutes
        let existing_summary = sqlx::query!(
            "SELECT system_detected_ot_minutes FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
            employee.id,
            date
        )
        .fetch_optional(&state.db)
        .await;

        if let Ok(Some(summary)) = existing_summary {
            let system_detected = summary.system_detected_ot_minutes;
            if minutes > system_detected {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str,
                    error_message: format!(
                        "Claimed {} minutes exceeds system detected {} minutes",
                        minutes, system_detected
                    ),
                });
                continue;
            }
        }

        // Call enter_paper_ot service
        match crate::services::attendance_summary_service::AttendanceSummaryService::enter_paper_ot(
            &state.db,
            employee.id,
            date,
            minutes,
            reason,
            user_id,
            None, // No photo URL for bulk upload
            false, // Don't allow exceed for bulk upload
        )
        .await
        {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str,
                    error_message: e,
                });
            }
        }
    }

    // 5. Return result
    let result = BulkOtUploadResult {
        total_rows,
        success_count,
        failed_count: errors.len(),
        errors,
    };

    Ok(Json(result))
}

