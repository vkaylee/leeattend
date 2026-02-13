use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::AppState;

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
