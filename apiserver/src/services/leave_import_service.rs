//! Leave Import Service
//!
//! Parses Excel files containing leave requests, validates rows, submits valid ones,
//! and generates an error Excel file for failed rows.

use calamine::{Data, Reader, Xls, Xlsx};
use chrono::NaiveDate;
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};
use sqlx::PgPool;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{LeaveDurationType, SubmitLeaveRequest};
use crate::repositories::leave_balance_repository::SqlxLeaveBalanceRepository;
use crate::repositories::leave_request_repository::SqlxLeaveRequestRepository;
use crate::repositories::roster::SqlxRosterRepository;
use crate::repositories::{EmployeeRepository, SqlxEmployeeRepository};
use crate::services::import_messages::LeaveImportMessages;
use crate::services::leave_service::LeaveService;

/// Result of a leave import operation.
#[derive(Debug, Clone)]
pub struct LeaveImportResult {
    pub success_count: i32,
    pub error_count: i32,
    pub error_file_path: Option<String>,
}

/// A parsed row from the Excel file, pending validation.
#[derive(Debug, Clone)]
struct PendingLeaveRow {
    line_number: usize,
    employee_id_number: String,
    leave_type_code: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    reason: Option<String>,
    duration_type: LeaveDurationType,
}

/// An error row with the original data + error message for the error Excel export.
#[derive(Debug, Clone)]
pub(crate) struct ErrorRow {
    line_number: usize,
    employee_id_number: String,
    leave_type_code: String,
    start_date_raw: String,
    end_date_raw: String,
    reason: String,
    duration_type_raw: String,
    error_message: String,
}

pub struct LeaveImportService {
    pool: PgPool,
}

impl LeaveImportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Process an imported Excel file of leave requests.
    pub async fn process_import(
        &self,
        _import_job_id: Uuid,
        workspace_id: Uuid,
        user_id: Uuid,
        file_path: &str,
        auto_approve: bool,
    ) -> AppResult<LeaveImportResult> {
        // Read file
        let file_bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read file: {e}")))?;

        // Fetch workspace language for i18n messages
        let lang = sqlx::query_scalar::<_, String>(
            "SELECT default_language FROM workspaces WHERE id = $1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(Some("en".to_string()))
        .unwrap_or_else(|| "en".to_string());

        let msgs = LeaveImportMessages::for_lang(&lang);

        // Parse rows from Excel or CSV
        let (pending_rows, mut errors) = self.parse_file(&file_bytes, &msgs);

        if pending_rows.is_empty() && errors.is_empty() {
            return Err(AppError::Validation(msgs.file_empty.to_string()));
        }

        // Build leave service for submission
        let employee_repo: Arc<dyn EmployeeRepository> =
            Arc::new(SqlxEmployeeRepository::new(self.pool.clone()));
        let leave_balance_repo = Arc::new(SqlxLeaveBalanceRepository::new(self.pool.clone()));
        let leave_request_repo = Arc::new(SqlxLeaveRequestRepository::new(self.pool.clone()));
        let roster_repo = Arc::new(SqlxRosterRepository::new(self.pool.clone()));
        let leave_service = LeaveService::new(
            leave_balance_repo,
            leave_request_repo,
            employee_repo.clone(),
            roster_repo,
            self.pool.clone(),
        );

        // Lookup caches
        let employees = self.load_employee_map(workspace_id).await?;
        let leave_types = self.load_leave_type_map(workspace_id).await?;

        let mut success_count = 0i32;

        for row in &pending_rows {
            // Lookup employee
            let employee_id = match employees.get(&row.employee_id_number.to_uppercase()) {
                Some(id) => *id,
                None => {
                    errors.push(ErrorRow {
                        line_number: row.line_number,
                        employee_id_number: row.employee_id_number.clone(),
                        leave_type_code: row.leave_type_code.clone(),
                        start_date_raw: row.start_date.to_string(),
                        end_date_raw: row.end_date.to_string(),
                        reason: row.reason.clone().unwrap_or_default(),
                        duration_type_raw: format!("{:?}", row.duration_type),
                        error_message: msgs
                            .employee_not_found_tpl
                            .replace("{}", &row.employee_id_number),
                    });
                    continue;
                }
            };

            // Lookup leave type
            let leave_type_id = match leave_types.get(&row.leave_type_code.to_uppercase()) {
                Some(id) => *id,
                None => {
                    errors.push(ErrorRow {
                        line_number: row.line_number,
                        employee_id_number: row.employee_id_number.clone(),
                        leave_type_code: row.leave_type_code.clone(),
                        start_date_raw: row.start_date.to_string(),
                        end_date_raw: row.end_date.to_string(),
                        reason: row.reason.clone().unwrap_or_default(),
                        duration_type_raw: format!("{:?}", row.duration_type),
                        error_message: msgs
                            .leave_type_not_found_tpl
                            .replace("{}", &row.leave_type_code),
                    });
                    continue;
                }
            };

            // Submit leave request
            let submit = SubmitLeaveRequest {
                employee_id,
                leave_type_id,
                start_date: row.start_date,
                end_date: row.end_date,
                reason: row.reason.clone(),
                duration_type: row.duration_type,
                start_time: None,
                end_time: None,
            };

            match leave_service
                .submit_leave_request(workspace_id, user_id, submit, auto_approve)
                .await
            {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(ErrorRow {
                        line_number: row.line_number,
                        employee_id_number: row.employee_id_number.clone(),
                        leave_type_code: row.leave_type_code.clone(),
                        start_date_raw: row.start_date.to_string(),
                        end_date_raw: row.end_date.to_string(),
                        reason: row.reason.clone().unwrap_or_default(),
                        duration_type_raw: format!("{:?}", row.duration_type),
                        error_message: format!("{}: {e}", msgs.submission_error_prefix),
                    });
                }
            }
        }

        // Generate error Excel if there are errors
        let error_file_path = if !errors.is_empty() {
            let error_dir = format!(
                "{}/storage/imports/errors",
                std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            tokio::fs::create_dir_all(&error_dir).await.ok();

            let error_path = format!("{error_dir}/{_import_job_id}_errors.xlsx");
            match Self::generate_error_excel(&errors, &msgs) {
                Ok(bytes) => {
                    if let Err(e) = tokio::fs::write(&error_path, &bytes).await {
                        tracing::error!(error = %e, "Failed to write error Excel file");
                        None
                    } else {
                        Some(error_path)
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to generate error Excel");
                    None
                }
            }
        } else {
            None
        };

        Ok(LeaveImportResult {
            success_count,
            error_count: errors.len() as i32,
            error_file_path,
        })
    }

    /// Parse file bytes into pending rows and parse-time errors.
    fn parse_file(
        &self,
        file_bytes: &[u8],
        msgs: &LeaveImportMessages,
    ) -> (Vec<PendingLeaveRow>, Vec<ErrorRow>) {
        let mut pending = Vec::new();
        let mut errors = Vec::new();

        match crate::utils::file_parser::extract_file_rows(file_bytes) {
            Ok(rows) => {
                for (line_number, cols) in rows {
                    let emp_id = cols.first().unwrap_or(&String::new()).trim().to_string();
                    let leave_code = cols.get(1).unwrap_or(&String::new()).trim().to_string();
                    let start_str = cols.get(2).unwrap_or(&String::new()).trim().to_string();
                    let end_str = cols.get(3).unwrap_or(&String::new()).trim().to_string();
                    let reason = cols.get(4).unwrap_or(&String::new()).trim().to_string();
                    let duration_str = cols.get(5).unwrap_or(&String::new()).trim().to_string();

                    if emp_id.is_empty() && leave_code.is_empty() {
                        continue;
                    }

                    if emp_id.is_empty() {
                        errors.push(ErrorRow {
                            line_number,
                            employee_id_number: emp_id.clone(),
                            leave_type_code: leave_code.clone(),
                            start_date_raw: start_str.clone(),
                            end_date_raw: end_str.clone(),
                            reason: reason.clone(),
                            duration_type_raw: duration_str.clone(),
                            error_message: msgs.missing_employee_id.to_string(),
                        });
                        continue;
                    }

                    if leave_code.is_empty() {
                        errors.push(ErrorRow {
                            line_number,
                            employee_id_number: emp_id.clone(),
                            leave_type_code: leave_code.clone(),
                            start_date_raw: start_str.clone(),
                            end_date_raw: end_str.clone(),
                            reason: reason.clone(),
                            duration_type_raw: duration_str.clone(),
                            error_message: msgs.missing_leave_type.to_string(),
                        });
                        continue;
                    }

                    let start_date = match parse_date(&start_str) {
                        Some(d) => d,
                        None => {
                            errors.push(ErrorRow {
                                line_number,
                                employee_id_number: emp_id.clone(),
                                leave_type_code: leave_code.clone(),
                                start_date_raw: start_str.clone(),
                                end_date_raw: end_str.clone(),
                                reason: reason.clone(),
                                duration_type_raw: duration_str.clone(),
                                error_message: msgs.invalid_start_date.to_string(),
                            });
                            continue;
                        }
                    };

                    let end_date = match parse_date(&end_str) {
                        Some(d) => d,
                        None => {
                            errors.push(ErrorRow {
                                line_number,
                                employee_id_number: emp_id.clone(),
                                leave_type_code: leave_code.clone(),
                                start_date_raw: start_str.clone(),
                                end_date_raw: end_str.clone(),
                                reason: reason.clone(),
                                duration_type_raw: duration_str.clone(),
                                error_message: msgs.invalid_end_date.to_string(),
                            });
                            continue;
                        }
                    };

                    if end_date < start_date {
                        errors.push(ErrorRow {
                            line_number,
                            employee_id_number: emp_id.clone(),
                            leave_type_code: leave_code.clone(),
                            start_date_raw: start_str.clone(),
                            end_date_raw: end_str.clone(),
                            reason: reason.clone(),
                            duration_type_raw: duration_str.clone(),
                            error_message: msgs.end_before_start.to_string(),
                        });
                        continue;
                    }

                    let duration_type = parse_duration_type(&duration_str);

                    pending.push(PendingLeaveRow {
                        line_number,
                        employee_id_number: emp_id,
                        leave_type_code: leave_code,
                        start_date,
                        end_date,
                        reason: if reason.is_empty() {
                            None
                        } else {
                            Some(reason)
                        },
                        duration_type,
                    });
                }
            }
            Err(e) => {
                errors.push(ErrorRow {
                    line_number: 0,
                    employee_id_number: String::new(),
                    leave_type_code: String::new(),
                    start_date_raw: String::new(),
                    end_date_raw: String::new(),
                    reason: String::new(),
                    duration_type_raw: String::new(),
                    error_message: e,
                });
            }
        }

        (pending, errors)
    }

    /// Load a map of employee_id_number (UPPER) -> employee_id for the workspace.
    async fn load_employee_map(
        &self,
        workspace_id: Uuid,
    ) -> AppResult<std::collections::HashMap<String, Uuid>> {
        let rows = sqlx::query!(
            "SELECT id, employee_id_number FROM employees WHERE workspace_id = $1",
            workspace_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to load employees: {e}")))?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            map.insert(row.employee_id_number.to_uppercase(), row.id);
        }
        Ok(map)
    }

    /// Load a map of leave_type_code (UPPER) -> leave_type_id for the workspace.
    async fn load_leave_type_map(
        &self,
        workspace_id: Uuid,
    ) -> AppResult<std::collections::HashMap<String, Uuid>> {
        let rows = sqlx::query!(
            "SELECT id, code FROM leave_types WHERE workspace_id = $1",
            workspace_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to load leave types: {e}")))?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            map.insert(row.code.to_uppercase(), row.id);
        }
        Ok(map)
    }

    /// Generate an error Excel file containing failed rows with error descriptions.
    pub(crate) fn generate_error_excel(
        errors: &[ErrorRow],
        msgs: &LeaveImportMessages,
    ) -> AppResult<Vec<u8>> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet().set_name(msgs.error_sheet_name)?;

        let header_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::error_header_format();
        let error_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::error_message_format();

        let headers = msgs.error_headers;

        for (col, header) in headers.iter().enumerate() {
            sheet.write_string_with_format(0, col as u16, *header, &header_fmt)?;
        }

        for (idx, err) in errors.iter().enumerate() {
            let row = (idx + 1) as u32;
            sheet.write_number(row, 0, err.line_number as f64)?;
            sheet.write_string(row, 1, &err.employee_id_number)?;
            sheet.write_string(row, 2, &err.leave_type_code)?;
            sheet.write_string(row, 3, &err.start_date_raw)?;
            sheet.write_string(row, 4, &err.end_date_raw)?;
            sheet.write_string(row, 5, &err.reason)?;
            sheet.write_string(row, 6, &err.duration_type_raw)?;
            sheet.write_string_with_format(row, 7, &err.error_message, &error_fmt)?;
        }

        sheet.autofit();

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }

    /// Generate a template Excel for HR to fill in.
    pub fn generate_template(
        leave_types: &[(String, String)], // (code, name) pairs
        lang: &str,                       // requested language code (e.g., "en", "vi", "zh")
    ) -> AppResult<Vec<u8>> {
        let msgs = LeaveImportMessages::for_lang(lang);

        let sheet1_name = msgs.template_sheet_name;
        let sheet2_name = msgs.template_ref_sheet_name;
        let headers = msgs.template_headers;
        let ex_reason1 = msgs.template_example_reason_1;
        let ex_reason2 = msgs.template_example_reason_2;
        let ref_header_code = msgs.ref_header_code;
        let ref_header_name = msgs.ref_header_name;
        let ref_header_duration = msgs.ref_header_duration;
        let ref_header_desc = msgs.ref_header_desc;
        let desc_full = msgs.desc_full_day;
        let desc_morning = msgs.desc_morning;
        let desc_afternoon = msgs.desc_afternoon;

        let mut workbook = Workbook::new();

        // Sheet 1: Template
        let sheet = workbook.add_worksheet().set_name(sheet1_name)?;

        let header_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::header_format();
        let example_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::example_row_format();

        for (col, header) in headers.iter().enumerate() {
            sheet.write_string_with_format(0, col as u16, *header, &header_fmt)?;
        }

        // Example rows
        let first_code = leave_types.first().map(|(c, _)| c.as_str()).unwrap_or("AL");
        let examples = [
            [
                "NV001",
                first_code,
                "2026-04-01",
                "2026-04-02",
                ex_reason1,
                "full_day",
            ],
            [
                "NV002",
                first_code,
                "2026-04-05",
                "2026-04-05",
                ex_reason2,
                "first_half",
            ],
        ];

        for (row_idx, example) in examples.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            for (col, val) in example.iter().enumerate() {
                sheet.write_string_with_format(row, col as u16, *val, &example_fmt)?;
            }
        }

        sheet.autofit();

        // Sheet 2: Leave type codes
        let ref_sheet = workbook.add_worksheet().set_name(sheet2_name)?;

        let ref_header_fmt =
            crate::utils::excel_format_utils::ExcelFormatUtils::ref_header_format();

        ref_sheet.write_string_with_format(0, 0, ref_header_code, &ref_header_fmt)?;
        ref_sheet.write_string_with_format(0, 1, ref_header_name, &ref_header_fmt)?;

        for (idx, (code, name)) in leave_types.iter().enumerate() {
            let row = (idx + 1) as u32;
            ref_sheet.write_string(row, 0, code)?;
            ref_sheet.write_string(row, 1, name)?;
        }

        // Duration type reference
        ref_sheet.write_string_with_format(0, 3, ref_header_duration, &ref_header_fmt)?;
        ref_sheet.write_string_with_format(0, 4, ref_header_desc, &ref_header_fmt)?;

        let duration_types = [
            ("full_day", desc_full),
            ("first_half", desc_morning),
            ("second_half", desc_afternoon),
        ];
        for (idx, (code, desc)) in duration_types.iter().enumerate() {
            let row = (idx + 1) as u32;
            ref_sheet.write_string(row, 3, *code)?;
            ref_sheet.write_string(row, 4, *desc)?;
        }

        ref_sheet.autofit();

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }
}

/// Parse a date string in various formats.
fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try ISO format first: YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d);
    }
    // DD/MM/YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
        return Some(d);
    }
    // MM/DD/YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return Some(d);
    }
    // DD-MM-YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
        return Some(d);
    }

    None
}

/// Parse duration type string to LeaveDurationType.
fn parse_duration_type(s: &str) -> LeaveDurationType {
    match s.trim().to_lowercase().as_str() {
        "first_half" | "half_am" | "sáng" => LeaveDurationType::FirstHalf,
        "second_half" | "half_pm" | "chiều" => LeaveDurationType::SecondHalf,
        "custom_hours" | "custom" => LeaveDurationType::CustomHours,
        _ => LeaveDurationType::FullDay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::import_messages::LeaveImportMessages;
    use crate::utils::file_parser::cell_to_string;
    use chrono::NaiveDate;

    /// Vietnamese messages (matches the original hardcoded strings for test backwards-compat).
    fn msgs_vi() -> LeaveImportMessages {
        LeaveImportMessages::for_lang("vi")
    }

    // ---- parse_date tests ----

    // ---- parse_duration_type tests ----

    #[test]
    fn test_parse_duration_type_full_day() {
        assert_eq!(parse_duration_type("full_day"), LeaveDurationType::FullDay);
        assert_eq!(parse_duration_type(""), LeaveDurationType::FullDay);
        assert_eq!(parse_duration_type("anything"), LeaveDurationType::FullDay);
    }

    #[test]
    fn test_parse_duration_type_first_half() {
        assert_eq!(
            parse_duration_type("first_half"),
            LeaveDurationType::FirstHalf
        );
        assert_eq!(parse_duration_type("half_am"), LeaveDurationType::FirstHalf);
        assert_eq!(parse_duration_type("sáng"), LeaveDurationType::FirstHalf);
    }

    #[test]
    fn test_parse_duration_type_second_half() {
        assert_eq!(
            parse_duration_type("second_half"),
            LeaveDurationType::SecondHalf
        );
        assert_eq!(
            parse_duration_type("half_pm"),
            LeaveDurationType::SecondHalf
        );
        assert_eq!(parse_duration_type("chiều"), LeaveDurationType::SecondHalf);
    }

    #[test]
    fn test_parse_duration_type_custom_hours() {
        assert_eq!(
            parse_duration_type("custom_hours"),
            LeaveDurationType::CustomHours
        );
        assert_eq!(
            parse_duration_type("custom"),
            LeaveDurationType::CustomHours
        );
        assert_eq!(
            parse_duration_type("CUSTOM"),
            LeaveDurationType::CustomHours
        );
    }

    #[test]
    fn test_parse_duration_type_case_insensitive() {
        assert_eq!(
            parse_duration_type("FIRST_HALF"),
            LeaveDurationType::FirstHalf
        );
        assert_eq!(parse_duration_type("Full_Day"), LeaveDurationType::FullDay);
        assert_eq!(
            parse_duration_type("SECOND_HALF"),
            LeaveDurationType::SecondHalf
        );
        assert_eq!(
            parse_duration_type("Custom_Hours"),
            LeaveDurationType::CustomHours
        );
    }

    #[test]
    fn test_parse_duration_type_with_whitespace() {
        assert_eq!(
            parse_duration_type("  first_half  "),
            LeaveDurationType::FirstHalf
        );
        assert_eq!(
            parse_duration_type("  second_half  "),
            LeaveDurationType::SecondHalf
        );
    }

    // ---- cell_to_string tests ----

    // ---- generate_error_excel tests ----

    #[test]
    fn test_generate_error_excel_empty() {
        let result = LeaveImportService::generate_error_excel(&[], &msgs_vi());
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK"); // Valid xlsx
    }

    #[test]
    fn test_generate_error_excel_with_data() {
        let errors = vec![
            ErrorRow {
                line_number: 3,
                employee_id_number: "NV999".to_string(),
                leave_type_code: "AL".to_string(),
                start_date_raw: "2026-03-25".to_string(),
                end_date_raw: "2026-03-26".to_string(),
                reason: "Test".to_string(),
                duration_type_raw: "full_day".to_string(),
                error_message: "Mã nhân viên không tồn tại".to_string(),
            },
            ErrorRow {
                line_number: 5,
                employee_id_number: "NV002".to_string(),
                leave_type_code: "XYZ".to_string(),
                start_date_raw: "2026-03-28".to_string(),
                end_date_raw: "2026-03-29".to_string(),
                reason: "".to_string(),
                duration_type_raw: "first_half".to_string(),
                error_message: "Loại nghỉ không tồn tại".to_string(),
            },
        ];

        let result = LeaveImportService::generate_error_excel(&errors, &msgs_vi());
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 100);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_generate_error_excel_roundtrip_verification() {
        // Generate error Excel → re-parse with calamine → verify content
        let errors = vec![ErrorRow {
            line_number: 2,
            employee_id_number: "EMP001".to_string(),
            leave_type_code: "AL".to_string(),
            start_date_raw: "2026-04-01".to_string(),
            end_date_raw: "2026-04-02".to_string(),
            reason: "Sick".to_string(),
            duration_type_raw: "full_day".to_string(),
            error_message: "Employee not found".to_string(),
        }];

        let bytes = LeaveImportService::generate_error_excel(&errors, &msgs_vi()).unwrap();

        // Re-parse with calamine
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(&bytes)).expect("parse xlsx");
        let range = workbook.worksheet_range_at(0).unwrap().unwrap();

        // Row 0 = header, Row 1 = first error
        assert!(
            range.rows().count() >= 2,
            "Should have header + at least 1 data row"
        );

        // Verify header
        let header = range.rows().next().unwrap();
        assert_eq!(cell_to_string(header.first()), "Dòng");

        // Verify data row
        let data_row: Vec<_> = range.rows().nth(1).unwrap().to_vec();
        assert_eq!(cell_to_string(Some(&data_row[1])), "EMP001");
        assert_eq!(cell_to_string(Some(&data_row[2])), "AL");
        assert_eq!(cell_to_string(Some(&data_row[7])), "Employee not found");
    }

    #[test]
    fn test_generate_error_excel_many_rows() {
        // Stress test: 100 error rows
        let errors: Vec<ErrorRow> = (0..100)
            .map(|i| ErrorRow {
                line_number: i + 2,
                employee_id_number: format!("NV{i:04}"),
                leave_type_code: "SL".to_string(),
                start_date_raw: "2026-01-01".to_string(),
                end_date_raw: "2026-01-01".to_string(),
                reason: format!("Reason {i}"),
                duration_type_raw: "full_day".to_string(),
                error_message: format!("Error on row {i}"),
            })
            .collect();

        let result = LeaveImportService::generate_error_excel(&errors, &msgs_vi());
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Re-parse to verify row count
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(&bytes)).expect("parse");
        let range = workbook.worksheet_range_at(0).unwrap().unwrap();
        // 1 header + 100 data rows
        assert_eq!(range.rows().count(), 101);
    }

    // ---- generate_template tests ----

    #[test]
    fn test_generate_template_empty_leave_types() {
        let result = LeaveImportService::generate_template(&[], "en");
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_generate_template_with_leave_types() {
        let leave_types = vec![
            ("AL".to_string(), "Annual Leave".to_string()),
            ("SL".to_string(), "Sick Leave".to_string()),
            ("ML".to_string(), "Maternity Leave".to_string()),
        ];

        let result = LeaveImportService::generate_template(&leave_types, "en");
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 100);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_generate_template_roundtrip_verification() {
        let leave_types = vec![
            ("AL".to_string(), "Annual Leave".to_string()),
            ("SL".to_string(), "Sick Leave".to_string()),
        ];

        let bytes = LeaveImportService::generate_template(&leave_types, "vi").unwrap();

        // Re-parse with calamine
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(&bytes)).expect("parse template xlsx");

        // Verify Sheet 1: Template
        let template_range = workbook.worksheet_range_at(0).unwrap().unwrap();
        let header = template_range.rows().next().unwrap();
        assert_eq!(cell_to_string(header.first()), "Mã NV");

        // Should have 2 example rows + header = 3 rows
        assert_eq!(template_range.rows().count(), 3);

        // Verify Sheet 2: Leave type reference
        let ref_range = workbook.worksheet_range_at(1).unwrap().unwrap();
        let ref_header = ref_range.rows().next().unwrap();
        assert_eq!(cell_to_string(ref_header.first()), "Code");

        // Should have header + 2 leave types = 3 rows (plus duration types in cols 3-4)
        assert!(ref_range.rows().count() >= 3);

        // Verify leave type codes
        let first_type: Vec<_> = ref_range.rows().nth(1).unwrap().to_vec();
        assert_eq!(cell_to_string(Some(&first_type[0])), "AL");
        assert_eq!(cell_to_string(Some(&first_type[1])), "Annual Leave");
    }

    #[test]
    fn test_generate_template_example_uses_first_leave_type() {
        let leave_types = vec![("CUSTOM".to_string(), "Custom Leave".to_string())];

        let bytes = LeaveImportService::generate_template(&leave_types, "vi").unwrap();
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(&bytes)).expect("parse");
        let range = workbook.worksheet_range_at(0).unwrap().unwrap();

        // Example row should use the first leave type code
        let example_row: Vec<_> = range.rows().nth(1).unwrap().to_vec();
        assert_eq!(cell_to_string(Some(&example_row[1])), "CUSTOM");
    }

    // ---- LeaveImportResult tests ----

    #[test]
    fn test_leave_import_result_clone_debug() {
        let result = LeaveImportResult {
            success_count: 5,
            error_count: 2,
            error_file_path: Some("/tmp/errors.xlsx".into()),
        };
        let cloned = result.clone();
        assert_eq!(cloned.success_count, 5);
        assert_eq!(cloned.error_count, 2);
        assert_eq!(cloned.error_file_path, Some("/tmp/errors.xlsx".to_string()));

        // Debug should not panic
        let _debug = format!("{result:?}");
    }

    // ---- ErrorRow tests ----

    #[test]
    fn test_error_row_clone_debug() {
        let err = ErrorRow {
            line_number: 1,
            employee_id_number: "E1".into(),
            leave_type_code: "AL".into(),
            start_date_raw: "2026-01-01".into(),
            end_date_raw: "2026-01-02".into(),
            reason: "test".into(),
            duration_type_raw: "full_day".into(),
            error_message: "some error".into(),
        };
        let cloned = err.clone();
        assert_eq!(cloned.line_number, 1);
        let _debug = format!("{err:?}");
    }

    // ---- PendingLeaveRow tests ----

    #[test]
    fn test_pending_leave_row_clone_debug() {
        let row = PendingLeaveRow {
            line_number: 5,
            employee_id_number: "NV005".into(),
            leave_type_code: "SL".into(),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            reason: Some("sick".into()),
            duration_type: LeaveDurationType::FullDay,
        };
        let cloned = row.clone();
        assert_eq!(cloned.employee_id_number, "NV005");
        let _debug = format!("{row:?}");
    }
}
