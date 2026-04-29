//! Shared i18n message definitions for import services.
//!
//! Provides language-keyed error messages used by both `LeaveImportService`
//! and `AttendanceImportService`, eliminating hardcoded Vietnamese strings.

/// Localized messages for leave import operations (parse + error Excel).
pub struct LeaveImportMessages {
    // Parse-level validation errors
    pub missing_employee_id: &'static str,
    pub missing_leave_type: &'static str,
    pub invalid_start_date: &'static str,
    pub invalid_end_date: &'static str,
    pub end_before_start: &'static str,
    pub missing_employee_or_leave: &'static str,
    pub csv_read_error_prefix: &'static str,
    pub submission_error_prefix: &'static str,

    // process_import-level errors (already partially i18n'd)
    pub file_empty: &'static str,
    pub employee_not_found_tpl: &'static str,
    pub leave_type_not_found_tpl: &'static str,

    // Error Excel output
    pub error_sheet_name: &'static str,
    pub error_headers: [&'static str; 8],

    // Template Excel output
    pub template_sheet_name: &'static str,
    pub template_ref_sheet_name: &'static str,
    pub template_headers: [&'static str; 6],
    pub template_example_reason_1: &'static str,
    pub template_example_reason_2: &'static str,
    pub ref_header_code: &'static str,
    pub ref_header_name: &'static str,
    pub ref_header_duration: &'static str,
    pub ref_header_desc: &'static str,
    pub desc_full_day: &'static str,
    pub desc_morning: &'static str,
    pub desc_afternoon: &'static str,
}

impl LeaveImportMessages {
    /// Return messages for the given language code.
    pub fn for_lang(lang: &str) -> Self {
        match lang {
            "vi" => Self::vi(),
            "zh" => Self::zh(),
            _ => Self::en(),
        }
    }

    fn en() -> Self {
        Self {
            missing_employee_id: "Missing employee ID",
            missing_leave_type: "Missing leave type code",
            invalid_start_date: "Invalid start date format (expected YYYY-MM-DD)",
            invalid_end_date: "Invalid end date format (expected YYYY-MM-DD)",
            end_before_start: "End date must be >= start date",
            missing_employee_or_leave: "Missing employee ID or leave type code",
            csv_read_error_prefix: "CSV row read error",
            submission_error_prefix: "Submission error",
            file_empty: "File is empty or contains no valid data",
            employee_not_found_tpl: "Employee ID '{}' does not exist",
            leave_type_not_found_tpl: "Leave type '{}' does not exist",
            error_sheet_name: "Error Rows",
            error_headers: [
                "Row", "Emp ID", "Leave Type", "Start Date",
                "End Date", "Reason", "Duration Type", "❌ Error",
            ],
            template_sheet_name: "Entry Template",
            template_ref_sheet_name: "Leave Types List",
            template_headers: [
                "Emp ID", "Leave Type (code)", "Start Date",
                "End Date", "Reason", "Duration Type",
            ],
            template_example_reason_1: "Family matter",
            template_example_reason_2: "Doctor appointment",
            ref_header_code: "Code",
            ref_header_name: "Leave Type Name",
            ref_header_duration: "Duration Type",
            ref_header_desc: "Description",
            desc_full_day: "Full day (default)",
            desc_morning: "Morning half",
            desc_afternoon: "Afternoon half",
        }
    }

    fn vi() -> Self {
        Self {
            missing_employee_id: "Thiếu mã nhân viên",
            missing_leave_type: "Thiếu mã loại nghỉ",
            invalid_start_date: "Ngày bắt đầu sai format (yêu cầu YYYY-MM-DD)",
            invalid_end_date: "Ngày kết thúc sai format (yêu cầu YYYY-MM-DD)",
            end_before_start: "Ngày kết thúc phải >= ngày bắt đầu",
            missing_employee_or_leave: "Thiếu mã nhân viên hoặc mã loại nghỉ",
            csv_read_error_prefix: "Lỗi đọc dòng CSV",
            submission_error_prefix: "Lỗi khi tạo đơn",
            file_empty: "File rỗng hoặc không có dữ liệu hợp lệ",
            employee_not_found_tpl: "Mã nhân viên '{}' không tồn tại",
            leave_type_not_found_tpl: "Loại nghỉ '{}' không tồn tại",
            error_sheet_name: "Dòng lỗi",
            error_headers: [
                "Dòng", "Mã NV", "Loại nghỉ", "Ngày bắt đầu",
                "Ngày kết thúc", "Lý do", "Loại thời gian", "❌ Lỗi",
            ],
            template_sheet_name: "Mẫu nhập",
            template_ref_sheet_name: "Danh sách loại nghỉ",
            template_headers: [
                "Mã NV", "Loại nghỉ (code)", "Ngày bắt đầu",
                "Ngày kết thúc", "Lý do", "Loại thời gian",
            ],
            template_example_reason_1: "Việc gia đình",
            template_example_reason_2: "Khám bệnh",
            ref_header_code: "Code",
            ref_header_name: "Tên loại nghỉ",
            ref_header_duration: "Loại thời gian",
            ref_header_desc: "Mô tả",
            desc_full_day: "Cả ngày (mặc định)",
            desc_morning: "Nửa ngày sáng",
            desc_afternoon: "Nửa ngày chiều",
        }
    }

    fn zh() -> Self {
        Self {
            missing_employee_id: "缺少员工编号",
            missing_leave_type: "缺少假期类型代码",
            invalid_start_date: "开始日期格式错误 (要求 YYYY-MM-DD)",
            invalid_end_date: "结束日期格式错误 (要求 YYYY-MM-DD)",
            end_before_start: "结束日期必须 >= 开始日期",
            missing_employee_or_leave: "缺少员工编号或假期类型代码",
            csv_read_error_prefix: "CSV行读取错误",
            submission_error_prefix: "提交错误",
            file_empty: "文件为空或没有有效数据",
            employee_not_found_tpl: "员工编号 '{}' 不存在",
            leave_type_not_found_tpl: "假期类型 '{}' 不存在",
            error_sheet_name: "错误行",
            error_headers: [
                "行号", "员工编号", "假期类型", "开始日期",
                "结束日期", "原因", "时长类型", "❌ 错误",
            ],
            template_sheet_name: "输入模板",
            template_ref_sheet_name: "假期类型列表",
            template_headers: [
                "员工编号", "假期类型(code)", "开始日期",
                "结束日期", "原因", "时长类型",
            ],
            template_example_reason_1: "家庭事务",
            template_example_reason_2: "看病",
            ref_header_code: "代码",
            ref_header_name: "假期类型名称",
            ref_header_duration: "时长类型",
            ref_header_desc: "描述",
            desc_full_day: "全天(默认)",
            desc_morning: "上午半天",
            desc_afternoon: "下午半天",
        }
    }
}

/// Localized messages for attendance import operations.
pub struct AttendanceImportMessages {
    pub missing_employee_or_time: &'static str,
    pub missing_employee_id: &'static str,
    pub missing_employee_name: &'static str,
    pub missing_employee_or_id: &'static str,
    pub missing_time_data: &'static str,
    pub invalid_datetime: &'static str,
    pub system_save_error: &'static str,
    pub db_save_error: &'static str,
    pub skipped_employees: &'static str,
    pub skipped_rows: &'static str,
    pub skipped_unknown_employees: &'static str,
}

impl AttendanceImportMessages {
    /// Return messages for the given language code.
    pub fn for_lang(lang: &str) -> Self {
        match lang {
            "vi" => Self::vi(),
            "zh" => Self::zh(),
            _ => Self::en(),
        }
    }

    fn en() -> Self {
        Self {
            missing_employee_or_time: "Row {line}: Missing employee ID or timestamp",
            missing_employee_id: "Row {line}: Missing employee ID",
            missing_employee_name: "Row {line}: Missing employee name",
            missing_employee_or_id: "Row {line}: Missing employee ID or name",
            missing_time_data: "Row {line} (Emp {emp}): Missing timestamp data",
            invalid_datetime: "Row {line} (Emp {emp}): Not a valid date/time format [Raw: '{raw}']",
            system_save_error: "Emp {emp}: System error while saving (Row {line})",
            db_save_error: "Row {line} (ID {emp}): Database save error: {error}",
            skipped_employees: "Skipped {count} employees with duplicate IDs.",
            skipped_rows: "Skipped {count} rows (duplicate or employee not found).",
            skipped_unknown_employees: "Skipped {count} unknown employee IDs: {ids}",
        }
    }

    fn vi() -> Self {
        Self {
            missing_employee_or_time: "Dòng {line}: Thiếu mã nhân viên hoặc thời gian",
            missing_employee_id: "Dòng {line}: Thiếu mã nhân viên",
            missing_employee_name: "Dòng {line}: Thiếu Tên nhân viên",
            missing_employee_or_id: "Dòng {line}: Thiếu ID hoặc Tên nhân viên",
            missing_time_data: "Dòng {line} (NV {emp}): Thiếu dữ liệu thời gian",
            invalid_datetime: "Dòng {line} (NV {emp}): Không phải định dạng ngày giờ hợp lệ [Dữ liệu gốc: '{raw}']",
            system_save_error: "NV {emp}: Lỗi hệ thống khi lưu (Dòng {line})",
            db_save_error: "Dòng {line} (ID {emp}): Lỗi khi lưu vào DB: {error}",
            skipped_employees: "Đã bỏ qua {count} nhân viên trùng mã.",
            skipped_rows: "Đã bỏ qua {count} dòng (trùng lặp hoặc NV không tồn tại).",
            skipped_unknown_employees: "Bỏ qua {count} mã nhân viên không tồn tại: {ids}",
        }
    }

    fn zh() -> Self {
        Self {
            missing_employee_or_time: "第{line}行: 缺少员工编号或时间",
            missing_employee_id: "第{line}行: 缺少员工编号",
            missing_employee_name: "第{line}行: 缺少员工姓名",
            missing_employee_or_id: "第{line}行: 缺少员工编号或姓名",
            missing_time_data: "第{line}行 (员工 {emp}): 缺少时间数据",
            invalid_datetime: "第{line}行 (员工 {emp}): 无效的日期时间格式 [原始数据: '{raw}']",
            system_save_error: "员工 {emp}: 保存时系统错误 (第{line}行)",
            db_save_error: "第{line}行 (编号 {emp}): 数据库保存错误: {error}",
            skipped_employees: "已跳过 {count} 个重复员工编号。",
            skipped_rows: "已跳过 {count} 行（重复或员工不存在）。",
            skipped_unknown_employees: "已跳过 {count} 个不存在的员工编号: {ids}",
        }
    }
}

/// Simple template-style string replacement for message formatting.
/// Replaces `{key}` placeholders with provided values.
pub fn fmt_msg(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in replacements {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_msg_single() {
        assert_eq!(
            fmt_msg("Hello {name}!", &[("name", "World")]),
            "Hello World!"
        );
    }

    #[test]
    fn test_fmt_msg_multiple() {
        assert_eq!(
            fmt_msg("Row {line} (Emp {emp}): Error", &[("line", "5"), ("emp", "NV001")]),
            "Row 5 (Emp NV001): Error"
        );
    }

    #[test]
    fn test_fmt_msg_no_match() {
        assert_eq!(fmt_msg("No placeholders here", &[("x", "y")]), "No placeholders here");
    }

    #[test]
    fn test_leave_messages_en() {
        let msgs = LeaveImportMessages::for_lang("en");
        assert_eq!(msgs.missing_employee_id, "Missing employee ID");
        assert_eq!(msgs.error_sheet_name, "Error Rows");
    }

    #[test]
    fn test_leave_messages_vi() {
        let msgs = LeaveImportMessages::for_lang("vi");
        assert_eq!(msgs.missing_employee_id, "Thiếu mã nhân viên");
        assert_eq!(msgs.error_sheet_name, "Dòng lỗi");
    }

    #[test]
    fn test_leave_messages_zh() {
        let msgs = LeaveImportMessages::for_lang("zh");
        assert_eq!(msgs.missing_employee_id, "缺少员工编号");
        assert_eq!(msgs.error_sheet_name, "错误行");
    }

    #[test]
    fn test_leave_messages_unknown_lang_defaults_to_en() {
        let msgs = LeaveImportMessages::for_lang("ja");
        assert_eq!(msgs.file_empty, "File is empty or contains no valid data");
    }

    #[test]
    fn test_attendance_messages_en() {
        let msgs = AttendanceImportMessages::for_lang("en");
        let result = fmt_msg(msgs.missing_employee_or_time, &[("line", "3")]);
        assert_eq!(result, "Row 3: Missing employee ID or timestamp");
    }

    #[test]
    fn test_attendance_messages_vi() {
        let msgs = AttendanceImportMessages::for_lang("vi");
        let result = fmt_msg(msgs.system_save_error, &[("emp", "NV001"), ("line", "5")]);
        assert_eq!(result, "NV NV001: Lỗi hệ thống khi lưu (Dòng 5)");
    }

    #[test]
    fn test_attendance_messages_zh() {
        let msgs = AttendanceImportMessages::for_lang("zh");
        let result = fmt_msg(msgs.missing_employee_id, &[("line", "7")]);
        assert_eq!(result, "第7行: 缺少员工编号");
    }
}
