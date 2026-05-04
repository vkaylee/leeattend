use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::AppState;

pub async fn get_attendance_report(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let start_date_str = params
        .get("start_date")
        .ok_or_else(|| AppError::Validation("Missing start_date".to_string()))?;
    let end_date_str = params
        .get("end_date")
        .ok_or_else(|| AppError::Validation("Missing end_date".to_string()))?;
    let report_type = params.get("type").map(|s| s.as_str()).unwrap_or("summary");

    let page = params.get("page").and_then(|s| s.parse::<i64>().ok());
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok());
    let sort = params.get("sort").and_then(|s| {
        serde_json::from_value::<crate::models::AttendanceReportSort>(serde_json::Value::String(
            s.clone(),
        ))
        .ok()
    });
    let employee_ids = params.get("employee_ids").map(|s| {
        s.split(',')
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect::<Vec<Uuid>>()
    });
    let department_id = params
        .get("department_id")
        .and_then(|s| Uuid::parse_str(s).ok());

    let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
        .map_err(|_e| AppError::Validation("Invalid start_date format".to_string()))?;
    let end_date = NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d")
        .map_err(|_e| AppError::Validation("Invalid end_date format".to_string()))?;

    if report_type == "daily" {
        let report = state
            .reports
            .service
            .get_daily_attendance_report(crate::services::report_service::AttendanceReportArgs {
                workspace_id,
                start_date,
                end_date,
                page,
                limit,
                employee_ids,
                department_id,
                sort: None,
            })
            .await?;

        Ok(Json(
            serde_json::to_value(report).map_err(|e| AppError::from(e.to_string()))?,
        ))
    } else {
        let summary = state
            .reports
            .service
            .get_workspace_attendance_summary(
                crate::services::report_service::AttendanceReportArgs {
                    workspace_id,
                    start_date,
                    end_date,
                    page,
                    limit,
                    employee_ids,
                    department_id,
                    sort,
                },
            )
            .await?;

        Ok(Json(
            serde_json::to_value(summary).map_err(|e| AppError::from(e.to_string()))?,
        ))
    }
}
