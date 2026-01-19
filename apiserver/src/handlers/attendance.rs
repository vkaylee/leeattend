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
    AdjustmentStatus, ApproveAttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest,
    MemberRole, PunchInRequest, PunchOutRequest, UpdateTimeEntryRequest,
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

    let job_id = crate::services::attendance_import_service::AttendanceImportService::batch_import_from_file(
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

    let job_id = crate::services::attendance_import_service::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "log".to_string()
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
        )
        .await
        {
            Ok(summary) => Ok(Json(summary).into_response()),
            Err(e) => Err(e),
        }
    }
}
