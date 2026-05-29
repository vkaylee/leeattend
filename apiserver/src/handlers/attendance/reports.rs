use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::NaiveDate;

use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct AttendanceReportQuery {
    pub start_date: String,
    pub end_date: String,
    pub r#type: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub employee_ids: Option<String>,
    pub department_id: Option<uuid::Uuid>,
}


#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/reports/attendance",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        AttendanceReportQuery
    ),
    responses(
        (status = 200, description = "Attendance report data")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_attendance_report(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<AttendanceReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let workspace_id = ctx.workspace_id;

    let start_date_str = &params.start_date;
    let end_date_str = &params.end_date;
    let report_type = params.r#type.as_deref().unwrap_or("summary");

    let page = params.page;
    let limit = params.limit;
    let sort = params.sort.as_ref().and_then(|s| serde_json::from_value::<crate::models::AttendanceReportSort>(serde_json::Value::String(s.clone())).ok());
    let employee_ids = params.employee_ids.as_ref().map(|s| {
        s.split(',')
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect::<Vec<Uuid>>()
    });
    let department_id = params.department_id;

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
