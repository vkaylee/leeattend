//! Leave Import Service
//!
//! Parses Excel files containing leave requests, validates rows, submits valid ones,
//! and generates an error Excel file for failed rows.

use crate::utils::datetime::parse_date;
use chrono::NaiveDate;
use rust_xlsxwriter::Workbook;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::models::{LeaveDurationType, SubmitLeaveRequest};
use crate::services::import_messages::LeaveImportMessages;
use crate::services::leave_service::LeaveSubmission;
use crate::services::leave_service::LeaveService;
use crate::utils::i18n::I18nKey;

use crate::models::AttendanceImportJob;

use crate::services::import_pipeline::{ImportPipeline, ImportStrategy};

/// A parsed row from the Excel file, pending validation.
#[derive(Debug, Clone)]
pub struct PendingLeaveRow {
    pub line_number: usize,
    pub employee_id_number: String,
    pub leave_type_code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub duration_type: LeaveDurationType,
}

pub struct LeaveImportService {
    deps: crate::worker::deps::WorkerDeps,
}

pub struct LeaveImportStrategy {
    pub msgs: LeaveImportMessages,
    pub auto_approve: bool,
    pub user_id: Uuid,
    pub employees_cache: std::collections::HashMap<String, Uuid>,
    pub leave_types_cache: std::collections::HashMap<String, Uuid>,
    pub leave_submission: Arc<dyn LeaveSubmission>,
}

#[async_trait::async_trait]
impl ImportStrategy for LeaveImportStrategy {
    type RowData = PendingLeaveRow;

    fn expected_headers(&self) -> &'static [&'static str] {
        &[
            "employee_code",
            "leave_type_code",
            "start_date",
            "end_date",
            "reason",
            "duration_type",
        ]
    }

    fn map_row(
        &mut self,
        row_data: &std::collections::HashMap<String, String>,
        current_line: usize,
        _workspace_tz: chrono_tz::Tz,
    ) -> Result<Self::RowData, crate::errors::AppError> {
        let raw_emp_id = row_data
            .get("employee_code")
            .map_or("", |s| s.trim())
            .to_string();
        let emp_id = crate::utils::normalize_employee_id(&raw_emp_id);
        let leave_code = row_data
            .get("leave_type_code")
            .map_or("", |s| s.trim())
            .to_string();
        let start_str = row_data
            .get("start_date")
            .map_or("", |s| s.trim())
            .to_string();
        let end_str = row_data
            .get("end_date")
            .map_or("", |s| s.trim())
            .to_string();
        let reason = row_data.get("reason").map_or("", |s| s.trim()).to_string();
        let duration_str = row_data
            .get("duration_type")
            .map_or("", |s| s.trim())
            .to_string();

        if emp_id.is_empty() {
            let msg = self.msgs.missing_employee_id.to_string();
            return Err(crate::errors::AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ));
        }
        if leave_code.is_empty() {
            let msg = self.msgs.missing_leave_type.to_string();
            return Err(crate::errors::AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ));
        }
        let Some(start_date) = parse_date(&start_str) else {
            let msg = self.msgs.invalid_start_date.to_string();
            return Err(crate::errors::AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ));
        };
        let Some(end_date) = parse_date(&end_str) else {
            let msg = self.msgs.invalid_end_date.to_string();
            return Err(crate::errors::AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ));
        };
        if end_date < start_date {
            let msg = self.msgs.end_before_start.to_string();
            return Err(crate::errors::AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ));
        }
        let duration_type = parse_duration_type(&duration_str);

        Ok(PendingLeaveRow {
            line_number: current_line,
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
        })
    }

    async fn init(
        &mut self,
        deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        _rows: &[Self::RowData],
    ) -> Result<(), crate::errors::AppError> {
        let options = crate::models::EmployeeFilterOptions::default()
            .with_page(1)
            .with_limit(100_000);
        let (emps, _) = deps
            .employee_repo
            .get_all_employees(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                options,
            )
            .await
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Failed to load employees: {e}"))
            })?;

        let employments = deps
            .employment_repo
            .get_active_employments_for_workspace(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
            )
            .await
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Failed to load employments: {e}"))
            })?;

        let mut emp_to_employment = std::collections::HashMap::new();
        for emp_ment in employments {
            emp_to_employment.insert(emp_ment.employee_id(), emp_ment.id());
        }

        for emp in emps {
            tracing::info!(
                "Loaded employee into cache: id_number = '{}'",
                emp.employee_id_number()
            );
            if !emp.employee_id_number().is_empty() {
                if let Some(employment_id) = emp_to_employment.get(&emp.id()) {
                    self.employees_cache
                        .insert(emp.employee_id_number().to_uppercase(), *employment_id);
                }
            }
        }

        let leaves = deps
            .leave_policy_repo
            .get_leave_types(job.workspace_id)
            .await
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Failed to load leave types: {e}"))
            })?;

        for lt in leaves {
            self.leave_types_cache
                .insert(lt.code().to_uppercase(), lt.id());
        }

        Ok(())
    }

    async fn process_row(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        row: &Self::RowData,
    ) -> Result<Option<Uuid>, crate::errors::AppError> {
        let employment_id = *self
            .employees_cache
            .get(&row.employee_id_number.to_uppercase())
            .ok_or_else(|| {
                let msg = self
                    .msgs
                    .employee_not_found_tpl
                    .replace("{}", &row.employee_id_number);
                crate::errors::AppError::validation_args(I18nKey::Validation, [("details", msg)])
            })?;

        let leave_type_id = *self
            .leave_types_cache
            .get(&row.leave_type_code.to_uppercase())
            .ok_or_else(|| {
                let msg = self
                    .msgs
                    .leave_type_not_found_tpl
                    .replace("{}", &row.leave_type_code);
                crate::errors::AppError::validation_args(I18nKey::Validation, [("details", msg)])
            })?;

        let submit = SubmitLeaveRequest {
            employment_id,
            leave_type_id,
            start_date: row.start_date,
            end_date: row.end_date,
            reason: row.reason.clone(),
            duration_type: row.duration_type,
            start_time: None,
            end_time: None,
            attachment_file_ids: None,
        };

        match self
            .leave_submission
            .submit_leave_request(job.workspace_id, self.user_id, submit, self.auto_approve)
            .await
        {
            Ok(res) => Ok(Some(res.id())),
            Err(e) => Err(crate::errors::AppError::Internal(format!(
                "{}: {e}",
                self.msgs.submission_error_prefix
            ))),
        }
    }
}

impl LeaveImportService {
    pub fn new(deps: crate::worker::deps::WorkerDeps) -> Self {
        Self { deps }
    }

    /// Process an imported Excel file of leave requests.
    #[allow(clippy::too_many_lines)]
    pub async fn process_job(
        &self,
        import_job_id: Uuid,
        workspace_id: Uuid,
        user_id: Uuid,
        file_path: &str,
        auto_approve: bool,
    ) -> Result<(), ()> {
        let file_bytes = tokio::fs::read(file_path).await.map_err(|_e| ())?;

        let Ok(Some(job)) = self
            .deps
            .import_job_repo
            .get_job_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                import_job_id,
            )
            .await
        else {
            return Err(());
        };

        let workspace = self
            .deps
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|_err| ())?
            .ok_or(())?;

        let lang = workspace.default_language().to_string();
        let tz_str = workspace.timezone().to_string();

        let workspace_tz: chrono_tz::Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);

        let msgs = LeaveImportMessages::for_lang(&lang);

        let leave_submission: Arc<dyn LeaveSubmission> = self
            .deps
            .leave_submission
            .clone()
            .unwrap_or_else(|| {
                Arc::new(LeaveService::new(
                    Arc::clone(&self.deps.leave_balance_repo),
                    Arc::clone(&self.deps.leave_request_repo),
                    Arc::clone(&self.deps.employee_repo),
                    Arc::clone(&self.deps.employment_repo),
                    Arc::clone(&self.deps.roster_repo),
                    Arc::clone(&self.deps.leave_accrual_log_repo),
                    Arc::clone(&self.deps.workspace_config_repo),
                    Arc::clone(&self.deps.user_repo),
                    Arc::clone(&self.deps.workspace_repo),
                    None,
                ))
            });

        let strategy = LeaveImportStrategy {
            msgs,
            auto_approve,
            user_id,
            employees_cache: std::collections::HashMap::new(),
            leave_types_cache: std::collections::HashMap::new(),
            leave_submission,
        };

        let pipeline = ImportPipeline {
            strategy,
            deps: self.deps.clone(),
            import_repo: &*self.deps.import_job_repo,
            job: &job,
            file_bytes,
            workspace_tz,
            start_index: None,
            end_index: None,
            is_chunk: false,
            queue_repo: None,
        };

        pipeline.run().await
    }

    /// Generate a template Excel for HR to fill in.
    pub fn generate_template(
        leave_types: &[(String, String)], // (code, name) pairs
        lang: &str,                       // requested language code (e.g., "en", "vi", "zh")
    ) -> AppResult<Vec<u8>> {
        let msgs = LeaveImportMessages::for_lang(lang);

        let sheet1_name = msgs.template_sheet_name;
        let sheet2_name = msgs.template_ref_sheet_name;
        let headers = [
            "employee_code",
            "leave_type_code",
            "start_date",
            "end_date",
            "reason",
            "duration_type",
        ];
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
            sheet.write_string_with_format(
                0,
                u16::try_from(col).unwrap_or(0),
                *header,
                &header_fmt,
            )?;
        }

        // Example rows
        let first_code = leave_types.first().map_or("AL", |(c, _)| c.as_str());
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
            let row = u32::try_from(row_idx + 1).unwrap_or(0);
            for (col, val) in example.iter().enumerate() {
                sheet.write_string_with_format(
                    row,
                    u16::try_from(col).unwrap_or(0),
                    *val,
                    &example_fmt,
                )?;
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
            let row = u32::try_from(idx + 1).unwrap_or(0);
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
            let row = u32::try_from(idx + 1).unwrap_or(0);
            ref_sheet.write_string(row, 3, *code)?;
            ref_sheet.write_string(row, 4, *desc)?;
        }

        ref_sheet.autofit();

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }
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
    use crate::utils::file_parser::cell_to_string;
    use calamine::Reader;
    use calamine::Xlsx;
    use chrono::NaiveDate;
    use std::io::Cursor;

    // ---- parse_date tests ----

    #[test]
    fn test_parse_date_formats() {
        // ISO format
        assert_eq!(
            parse_date("2026-05-29"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // ISO format with time
        assert_eq!(
            parse_date("2026-05-29 00:00:00"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // ISO format with time & T
        assert_eq!(
            parse_date("2026-05-29T13:45:00Z"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // DD/MM/YYYY
        assert_eq!(
            parse_date("29/05/2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // MM/DD/YYYY
        assert_eq!(
            parse_date("05/29/2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // DD-MM-YYYY
        assert_eq!(
            parse_date("29-05-2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // YYYY/MM/DD
        assert_eq!(
            parse_date("2026/05/29"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // DD-MM-YY (2-digit year)
        assert_eq!(
            parse_date("29-05-26"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // DD/MM/YY (2-digit year)
        assert_eq!(
            parse_date("29/05/26"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // MM/DD/YY (2-digit year)
        assert_eq!(
            parse_date("05/29/26"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );

        // Edge Cases:
        // Single digit day/month
        assert_eq!(
            parse_date("2026-5-9"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 9).unwrap())
        );
        assert_eq!(
            parse_date("9-5-26"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 9).unwrap())
        );
        // Spaces around separators
        assert_eq!(
            parse_date("29 - 05 - 26"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        assert_eq!(
            parse_date(" 2026 / 05 / 29 "),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // Dot separators
        assert_eq!(
            parse_date("29.05.2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        assert_eq!(
            parse_date("29.05.26"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        assert_eq!(
            parse_date("2026.05.29"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        // Datetime and Timezone structures
        assert_eq!(
            parse_date("29/05/2026 10:30:00"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );
        assert_eq!(
            parse_date("29-05-26T10:30:00.000Z"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 29).unwrap())
        );

        // Invalid formats
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("invalid"), None);
    }

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
        assert_eq!(cell_to_string(header.first()), "employee_code");

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
