use axum::{
    extract::{Multipart, Path, State},
    response::Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::employee::Employee;
use crate::models::{AttendanceDailySummary, BulkOtEntryError, BulkOtUploadResult};
use crate::utils::i18n::I18nKey;
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
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, employee_id = %__path_params.1))]
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
    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

    // 3. Validate minutes
    if req.claimed_minutes <= 0 {
        return Err(AppError::validation(I18nKey::MinutesGreaterThanZero));
    }

    let active_employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    // 4. Call service
    let result = state
        .attendance()
        .summary_service()
        .enter_paper_ot(
            &ctx.auth_token,
            crate::services::attendance_summary_service::EnterPaperOtArgs {
                employment_id: active_employment_id,
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

/// Parsed row after basic in-memory validation (no DB lookups yet).
struct ParsedBulkOtRow {
    line_number: usize,
    employee_id_str: String,
    date: NaiveDate,
    minutes: i32,
    reason: String,
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
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn bulk_upload_paper_ot(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<BulkOtUploadResult>> {
    // 1. RBAC: OvertimePaperUpload permission required
    ctx.require_permission(Permission::OvertimePaperUpload)?;

    // 2. Extract CSV file from multipart
    let csv_bytes = extract_csv_from_multipart(&mut multipart).await?;

    // 3. Parse CSV and perform basic in-memory validation (no DB calls)
    let (parsed_rows, mut errors) = parse_and_validate_csv_rows(&csv_bytes)?;
    if parsed_rows.is_empty() {
        return Ok(Json(BulkOtUploadResult {
            total_rows: errors.len(),
            success_count: 0,
            failed_count: errors.len(),
            errors,
        }));
    }

    let total_rows = parsed_rows.len() + errors.len();

    // 4. Phase 1: Batch-fetch all employees in ONE query
    let unique_codes: Vec<String> = {
        let mut codes: Vec<String> = parsed_rows
            .iter()
            .map(|r| r.employee_id_str.clone())
            .collect();
        codes.sort();
        codes.dedup();
        codes
    };

    let employees_list = state
        .employees()
        .repo()
        .get_employees_by_id_numbers(&unique_codes, &ctx.auth_token)
        .await?;

    let employees_by_code: HashMap<String, Employee> = employees_list
        .into_iter()
        .map(|emp| (emp.employee_id_number().to_string(), emp))
        .collect();

    // 5. Phase 2: Resolve employment IDs and collect (employment_id, date) pairs for batch summary fetch
    let mut row_to_employment: HashMap<usize, Uuid> = HashMap::new();
    let mut summary_pairs: Vec<(Uuid, NaiveDate)> = Vec::new();
    let mut valid_parsed: Vec<&ParsedBulkOtRow> = Vec::new();

    let workspace_tz = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(ctx.workspace_id)
        .await?
        .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
        .unwrap_or(chrono_tz::UTC);
    let today = crate::utils::tz::utc_now_date_in_tz(&workspace_tz);

    for row in &parsed_rows {
        let Some(employee) = employees_by_code.get(&row.employee_id_str) else {
            errors.push(BulkOtEntryError {
                line_number: row.line_number,
                employment_id: row.employee_id_str.clone(),
                error_message: "Employee not found in workspace".to_string(),
            });
            continue;
        };

        let Some(active_employment_id) = employee.active_employment_id() else {
            errors.push(BulkOtEntryError {
                line_number: row.line_number,
                employment_id: row.employee_id_str.clone(),
                error_message: "No active employment found for employee".to_string(),
            });
            continue;
        };

        // Check future date
        if row.date > today {
            errors.push(BulkOtEntryError {
                line_number: row.line_number,
                employment_id: row.employee_id_str.clone(),
                error_message: "Date cannot be in the future".to_string(),
            });
            continue;
        }

        row_to_employment.insert(row.line_number, active_employment_id);
        summary_pairs.push((active_employment_id, row.date));
        valid_parsed.push(row);
    }

    if valid_parsed.is_empty() {
        return Ok(Json(BulkOtUploadResult {
            total_rows,
            success_count: 0,
            failed_count: errors.len(),
            errors,
        }));
    }

    // 6. Phase 3: Batch-fetch ALL summaries in ONE query
    let (min_date, max_date) = summary_pairs.iter().fold(
        (summary_pairs[0].1, summary_pairs[0].1),
        |(min, max), (_, d)| (std::cmp::min(min, *d), std::cmp::max(max, *d)),
    );
    let all_employment_ids: Vec<Uuid> = {
        let mut ids: Vec<Uuid> = valid_parsed
            .iter()
            .filter_map(|r| row_to_employment.get(&r.line_number).copied())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    };

    let summaries_map = state
        .attendance()
        .attendance_daily_summary_repo()
        .get_summaries_by_employments_batch(
            &ctx.auth_token,
            &all_employment_ids,
            min_date,
            max_date,
        )
        .await?;

    // 7. Phase 4: Process each row with in-memory data, persist via enter_paper_ot
    let mut success_count = 0;

    for row in valid_parsed {
        let active_employment_id = match row_to_employment.get(&row.line_number) {
            Some(id) => *id,
            None => continue, // Should not happen — already filtered above
        };

        // Validate against system_detected_ot_minutes (in-memory)
        if let Some(summary) = summaries_map.get(&(active_employment_id, row.date)) {
            if row.minutes > summary.system_detected_ot_minutes {
                errors.push(BulkOtEntryError {
                    line_number: row.line_number,
                    employment_id: row.employee_id_str.clone(),
                    error_message: format!(
                        "Claimed {} minutes exceeds system detected {} minutes",
                        row.minutes, summary.system_detected_ot_minutes
                    ),
                });
                continue;
            }
        }

        // Persist
        match state
            .attendance()
            .summary_service()
            .enter_paper_ot(
                &ctx.auth_token,
                crate::services::attendance_summary_service::EnterPaperOtArgs {
                    employment_id: active_employment_id,
                    date: row.date,
                    claimed_minutes: row.minutes,
                    reason: row.reason.clone(),
                    entered_by_user_id: ctx.user_id,
                    photo_url: None,
                    allow_exceed: false,
                },
            )
            .await
        {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                errors.push(BulkOtEntryError {
                    line_number: row.line_number,
                    employment_id: row.employee_id_str.clone(),
                    error_message: e.to_string(),
                });
            }
        }
    }

    // 8. Return result
    Ok(Json(BulkOtUploadResult {
        total_rows,
        success_count,
        failed_count: errors.len(),
        errors,
    }))
}

/// Extract CSV bytes from multipart form data.
///
/// The size limit is enforced while streaming, so an oversized upload is
/// rejected as soon as it crosses the threshold rather than after the whole
/// body has been buffered.
async fn extract_csv_from_multipart(multipart: &mut Multipart) -> AppResult<Vec<u8>> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::utils::upload::multipart_error_to_app_error(&e, I18nKey::MultipartError)
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let data =
                crate::utils::upload::read_field_limited(field, MAX_BULK_UPLOAD_FILE_SIZE).await?;
            return Ok(data.to_vec());
        }
    }
    Err(AppError::validation(I18nKey::NoFileUploaded))
}

/// Parse CSV and perform basic in-memory validation (no DB calls).
/// Returns parsed rows and any validation errors.
fn parse_and_validate_csv_rows(
    csv_bytes: &[u8],
) -> AppResult<(Vec<ParsedBulkOtRow>, Vec<BulkOtEntryError>)> {
    let csv_str = String::from_utf8(csv_bytes.to_vec())
        .map_err(|_e| AppError::validation(I18nKey::InvalidUtf8Csv))?;

    let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());
    let mut parsed_rows = Vec::new();
    let mut errors = Vec::new();
    let mut seen_keys: std::collections::HashSet<(String, NaiveDate)> =
        std::collections::HashSet::new();

    for (idx, result) in rdr.records().enumerate() {
        let line_number = idx + 2; // +2: CSV header is line 1, records start at line 2

        if parsed_rows.len() + errors.len() >= MAX_BULK_UPLOAD_ROWS {
            return Err(AppError::validation(I18nKey::MaxRowsExceeded));
        }

        let record = result.map_err(|_e| AppError::validation(I18nKey::CsvParsingError))?;

        let employee_id_str = record.get(0).unwrap_or("").trim().to_string();
        let date_str = record.get(1).unwrap_or("").trim();
        let minutes_str = record.get(2).unwrap_or("").trim();
        let reason = record.get(3).unwrap_or("").trim().to_string();

        // Validate required fields
        if employee_id_str.is_empty() {
            errors.push(BulkOtEntryError {
                line_number,
                employment_id: employee_id_str,
                error_message: "Employee ID is required".to_string(),
            });
            continue;
        }

        let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
            errors.push(BulkOtEntryError {
                line_number,
                employment_id: employee_id_str,
                error_message: format!("Invalid date format: {date_str}. Expected YYYY-MM-DD"),
            });
            continue;
        };

        let Ok(minutes) = minutes_str.parse::<i32>() else {
            errors.push(BulkOtEntryError {
                line_number,
                employment_id: employee_id_str,
                error_message: format!("Invalid minutes value: {minutes_str}"),
            });
            continue;
        };

        if minutes <= 0 {
            errors.push(BulkOtEntryError {
                line_number,
                employment_id: employee_id_str,
                error_message: "Minutes must be greater than 0".to_string(),
            });
            continue;
        }

        if reason.is_empty() {
            errors.push(BulkOtEntryError {
                line_number,
                employment_id: employee_id_str,
                error_message: "Reason is required".to_string(),
            });
            continue;
        }

        // Check for duplicate within upload
        let key = (employee_id_str.clone(), date);
        if seen_keys.contains(&key) {
            errors.push(BulkOtEntryError {
                line_number,
                employment_id: employee_id_str,
                error_message: "Duplicate entry in this upload".to_string(),
            });
            continue;
        }
        seen_keys.insert(key);

        parsed_rows.push(ParsedBulkOtRow {
            line_number,
            employee_id_str,
            date,
            minutes,
            reason,
        });
    }

    Ok((parsed_rows, errors))
}
