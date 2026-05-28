use axum::{
    extract::{Multipart, Path, State},
    response::Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AttendanceDailySummary, BulkOtEntryError, BulkOtUploadResult};
use crate::AppState;

const MAX_BULK_UPLOAD_ROWS: usize = 10_000;
const MAX_BULK_UPLOAD_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EnterPaperOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: String,
    #[serde(default)]
    pub allow_exceed_system_detected: bool,
    pub paper_form_photo_url: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/{employee_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = EnterPaperOtRequest,
    responses(
        (status = 200, description = "Paper OT entered successfully", body = AttendanceDailySummary)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn enter_paper_ot(
    State(state): State<Arc<AppState>>,
    Path(__path_params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<EnterPaperOtRequest>,
) -> AppResult<Json<AttendanceDailySummary>> {
    let (_, employee_id) = __path_params;
    // 1. RBAC: OvertimePaperUpload permission required
    ctx.require_permission(Permission::OvertimePaperUpload)?;

    // 2. Validate employee exists in workspace
    let _employee = state
        .employees
        .repo
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::NotFound("Employee not found".to_string()))?;

    // 3. Validate minutes
    if req.claimed_minutes <= 0 {
        return Err(AppError::Validation(
            "Minutes must be greater than 0".to_string(),
        ));
    }

    // 4. Call service
    let result = state
        .attendance
        .summary_service
        .enter_paper_ot(
            crate::services::attendance_summary_service::EnterPaperOtArgs {
                workspace_id: ctx.workspace_id,
                employee_id,
                date: req.date,
                claimed_minutes: req.claimed_minutes,
                reason: req.reason,
                entered_by_user_id: ctx.user_id,
                photo_url: req.paper_form_photo_url,
                allow_exceed: req.allow_exceed_system_detected,
            },
        )
        .await?;

    Ok(Json(result))
}

#[allow(clippy::too_many_lines)]
#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/bulk",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Bulk OT upload result", body = BulkOtUploadResult)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[allow(clippy::too_many_lines)]
pub async fn bulk_upload_paper_ot(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<BulkOtUploadResult>> {
    // 1. RBAC: OvertimePaperUpload permission required
    ctx.require_permission(Permission::OvertimePaperUpload)?;

    // 2. Extract CSV file from multipart
    let mut csv_data: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("Failed to read file bytes: {e}")))?;

            // File size limit check
            if data.len() > MAX_BULK_UPLOAD_FILE_SIZE {
                return Err(AppError::Validation("File too large".to_string()));
            }

            csv_data = Some(data.to_vec());
            break;
        }
    }

    let csv_bytes = csv_data.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

    // 3. Parse CSV
    let csv_str = String::from_utf8(csv_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid UTF-8 in CSV: {e}")))?;

    let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());

    // Collect all rows first to validate count
    let mut rows: Vec<(usize, csv::StringRecord)> = Vec::new();
    for (idx, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| {
            AppError::Validation(format!("CSV parsing error at row {}: {e}", idx + 2))
        })?;
        rows.push((idx + 2, record)); // +2 because CSV line 1 is header, records start at line 2

        if rows.len() > MAX_BULK_UPLOAD_ROWS {
            return Err(AppError::Validation(format!(
                "Max {MAX_BULK_UPLOAD_ROWS} rows allowed"
            )));
        }
    }

    let total_rows = rows.len();
    let mut errors: Vec<BulkOtEntryError> = Vec::new();
    let mut success_count = 0;

    // Track duplicates within this upload
    let mut seen_keys: std::collections::HashSet<(String, NaiveDate)> =
        std::collections::HashSet::new();

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
        let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
            errors.push(BulkOtEntryError {
                line_number,
                employee_id: employee_id_str,
                error_message: format!("Invalid date format: {date_str}. Expected YYYY-MM-DD"),
            });
            continue;
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
        let Ok(minutes) = minutes_str.parse::<i32>() else {
            errors.push(BulkOtEntryError {
                line_number,
                employee_id: employee_id_str,
                error_message: format!("Invalid minutes value: {minutes_str}"),
            });
            continue;
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
        let Some(employee) = (match state
            .employees
            .repo
            .get_employee_by_id_number(&employee_id_str, &ctx.auth_token)
            .await
        {
            Ok(Some(emp)) => Some(emp),
            Ok(None) => {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str.clone(),
                    error_message: "Employee not found in workspace".to_string(),
                });
                None
            }
            Err(e) => {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str.clone(),
                    error_message: format!("Database error: {e}"),
                });
                None
            }
        }) else {
            continue;
        };

        // Validate against system_detected_ot_minutes
        let existing_summary = state
            .attendance
            .attendance_daily_summary_repo
            .get_summary(&ctx.auth_token, employee.id, date)
            .await;

        if let Ok(Some(summary)) = existing_summary {
            let system_detected = summary.system_detected_ot_minutes;
            if minutes > system_detected {
                errors.push(BulkOtEntryError {
                    line_number,
                    employee_id: employee_id_str,
                    error_message: format!(
                        "Claimed {minutes} minutes exceeds system detected {system_detected} minutes"
                    ),
                });
                continue;
            }
        }

        // Call enter_paper_ot service
        match state
            .attendance
            .summary_service
            .enter_paper_ot(
                crate::services::attendance_summary_service::EnterPaperOtArgs {
                    workspace_id: ctx.workspace_id,
                    employee_id: employee.id,
                    date,
                    claimed_minutes: minutes,
                    reason,
                    entered_by_user_id: ctx.user_id,
                    photo_url: None,     // No photo URL for bulk upload
                    allow_exceed: false, // Don't allow exceed for bulk upload
                },
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
                    error_message: e.to_string(),
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
