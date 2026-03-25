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

use crate::models::{LeaveDurationType, SubmitLeaveRequest};
use crate::repositories::leave_balance_repository::SqlxLeaveBalanceRepository;
use crate::repositories::leave_request_repository::SqlxLeaveRequestRepository;
use crate::repositories::roster::SqlxRosterRepository;
use crate::repositories::{SqlxEmployeeRepository, EmployeeRepository};
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
    ) -> Result<LeaveImportResult, String> {
        // Read file
        let file_bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read file: {e}"))?;

        // Parse rows from Excel or CSV
        let (pending_rows, mut errors) = self.parse_file(&file_bytes);

        if pending_rows.is_empty() && errors.is_empty() {
            return Err("File rỗng hoặc không có dữ liệu hợp lệ".to_string());
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
                        error_message: format!("Mã nhân viên '{}' không tồn tại", row.employee_id_number),
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
                        error_message: format!("Loại nghỉ '{}' không tồn tại", row.leave_type_code),
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
                        error_message: format!("Lỗi khi tạo đơn: {e}"),
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
            match Self::generate_error_excel(&errors) {
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
    fn parse_file(&self, file_bytes: &[u8]) -> (Vec<PendingLeaveRow>, Vec<ErrorRow>) {
        let mut pending = Vec::new();
        let mut errors = Vec::new();

        // Try XLSX first, then XLS, then CSV
        let mut is_excel = false;
        if let Ok(mut reader) = Xlsx::new(Cursor::new(file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::parse_excel_range(range, &mut pending, &mut errors);
            }
        }
        if !is_excel {
            if let Ok(mut reader) = Xls::new(Cursor::new(file_bytes)) {
                if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                    is_excel = true;
                    Self::parse_excel_range(range, &mut pending, &mut errors);
                }
            }
        }
        if !is_excel {
            Self::parse_csv(file_bytes, &mut pending, &mut errors);
        }

        (pending, errors)
    }

    /// Parse Excel range (first sheet).
    /// Expected columns: A=employee_id_number, B=leave_type_code, C=start_date, D=end_date, E=reason, F=duration_type
    fn parse_excel_range(
        range: calamine::Range<Data>,
        pending: &mut Vec<PendingLeaveRow>,
        errors: &mut Vec<ErrorRow>,
    ) {
        for (idx, row) in range.rows().enumerate() {
            if idx == 0 {
                continue; // Skip header
            }
            let line_number = idx + 1;

            let emp_id = cell_to_string(row.first());
            let leave_code = cell_to_string(row.get(1));
            let start_str = cell_to_string(row.get(2));
            let end_str = cell_to_string(row.get(3));
            let reason = cell_to_string(row.get(4));
            let duration_str = cell_to_string(row.get(5));

            if emp_id.is_empty() && leave_code.is_empty() {
                continue; // Skip empty rows
            }

            if emp_id.is_empty() {
                errors.push(ErrorRow {
                    line_number,
                    employee_id_number: emp_id,
                    leave_type_code: leave_code,
                    start_date_raw: start_str,
                    end_date_raw: end_str,
                    reason: reason.clone(),
                    duration_type_raw: duration_str,
                    error_message: "Thiếu mã nhân viên".to_string(),
                });
                continue;
            }

            if leave_code.is_empty() {
                errors.push(ErrorRow {
                    line_number,
                    employee_id_number: emp_id,
                    leave_type_code: leave_code,
                    start_date_raw: start_str,
                    end_date_raw: end_str,
                    reason: reason.clone(),
                    duration_type_raw: duration_str,
                    error_message: "Thiếu mã loại nghỉ".to_string(),
                });
                continue;
            }

            let start_date = match parse_date(&start_str) {
                Some(d) => d,
                None => {
                    errors.push(ErrorRow {
                        line_number,
                        employee_id_number: emp_id,
                        leave_type_code: leave_code,
                        start_date_raw: start_str,
                        end_date_raw: end_str,
                        reason: reason.clone(),
                        duration_type_raw: duration_str,
                        error_message: "Ngày bắt đầu sai format (yêu cầu YYYY-MM-DD)".to_string(),
                    });
                    continue;
                }
            };

            let end_date = match parse_date(&end_str) {
                Some(d) => d,
                None => {
                    errors.push(ErrorRow {
                        line_number,
                        employee_id_number: emp_id,
                        leave_type_code: leave_code,
                        start_date_raw: start_str,
                        end_date_raw: end_str,
                        reason: reason.clone(),
                        duration_type_raw: duration_str,
                        error_message: "Ngày kết thúc sai format (yêu cầu YYYY-MM-DD)".to_string(),
                    });
                    continue;
                }
            };

            if end_date < start_date {
                errors.push(ErrorRow {
                    line_number,
                    employee_id_number: emp_id,
                    leave_type_code: leave_code,
                    start_date_raw: start_str,
                    end_date_raw: end_str,
                    reason: reason.clone(),
                    duration_type_raw: duration_str,
                    error_message: "Ngày kết thúc phải >= ngày bắt đầu".to_string(),
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
                reason: if reason.is_empty() { None } else { Some(reason) },
                duration_type,
            });
        }
    }

    /// Parse CSV fallback.
    fn parse_csv(
        file_bytes: &[u8],
        pending: &mut Vec<PendingLeaveRow>,
        errors: &mut Vec<ErrorRow>,
    ) {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(Cursor::new(file_bytes));

        for (idx, result) in reader.records().enumerate() {
            let line_number = idx + 2; // 1-indexed + header skip
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    errors.push(ErrorRow {
                        line_number,
                        employee_id_number: String::new(),
                        leave_type_code: String::new(),
                        start_date_raw: String::new(),
                        end_date_raw: String::new(),
                        reason: String::new(),
                        duration_type_raw: String::new(),
                        error_message: format!("Lỗi đọc dòng CSV: {e}"),
                    });
                    continue;
                }
            };

            let emp_id = record.get(0).unwrap_or("").trim().to_string();
            let leave_code = record.get(1).unwrap_or("").trim().to_string();
            let start_str = record.get(2).unwrap_or("").trim().to_string();
            let end_str = record.get(3).unwrap_or("").trim().to_string();
            let reason = record.get(4).unwrap_or("").trim().to_string();
            let duration_str = record.get(5).unwrap_or("").trim().to_string();

            if emp_id.is_empty() && leave_code.is_empty() {
                continue;
            }

            if emp_id.is_empty() || leave_code.is_empty() {
                errors.push(ErrorRow {
                    line_number,
                    employee_id_number: emp_id,
                    leave_type_code: leave_code,
                    start_date_raw: start_str,
                    end_date_raw: end_str,
                    reason: reason.clone(),
                    duration_type_raw: duration_str,
                    error_message: "Thiếu mã nhân viên hoặc mã loại nghỉ".to_string(),
                });
                continue;
            }

            let start_date = match parse_date(&start_str) {
                Some(d) => d,
                None => {
                    errors.push(ErrorRow {
                        line_number,
                        employee_id_number: emp_id,
                        leave_type_code: leave_code,
                        start_date_raw: start_str,
                        end_date_raw: end_str,
                        reason: reason.clone(),
                        duration_type_raw: duration_str,
                        error_message: "Ngày bắt đầu sai format".to_string(),
                    });
                    continue;
                }
            };

            let end_date = match parse_date(&end_str) {
                Some(d) => d,
                None => {
                    errors.push(ErrorRow {
                        line_number,
                        employee_id_number: emp_id,
                        leave_type_code: leave_code,
                        start_date_raw: start_str,
                        end_date_raw: end_str,
                        reason: reason.clone(),
                        duration_type_raw: duration_str,
                        error_message: "Ngày kết thúc sai format".to_string(),
                    });
                    continue;
                }
            };

            let duration_type = parse_duration_type(&duration_str);

            pending.push(PendingLeaveRow {
                line_number,
                employee_id_number: emp_id,
                leave_type_code: leave_code,
                start_date,
                end_date,
                reason: if reason.is_empty() { None } else { Some(reason) },
                duration_type,
            });
        }
    }

    /// Load a map of employee_id_number (UPPER) -> employee_id for the workspace.
    async fn load_employee_map(
        &self,
        workspace_id: Uuid,
    ) -> Result<std::collections::HashMap<String, Uuid>, String> {
        let rows = sqlx::query!(
            "SELECT id, employee_id_number FROM employees WHERE workspace_id = $1",
            workspace_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load employees: {e}"))?;

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
    ) -> Result<std::collections::HashMap<String, Uuid>, String> {
        let rows = sqlx::query!(
            "SELECT id, code FROM leave_types WHERE workspace_id = $1",
            workspace_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load leave types: {e}"))?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            map.insert(row.code.to_uppercase(), row.id);
        }
        Ok(map)
    }

    /// Generate an error Excel file containing failed rows with error descriptions.
    pub(crate) fn generate_error_excel(errors: &[ErrorRow]) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let sheet = workbook
            .add_worksheet()
            .set_name("Dòng lỗi")
            .map_err(|e| e.to_string())?;

        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0xE53935))
            .set_font_color(Color::RGB(0xFFFFFF));

        let error_fmt = Format::new()
            .set_font_color(Color::RGB(0xD32F2F))
            .set_bold();

        let headers = [
            "Dòng", "Mã NV", "Loại nghỉ", "Ngày bắt đầu",
            "Ngày kết thúc", "Lý do", "Loại thời gian", "❌ Lỗi",
        ];

        for (col, header) in headers.iter().enumerate() {
            sheet
                .write_string_with_format(0, col as u16, *header, &header_fmt)
                .map_err(|e| e.to_string())?;
        }

        for (idx, err) in errors.iter().enumerate() {
            let row = (idx + 1) as u32;
            sheet.write_number(row, 0, err.line_number as f64).map_err(|e| e.to_string())?;
            sheet.write_string(row, 1, &err.employee_id_number).map_err(|e| e.to_string())?;
            sheet.write_string(row, 2, &err.leave_type_code).map_err(|e| e.to_string())?;
            sheet.write_string(row, 3, &err.start_date_raw).map_err(|e| e.to_string())?;
            sheet.write_string(row, 4, &err.end_date_raw).map_err(|e| e.to_string())?;
            sheet.write_string(row, 5, &err.reason).map_err(|e| e.to_string())?;
            sheet.write_string(row, 6, &err.duration_type_raw).map_err(|e| e.to_string())?;
            sheet
                .write_string_with_format(row, 7, &err.error_message, &error_fmt)
                .map_err(|e| e.to_string())?;
        }

        sheet.autofit();

        workbook
            .save_to_buffer()
            .map_err(|e| format!("Excel write error: {e}"))
    }

    /// Generate a template Excel for HR to fill in.
    pub fn generate_template(
        leave_types: &[(String, String)], // (code, name) pairs
        lang: &str, // requested language code (e.g., "en", "vi", "zh")
    ) -> Result<Vec<u8>, String> {
        // --- Translations ---
        let (
            sheet1_name, sheet2_name,
            headers,
            ex_reason1, ex_reason2,
            ref_header_code, ref_header_name,
            ref_header_duration, ref_header_desc,
            desc_full, desc_morning, desc_afternoon
        ) = match lang {
            "vi" => (
                "Mẫu nhập", "Danh sách loại nghỉ",
                ["Mã NV", "Loại nghỉ (code)", "Ngày bắt đầu", "Ngày kết thúc", "Lý do", "Loại thời gian"],
                "Việc gia đình", "Khám bệnh",
                "Code", "Tên loại nghỉ",
                "Loại thời gian", "Mô tả",
                "Cả ngày (mặc định)", "Nửa ngày sáng", "Nửa ngày chiều"
            ),
            "zh" => (
                "输入模板", "假期类型列表",
                ["员工编号", "假期类型(code)", "开始日期", "结束日期", "原因", "时长类型"],
                "家庭事务", "看病",
                "代码", "假期类型名称",
                "时长类型", "描述",
                "全天(默认)", "上午半天", "下午半天"
            ),
            _ => (
                "Entry Template", "Leave Types List",
                ["Emp ID", "Leave Type (code)", "Start Date", "End Date", "Reason", "Duration Type"],
                "Family matter", "Doctor appointment",
                "Code", "Leave Type Name",
                "Duration Type", "Description",
                "Full day (default)", "Morning half", "Afternoon half"
            ),
        };

        let mut workbook = Workbook::new();

        // Sheet 1: Template
        let sheet = workbook
            .add_worksheet()
            .set_name(sheet1_name)
            .map_err(|e| e.to_string())?;

        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::RGB(0xFFFFFF));

        let example_fmt = Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x757575));

        for (col, header) in headers.iter().enumerate() {
            sheet
                .write_string_with_format(0, col as u16, *header, &header_fmt)
                .map_err(|e| e.to_string())?;
        }

        // Example rows
        let first_code = leave_types.first().map(|(c, _)| c.as_str()).unwrap_or("AL");
        let examples = [
            [
                "NV001", first_code, "2026-04-01", "2026-04-02", ex_reason1, "full_day",
            ],
            [
                "NV002", first_code, "2026-04-05", "2026-04-05", ex_reason2, "first_half",
            ],
        ];

        for (row_idx, example) in examples.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            for (col, val) in example.iter().enumerate() {
                sheet
                    .write_string_with_format(row, col as u16, *val, &example_fmt)
                    .map_err(|e| e.to_string())?;
            }
        }

        sheet.autofit();

        // Sheet 2: Leave type codes
        let ref_sheet = workbook
            .add_worksheet()
            .set_name(sheet2_name)
            .map_err(|e| e.to_string())?;

        let ref_header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x2E7D32))
            .set_font_color(Color::RGB(0xFFFFFF));

        ref_sheet
            .write_string_with_format(0, 0, ref_header_code, &ref_header_fmt)
            .map_err(|e| e.to_string())?;
        ref_sheet
            .write_string_with_format(0, 1, ref_header_name, &ref_header_fmt)
            .map_err(|e| e.to_string())?;

        for (idx, (code, name)) in leave_types.iter().enumerate() {
            let row = (idx + 1) as u32;
            ref_sheet.write_string(row, 0, code).map_err(|e| e.to_string())?;
            ref_sheet.write_string(row, 1, name).map_err(|e| e.to_string())?;
        }

        // Duration type reference
        ref_sheet
            .write_string_with_format(0, 3, ref_header_duration, &ref_header_fmt)
            .map_err(|e| e.to_string())?;
        ref_sheet
            .write_string_with_format(0, 4, ref_header_desc, &ref_header_fmt)
            .map_err(|e| e.to_string())?;

        let duration_types = [
            ("full_day", desc_full),
            ("first_half", desc_morning),
            ("second_half", desc_afternoon),
        ];
        for (idx, (code, desc)) in duration_types.iter().enumerate() {
            let row = (idx + 1) as u32;
            ref_sheet.write_string(row, 3, *code).map_err(|e| e.to_string())?;
            ref_sheet.write_string(row, 4, *desc).map_err(|e| e.to_string())?;
        }

        ref_sheet.autofit();

        workbook
            .save_to_buffer()
            .map_err(|e| format!("Excel write error: {e}"))
    }
}

/// Convert a calamine cell to a trimmed string.
fn cell_to_string(cell: Option<&Data>) -> String {
    match cell {
        Some(Data::String(s)) => s.trim().to_string(),
        Some(Data::Int(n)) => n.to_string(),
        Some(Data::Float(f)) => {
            // Handle Excel numeric dates or integer-like floats
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Some(Data::DateTime(dt)) => {
            // calamine DateTime → try to format as date
            if let Some(dt_val) = dt.as_datetime() {
                dt_val.date().to_string()
            } else {
                String::new()
            }
        }
        Some(Data::Bool(b)) => b.to_string(),
        _ => String::new(),
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
    use chrono::NaiveDate;

    // ---- parse_date tests ----

    #[test]
    fn test_parse_date_iso() {
        assert_eq!(
            parse_date("2026-03-25"),
            Some(NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()),
        );
    }

    #[test]
    fn test_parse_date_dd_mm_yyyy() {
        assert_eq!(
            parse_date("25/03/2026"),
            Some(NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()),
        );
    }

    #[test]
    fn test_parse_date_dd_dash_mm_yyyy() {
        assert_eq!(
            parse_date("25-03-2026"),
            Some(NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()),
        );
    }

    #[test]
    fn test_parse_date_mm_dd_yyyy() {
        // MM/DD/YYYY format: March 5th, not May 3rd
        // This is ambiguous but 03/05/2026 could be DD/MM or MM/DD
        // Our parser tries DD/MM/YYYY first, then MM/DD/YYYY
        let result = parse_date("12/25/2026");
        // 12/25 is unambiguously MM/DD since day=25 > 12
        assert_eq!(result, Some(NaiveDate::from_ymd_opt(2026, 12, 25).unwrap()));
    }

    #[test]
    fn test_parse_date_with_whitespace() {
        assert_eq!(
            parse_date("  2026-03-25  "),
            Some(NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()),
        );
    }

    #[test]
    fn test_parse_date_empty() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("   "), None);
    }

    #[test]
    fn test_parse_date_invalid() {
        assert_eq!(parse_date("abc"), None);
        assert_eq!(parse_date("2026-13-40"), None);
    }

    #[test]
    fn test_parse_date_boundary_values() {
        // First day of year
        assert_eq!(
            parse_date("2026-01-01"),
            Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        );
        // Last day of year
        assert_eq!(
            parse_date("2026-12-31"),
            Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        );
        // Leap year Feb 29
        assert_eq!(
            parse_date("2024-02-29"),
            Some(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()),
        );
        // Non-leap year Feb 29 → None
        assert_eq!(parse_date("2026-02-29"), None);
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
        assert_eq!(parse_duration_type("first_half"), LeaveDurationType::FirstHalf);
        assert_eq!(parse_duration_type("half_am"), LeaveDurationType::FirstHalf);
        assert_eq!(parse_duration_type("sáng"), LeaveDurationType::FirstHalf);
    }

    #[test]
    fn test_parse_duration_type_second_half() {
        assert_eq!(parse_duration_type("second_half"), LeaveDurationType::SecondHalf);
        assert_eq!(parse_duration_type("half_pm"), LeaveDurationType::SecondHalf);
        assert_eq!(parse_duration_type("chiều"), LeaveDurationType::SecondHalf);
    }

    #[test]
    fn test_parse_duration_type_custom_hours() {
        assert_eq!(parse_duration_type("custom_hours"), LeaveDurationType::CustomHours);
        assert_eq!(parse_duration_type("custom"), LeaveDurationType::CustomHours);
        assert_eq!(parse_duration_type("CUSTOM"), LeaveDurationType::CustomHours);
    }

    #[test]
    fn test_parse_duration_type_case_insensitive() {
        assert_eq!(parse_duration_type("FIRST_HALF"), LeaveDurationType::FirstHalf);
        assert_eq!(parse_duration_type("Full_Day"), LeaveDurationType::FullDay);
        assert_eq!(parse_duration_type("SECOND_HALF"), LeaveDurationType::SecondHalf);
        assert_eq!(parse_duration_type("Custom_Hours"), LeaveDurationType::CustomHours);
    }

    #[test]
    fn test_parse_duration_type_with_whitespace() {
        assert_eq!(parse_duration_type("  first_half  "), LeaveDurationType::FirstHalf);
        assert_eq!(parse_duration_type("  second_half  "), LeaveDurationType::SecondHalf);
    }

    // ---- cell_to_string tests ----

    #[test]
    fn test_cell_to_string_string() {
        assert_eq!(cell_to_string(Some(&Data::String("NV001".into()))), "NV001");
        assert_eq!(cell_to_string(Some(&Data::String("  NV002  ".into()))), "NV002");
    }

    #[test]
    fn test_cell_to_string_int() {
        assert_eq!(cell_to_string(Some(&Data::Int(42))), "42");
        assert_eq!(cell_to_string(Some(&Data::Int(0))), "0");
        assert_eq!(cell_to_string(Some(&Data::Int(-1))), "-1");
    }

    #[test]
    fn test_cell_to_string_float() {
        assert_eq!(cell_to_string(Some(&Data::Float(100.0))), "100");
        assert_eq!(cell_to_string(Some(&Data::Float(3.14))), "3.14");
        assert_eq!(cell_to_string(Some(&Data::Float(0.0))), "0");
    }

    #[test]
    fn test_cell_to_string_bool() {
        assert_eq!(cell_to_string(Some(&Data::Bool(true))), "true");
        assert_eq!(cell_to_string(Some(&Data::Bool(false))), "false");
    }

    #[test]
    fn test_cell_to_string_empty() {
        assert_eq!(cell_to_string(None), "");
        assert_eq!(cell_to_string(Some(&Data::Empty)), "");
    }

    // ---- generate_error_excel tests ----

    #[test]
    fn test_generate_error_excel_empty() {
        let result = LeaveImportService::generate_error_excel(&[]);
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

        let result = LeaveImportService::generate_error_excel(&errors);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 100);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_generate_error_excel_roundtrip_verification() {
        // Generate error Excel → re-parse with calamine → verify content
        let errors = vec![
            ErrorRow {
                line_number: 2,
                employee_id_number: "EMP001".to_string(),
                leave_type_code: "AL".to_string(),
                start_date_raw: "2026-04-01".to_string(),
                end_date_raw: "2026-04-02".to_string(),
                reason: "Sick".to_string(),
                duration_type_raw: "full_day".to_string(),
                error_message: "Employee not found".to_string(),
            },
        ];

        let bytes = LeaveImportService::generate_error_excel(&errors).unwrap();

        // Re-parse with calamine
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(&bytes)).expect("parse xlsx");
        let range = workbook.worksheet_range_at(0).unwrap().unwrap();

        // Row 0 = header, Row 1 = first error
        assert!(range.rows().count() >= 2, "Should have header + at least 1 data row");

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

        let result = LeaveImportService::generate_error_excel(&errors);
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
        let leave_types = vec![
            ("CUSTOM".to_string(), "Custom Leave".to_string()),
        ];

        let bytes = LeaveImportService::generate_template(&leave_types, "vi").unwrap();
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(&bytes)).expect("parse");
        let range = workbook.worksheet_range_at(0).unwrap().unwrap();

        // Example row should use the first leave type code
        let example_row: Vec<_> = range.rows().nth(1).unwrap().to_vec();
        assert_eq!(cell_to_string(Some(&example_row[1])), "CUSTOM");
    }

    // ---- parse_csv tests ----

    #[test]
    fn test_parse_csv_valid() {
        let csv = "Mã NV,Loại nghỉ,Ngày bắt đầu,Ngày kết thúc,Lý do,Loại thời gian\n\
                    NV001,AL,2026-03-25,2026-03-26,Việc gia đình,full_day\n\
                    NV002,SL,2026-04-01,2026-04-01,Khám bệnh,first_half\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 2);
        assert_eq!(errors.len(), 0);
        assert_eq!(pending[0].employee_id_number, "NV001");
        assert_eq!(pending[0].leave_type_code, "AL");
        assert_eq!(pending[0].start_date, NaiveDate::from_ymd_opt(2026, 3, 25).unwrap());
        assert_eq!(pending[0].end_date, NaiveDate::from_ymd_opt(2026, 3, 26).unwrap());
        assert_eq!(pending[0].reason, Some("Việc gia đình".to_string()));
        assert_eq!(pending[0].duration_type, LeaveDurationType::FullDay);
        assert_eq!(pending[1].duration_type, LeaveDurationType::FirstHalf);
    }

    #[test]
    fn test_parse_csv_with_errors() {
        let csv = "Mã NV,Loại nghỉ,Ngày bắt đầu,Ngày kết thúc,Lý do,Loại thời gian\n\
                    ,AL,2026-03-25,2026-03-26,Missing emp id,full_day\n\
                    NV001,,2026-03-25,2026-03-26,Missing leave type,full_day\n\
                    NV002,SL,abc,2026-03-26,Bad date,full_day\n\
                    NV003,SL,2026-03-25,2026-03-26,Valid,full_day\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 1); // Only NV003 is valid
        assert_eq!(errors.len(), 3); // 3 rows with errors
        assert_eq!(pending[0].employee_id_number, "NV003");
    }

    #[test]
    fn test_parse_csv_empty_rows_skipped() {
        let csv = "Mã NV,Loại nghỉ,Ngày bắt đầu,Ngày kết thúc,Lý do,Loại thời gian\n\
                    ,,,,,\n\
                    NV001,AL,2026-03-25,2026-03-26,Test,full_day\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_parse_csv_bad_end_date() {
        let csv = "emp,type,start,end,reason,dur\n\
                    NV001,AL,2026-03-25,invalid-date,Test,full_day\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error_message.contains("Ngày kết thúc sai format"));
    }

    #[test]
    fn test_parse_csv_custom_hours_duration() {
        let csv = "emp,type,start,end,reason,dur\n\
                    NV001,AL,2026-03-25,2026-03-25,,custom_hours\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].duration_type, LeaveDurationType::CustomHours);
    }

    #[test]
    fn test_parse_csv_reason_is_optional() {
        let csv = "emp,type,start,end,reason,dur\n\
                    NV001,AL,2026-03-25,2026-03-26,,full_day\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].reason, None); // Empty reason → None
    }

    #[test]
    fn test_parse_csv_reason_with_value() {
        let csv = "emp,type,start,end,reason,dur\n\
                    NV001,AL,2026-03-25,2026-03-26,Family emergency,full_day\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].reason, Some("Family emergency".to_string()));
    }

    #[test]
    fn test_parse_csv_all_rows_invalid() {
        let csv = "emp,type,start,end,reason,dur\n\
                    ,AL,2026-03-25,2026-03-26,No emp,full_day\n\
                    NV001,,2026-03-25,2026-03-26,No type,full_day\n\
                    NV002,SL,bad-start,2026-03-26,Bad start,full_day\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_parse_csv_header_only() {
        let csv = "emp,type,start,end,reason,dur\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_parse_csv_missing_duration_defaults_to_full_day() {
        // CSV with fewer columns — duration field missing
        let csv = "emp,type,start,end,reason\n\
                    NV001,AL,2026-03-25,2026-03-26,Test\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        // Missing duration should default to FullDay
        assert_eq!(pending[0].duration_type, LeaveDurationType::FullDay);
    }

    #[test]
    fn test_parse_csv_line_numbers_correct() {
        let csv = "emp,type,start,end,reason,dur\n\
                    NV001,AL,2026-03-25,2026-03-26,Good,full_day\n\
                    ,AL,2026-03-25,2026-03-26,Bad,full_day\n\
                    NV003,SL,2026-04-01,2026-04-01,Good2,first_half\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        // Line numbers: header=1, NV001=2, empty=3, NV003=4
        assert_eq!(pending.len(), 2);
        assert_eq!(errors.len(), 1);
        assert_eq!(pending[0].line_number, 2);
        assert_eq!(errors[0].line_number, 3);
        assert_eq!(pending[1].line_number, 4);
    }

    #[test]
    fn test_parse_csv_unicode_vietnamese() {
        let csv = "emp,type,start,end,reason,dur\n\
                    NV001,AL,2026-03-25,2026-03-26,Nghỉ phép năm theo quy định,full_day\n";

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_csv(csv.as_bytes(), &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].reason,
            Some("Nghỉ phép năm theo quy định".to_string())
        );
    }

    // ---- parse_excel_range tests ----

    fn make_range(rows: Vec<Vec<Data>>) -> calamine::Range<Data> {
        if rows.is_empty() {
            return calamine::Range::new((0, 0), (0, 0));
        }
        let num_rows = rows.len() as u32;
        let num_cols = rows.first().map_or(0, |r| r.len()) as u32;
        let mut range = calamine::Range::new((0, 0), (num_rows.saturating_sub(1), num_cols.saturating_sub(1)));
        for (r, row) in rows.iter().enumerate() {
            for (c, val) in row.iter().enumerate() {
                range.set_value((r as u32, c as u32), val.clone());
            }
        }
        range
    }

    fn header_row() -> Vec<Data> {
        vec![
            Data::String("Mã NV".into()),
            Data::String("Loại nghỉ".into()),
            Data::String("Ngày bắt đầu".into()),
            Data::String("Ngày kết thúc".into()),
            Data::String("Lý do".into()),
            Data::String("Loại thời gian".into()),
        ]
    }

    #[test]
    fn test_parse_excel_range_valid_rows() {
        let range = make_range(vec![
            header_row(),
            vec![
                Data::String("NV001".into()),
                Data::String("AL".into()),
                Data::String("2026-03-25".into()),
                Data::String("2026-03-26".into()),
                Data::String("Việc gia đình".into()),
                Data::String("full_day".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(pending[0].employee_id_number, "NV001");
        assert_eq!(pending[0].leave_type_code, "AL");
        assert_eq!(pending[0].start_date, NaiveDate::from_ymd_opt(2026, 3, 25).unwrap());
        assert_eq!(pending[0].end_date, NaiveDate::from_ymd_opt(2026, 3, 26).unwrap());
        assert_eq!(pending[0].reason, Some("Việc gia đình".to_string()));
        assert_eq!(pending[0].duration_type, LeaveDurationType::FullDay);
    }

    #[test]
    fn test_parse_excel_range_missing_employee_id() {
        let range = make_range(vec![
            header_row(),
            vec![
                Data::Empty,
                Data::String("AL".into()),
                Data::String("2026-03-25".into()),
                Data::String("2026-03-25".into()),
                Data::String("test".into()),
                Data::String("full_day".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error_message.contains("Thiếu mã nhân viên"));
    }

    #[test]
    fn test_parse_excel_range_missing_leave_code() {
        let range = make_range(vec![
            header_row(),
            vec![
                Data::String("NV001".into()),
                Data::Empty,
                Data::String("2026-03-25".into()),
                Data::String("2026-03-25".into()),
                Data::String("test".into()),
                Data::String("full_day".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error_message.contains("Thiếu mã loại nghỉ"));
    }

    #[test]
    fn test_parse_excel_range_bad_start_date() {
        let range = make_range(vec![
            header_row(),
            vec![
                Data::String("NV001".into()),
                Data::String("AL".into()),
                Data::String("not-a-date".into()),
                Data::String("2026-03-25".into()),
                Data::Empty,
                Data::String("full_day".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error_message.contains("Ngày bắt đầu sai format"));
    }

    #[test]
    fn test_parse_excel_range_bad_end_date() {
        let range = make_range(vec![
            header_row(),
            vec![
                Data::String("NV001".into()),
                Data::String("AL".into()),
                Data::String("2026-03-25".into()),
                Data::String("bad".into()),
                Data::Empty,
                Data::String("full_day".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error_message.contains("Ngày kết thúc sai format"));
    }

    #[test]
    fn test_parse_excel_range_end_before_start() {
        let range = make_range(vec![
            header_row(),
            vec![
                Data::String("NV001".into()),
                Data::String("AL".into()),
                Data::String("2026-03-28".into()),
                Data::String("2026-03-25".into()), // End before start!
                Data::Empty,
                Data::String("full_day".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error_message.contains("Ngày kết thúc phải >= ngày bắt đầu"));
    }

    #[test]
    fn test_parse_excel_range_empty_rows_skipped() {
        let range = make_range(vec![
            header_row(),
            vec![Data::Empty, Data::Empty, Data::Empty, Data::Empty, Data::Empty, Data::Empty],
            vec![
                Data::String("NV001".into()),
                Data::String("AL".into()),
                Data::String("2026-03-25".into()),
                Data::String("2026-03-26".into()),
                Data::Empty,
                Data::String("full_day".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_parse_excel_range_numeric_employee_id() {
        // Excel sometimes stores IDs as numbers
        let range = make_range(vec![
            header_row(),
            vec![
                Data::Float(1001.0), // Numeric emp ID
                Data::String("SL".into()),
                Data::String("2026-04-01".into()),
                Data::String("2026-04-01".into()),
                Data::String("Sick".into()),
                Data::String("first_half".into()),
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].employee_id_number, "1001"); // Float 1001.0 → "1001"
        assert_eq!(pending[0].duration_type, LeaveDurationType::FirstHalf);
    }

    #[test]
    fn test_parse_excel_range_multiple_valid_and_invalid() {
        let range = make_range(vec![
            header_row(),
            // Valid
            vec![
                Data::String("NV001".into()),
                Data::String("AL".into()),
                Data::String("2026-03-25".into()),
                Data::String("2026-03-26".into()),
                Data::String("OK".into()),
                Data::String("full_day".into()),
            ],
            // Error: missing emp
            vec![
                Data::Empty,
                Data::String("AL".into()),
                Data::String("2026-03-25".into()),
                Data::String("2026-03-25".into()),
                Data::Empty,
                Data::Empty,
            ],
            // Valid
            vec![
                Data::String("NV003".into()),
                Data::String("SL".into()),
                Data::String("2026-04-10".into()),
                Data::String("2026-04-10".into()),
                Data::Empty,
                Data::String("second_half".into()),
            ],
            // Error: bad date
            vec![
                Data::String("NV004".into()),
                Data::String("AL".into()),
                Data::String("xxx".into()),
                Data::String("2026-04-10".into()),
                Data::Empty,
                Data::Empty,
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 2); // NV001 + NV003
        assert_eq!(errors.len(), 2);  // missing emp + bad date
        assert_eq!(pending[0].employee_id_number, "NV001");
        assert_eq!(pending[1].employee_id_number, "NV003");
        assert_eq!(pending[1].reason, None); // Empty reason → None
        assert_eq!(pending[1].duration_type, LeaveDurationType::SecondHalf);
    }

    #[test]
    fn test_parse_excel_range_header_only() {
        let range = make_range(vec![header_row()]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_parse_excel_range_line_numbers() {
        let range = make_range(vec![
            header_row(), // idx=0 → skip
            // idx=1 → line_number = 2
            vec![
                Data::String("NV001".into()),
                Data::String("AL".into()),
                Data::String("2026-03-25".into()),
                Data::String("2026-03-26".into()),
                Data::Empty,
                Data::Empty,
            ],
            // idx=2 → line_number = 3 (error)
            vec![
                Data::Empty,
                Data::String("SL".into()),
                Data::String("2026-04-01".into()),
                Data::String("2026-04-01".into()),
                Data::Empty,
                Data::Empty,
            ],
        ]);

        let mut pending = Vec::new();
        let mut errors = Vec::new();
        LeaveImportService::parse_excel_range(range, &mut pending, &mut errors);

        assert_eq!(pending[0].line_number, 2);
        assert_eq!(errors[0].line_number, 3);
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

