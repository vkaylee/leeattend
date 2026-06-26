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
                "Row",
                "Emp ID",
                "Leave Type",
                "Start Date",
                "End Date",
                "Reason",
                "Duration Type",
                "❌ Error",
            ],
            template_sheet_name: "Entry Template",
            template_ref_sheet_name: "Leave Types List",
            template_headers: [
                "Emp ID",
                "Leave Type (code)",
                "Start Date",
                "End Date",
                "Reason",
                "Duration Type",
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
                "Dòng",
                "Mã NV",
                "Loại nghỉ",
                "Ngày bắt đầu",
                "Ngày kết thúc",
                "Lý do",
                "Loại thời gian",
                "❌ Lỗi",
            ],
            template_sheet_name: "Mẫu nhập",
            template_ref_sheet_name: "Danh sách loại nghỉ",
            template_headers: [
                "Mã NV",
                "Loại nghỉ (code)",
                "Ngày bắt đầu",
                "Ngày kết thúc",
                "Lý do",
                "Loại thời gian",
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
                "行号",
                "员工编号",
                "假期类型",
                "开始日期",
                "结束日期",
                "原因",
                "时长类型",
                "❌ 错误",
            ],
            template_sheet_name: "输入模板",
            template_ref_sheet_name: "假期类型列表",
            template_headers: [
                "员工编号",
                "假期类型(code)",
                "开始日期",
                "结束日期",
                "原因",
                "时长类型",
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
#[derive(Clone)]
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

/// Localized labels for Excel report output (Summary + Detail sheets).
#[derive(Debug, Clone, Copy)]
pub struct ExcelReportMessages {
    // Sheet names
    pub summary_sheet_name: &'static str,
    pub timesheet_sheet_name: &'static str,
    pub raw_details_sheet_name: &'static str,
    pub guide_sheet_name: &'static str,

    // Title templates (use {workspace} placeholder)
    pub summary_title_tpl: &'static str,
    pub timesheet_title_tpl: &'static str,
    pub raw_details_title_tpl: &'static str,
    pub period_label: &'static str,

    // Summary headers
    pub summary_headers: [&'static str; 5],

    // Timesheet matrix headers
    pub timesheet_emp_id_header: &'static str,
    pub timesheet_emp_header: &'static str,
    pub timesheet_employer_header: &'static str,

    // Raw details headers
    pub detail_headers: [&'static str; 15],

    // Guide page translations
    pub guide_title: &'static str,
    pub guide_headers: [&'static str; 3],
    pub guide_explanation_title: &'static str,
    pub guide_explanation_format: &'static str,
    pub guide_explanation_example: &'static str,
    pub guide_abbreviations: [[&'static str; 3]; 5],

    // Status translations
    pub status_present: &'static str,
    pub status_late: &'static str,
    pub status_absent: &'static str,
    pub status_off: &'static str,
    pub status_holiday: &'static str,
    pub status_holiday_work: &'static str,
    pub status_leave_paid: &'static str,
    pub status_leave_unpaid: &'static str,
    pub status_unscheduled: &'static str,

    // OT status translations
    pub ot_status_pending_employee: &'static str,
    pub ot_status_pending_manager: &'static str,
    pub ot_status_approved: &'static str,
    pub ot_status_rejected: &'static str,
}

impl ExcelReportMessages {
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
            summary_sheet_name: "Summary",
            timesheet_sheet_name: "Timesheet View",
            raw_details_sheet_name: "Raw Details",
            guide_sheet_name: "Guide",
            summary_title_tpl: "Attendance Summary Report - {workspace}",
            timesheet_title_tpl: "Timesheet View - {workspace}",
            raw_details_title_tpl: "Raw Details - {workspace}",
            period_label: "Period: {start} to {end}",
            summary_headers: ["Employee Name", "Employer", "Std Hours", "OT Hours", "Total Leave Days"],
            timesheet_emp_id_header: "Employee ID",
            timesheet_emp_header: "Employee",
            timesheet_employer_header: "Employer",
            detail_headers: [
                "Date",
                "Employee ID",
                "Employee Name",
                "Employer",
                "Segment Type",
                "Interval",
                "Scheduled Start",
                "Scheduled End",
                "Actual In",
                "Actual Out",
                "Std Hours",
                "OT Hours",
                "OT Status",
                "Status",
                "Holiday",
            ],
            guide_title: "Attendance Timesheet Legend & Guide",
            guide_headers: ["Symbol / Format", "Meaning", "Description"],
            guide_explanation_title: "Timesheet Cell Format",
            guide_explanation_format: "[Shift Symbol][Std Hours]/[Late Hours]/[Early Leave Hours]/[OT Hours]",
            guide_explanation_example: "Example: N8/0.1/0/2 means Shift N, 8h standard, 6m late, 0h early leave, 2h overtime",
            guide_abbreviations: [
                ["(PL)", "Paid Leave", "Approved paid leave days"],
                ["(UL)", "Unpaid Leave", "Approved unpaid leave days"],
                ["(HP)", "Holiday (Paid)", "Paid public holidays, employee did not work"],
                ["(HU)", "Holiday (Unpaid)", "Unpaid public holidays, employee did not work"],
                ["(HW)", "Holiday Work", "Employee worked on a public holiday (followed by work hours details)"],
            ],
            status_present: "Present",
            status_late: "Late",
            status_absent: "Absent",
            status_off: "Off",
            status_holiday: "Holiday (Paid)",
            status_holiday_work: "Holiday Work",
            status_leave_paid: "Paid Leave",
            status_leave_unpaid: "Unpaid Leave",
            status_unscheduled: "Unscheduled",
            ot_status_pending_employee: "Pending Emp",
            ot_status_pending_manager: "Pending Mgr",
            ot_status_approved: "Approved",
            ot_status_rejected: "Rejected",
        }
    }

    fn vi() -> Self {
        Self {
            summary_sheet_name: "Tổng hợp",
            timesheet_sheet_name: "Bảng chấm công",
            raw_details_sheet_name: "Chi tiết",
            guide_sheet_name: "Hướng dẫn",
            summary_title_tpl: "Báo cáo tổng hợp chấm công - {workspace}",
            timesheet_title_tpl: "Bảng chấm công - {workspace}",
            raw_details_title_tpl: "Chi tiết chấm công - {workspace}",
            period_label: "Kỳ: {start} đến {end}",
            summary_headers: [
                "Tên nhân viên",
                "Công ty",
                "Giờ công chuẩn",
                "Giờ tăng ca",
                "Tổng ngày nghỉ",
            ],
            timesheet_emp_id_header: "Mã NV",
            timesheet_emp_header: "Nhân viên",
            timesheet_employer_header: "Công ty",
            detail_headers: [
                "Ngày",
                "Mã NV",
                "Tên nhân viên",
                "Công ty",
                "Loại phân đoạn",
                "Tên khung giờ",
                "Giờ bắt đầu",
                "Giờ kết thúc",
                "Giờ vào thực tế",
                "Giờ ra thực tế",
                "Giờ chuẩn",
                "Giờ tăng ca",
                "Trạng thái TC",
                "Trạng thái",
                "Ngày lễ",
            ],
            guide_title: "Hướng dẫn & Chú giải Bảng chấm công",
            guide_headers: ["Ký hiệu / Định dạng", "Ý nghĩa", "Mô tả chi tiết"],
            guide_explanation_title: "Định dạng ô chấm công",
            guide_explanation_format: "[Mã ca][Giờ chuẩn]/[Giờ đi trễ]/[Giờ về sớm]/[Giờ tăng ca]",
            guide_explanation_example: "Ví dụ: N8/0.1/0/2 có nghĩa là Ca N, 8h chuẩn, đi trễ 0.1h (6 phút), không về sớm, 2h tăng ca",
            guide_abbreviations: [
                ["(PL)", "Phép năm / Nghỉ có lương", "Ngày nghỉ phép đã được phê duyệt hưởng lương"],
                ["(UL)", "Nghỉ không lương", "Nghỉ không lương đã được phê duyệt"],
                ["(HP)", "Nghỉ lễ có lương", "Ngày lễ được hưởng lương, nhân viên không đi làm"],
                ["(HU)", "Nghỉ lễ không lương", "Ngày lễ không hưởng lương, nhân viên không đi làm"],
                ["(HW)", "Đi làm ngày lễ", "Nhân viên đi làm vào ngày lễ (kèm theo chi tiết số giờ làm việc)"],
            ],
            status_present: "Đúng giờ",
            status_late: "Đi muộn",
            status_absent: "Vắng mặt",
            status_off: "Ngày nghỉ",
            status_holiday: "Nghỉ lễ có lương",
            status_holiday_work: "Làm ngày lễ",
            status_leave_paid: "Nghỉ phép có lương",
            status_leave_unpaid: "Nghỉ không lương",
            status_unscheduled: "Ngoài lịch",
            ot_status_pending_employee: "Chờ NV",
            ot_status_pending_manager: "Chờ duyệt",
            ot_status_approved: "Đã duyệt",
            ot_status_rejected: "Từ chối",
        }
    }

    fn zh() -> Self {
        Self {
            summary_sheet_name: "汇总",
            timesheet_sheet_name: "考勤表",
            raw_details_sheet_name: "明细",
            guide_sheet_name: "使用说明",
            summary_title_tpl: "考勤汇总报表 - {workspace}",
            timesheet_title_tpl: "考勤表 - {workspace}",
            raw_details_title_tpl: "考勤明细 - {workspace}",
            period_label: "期间: {start} 至 {end}",
            summary_headers: ["员工姓名", "雇主", "标准工时", "加班工时", "请假天数"],
            timesheet_emp_id_header: "员工编号",
            timesheet_emp_header: "员工",
            timesheet_employer_header: "雇主",
            detail_headers: [
                "日期",
                "员工编号",
                "员工姓名",
                "雇主",
                "分段类型",
                "时段名称",
                "排班开始",
                "排班结束",
                "实际签到",
                "实际签退",
                "标准工时",
                "加班工时",
                "加班状态",
                "状态",
                "节假日",
            ],
            guide_title: "考勤表图例与使用说明",
            guide_headers: ["符号 / 格式", "含义", "详细说明"],
            guide_explanation_title: "考勤单元格格式",
            guide_explanation_format: "[班次符号][标准工时]/[迟到工时]/[早退工时]/[加班工时]",
            guide_explanation_example:
                "示例: N8/0.1/0/2 表示班次N, 8小时标准工时, 迟到0.1小时(6分钟), 无早退, 2小时加班",
            guide_abbreviations: [
                ["(PL)", "有薪假", "已批准的有薪假期"],
                ["(UL)", "无薪假", "已批准的无薪假期"],
                ["(HP)", "法定节假日(带薪)", "带薪法定节假日，员工未出勤"],
                ["(HU)", "法定节假日(无薪)", "无薪法定节假日，员工未出勤"],
                [
                    "(HW)",
                    "节假日出勤",
                    "员工在法定节假日出勤（后跟具体工时明细）",
                ],
            ],
            status_present: "准时",
            status_late: "迟到",
            status_absent: "缺勤",
            status_off: "休息日",
            status_holiday: "法定节假日(带薪)",
            status_holiday_work: "节假日出勤",
            status_leave_paid: "有薪假",
            status_leave_unpaid: "无薪假",
            status_unscheduled: "未排班",
            ot_status_pending_employee: "待员工确认",
            ot_status_pending_manager: "待经理审批",
            ot_status_approved: "已批准",
            ot_status_rejected: "已拒绝",
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

/// Localized messages for termination import operations.
pub struct TerminationImportMessages {
    pub template_sheet_name: &'static str,
    pub instructions_sheet_name: &'static str,
    pub title_instructions: &'static str,
    pub header_column: &'static str,
    pub header_required: &'static str,
    pub header_desc: &'static str,
    pub col_employee_code_desc: &'static str,
    pub col_termination_date_desc: &'static str,
    pub col_termination_reason_desc: &'static str,
    pub col_termination_notes_desc: &'static str,
    pub title_valid_reasons: &'static str,
    pub reason_header_key: &'static str,
    pub reason_header_desc: &'static str,
    pub yes_label: &'static str,
    pub no_label: &'static str,
    pub example_notes: &'static str,
    pub reason_voluntary: &'static str,
    pub reason_involuntary: &'static str,
    pub reason_end_of_contract: &'static str,
    pub reason_retirement: &'static str,
    pub reason_mutual_agreement: &'static str,
    pub reason_abandonment: &'static str,
    pub reason_death: &'static str,
    pub reason_other: &'static str,
}

impl TerminationImportMessages {
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
            template_sheet_name: "Template",
            instructions_sheet_name: "Instructions",
            title_instructions: "Instructions & Valid Reasons",
            header_column: "Column",
            header_required: "Required?",
            header_desc: "Description",
            col_employee_code_desc: "Unique employee identifier code",
            col_termination_date_desc: "Last day of work (YYYY-MM-DD or DD/MM/YYYY)",
            col_termination_reason_desc: "Reason code (Must select from table below)",
            col_termination_notes_desc: "Optional comments/notes",
            title_valid_reasons: "Valid Reason Keys",
            reason_header_key: "termination_reason Value",
            reason_header_desc: "Description",
            yes_label: "Yes",
            no_label: "No",
            example_notes: "Relocation to another city",
            reason_voluntary: "Voluntary Resignation",
            reason_involuntary: "Involuntary Termination",
            reason_end_of_contract: "End of Contract",
            reason_retirement: "Retirement",
            reason_mutual_agreement: "Mutual Agreement",
            reason_abandonment: "Job Abandonment",
            reason_death: "Death",
            reason_other: "Other",
        }
    }

    fn vi() -> Self {
        Self {
            template_sheet_name: "Mẫu nhập",
            instructions_sheet_name: "Hướng dẫn",
            title_instructions: "Hướng dẫn & Lý do hợp lệ",
            header_column: "Cột",
            header_required: "Bắt buộc?",
            header_desc: "Mô tả",
            col_employee_code_desc: "Mã số nhân viên duy nhất",
            col_termination_date_desc:
                "Ngày làm việc cuối cùng (định dạng YYYY-MM-DD hoặc DD/MM/YYYY)",
            col_termination_reason_desc:
                "Mã lý do nghỉ việc (Phải nhập chính xác từ bảng bên dưới)",
            col_termination_notes_desc: "Ghi chú thêm (không bắt buộc)",
            title_valid_reasons: "Mã lý do hợp lệ",
            reason_header_key: "Giá trị của termination_reason",
            reason_header_desc: "Mô tả lý do",
            yes_label: "Có",
            no_label: "Không",
            example_notes: "Chuyển nhà sang thành phố khác",
            reason_voluntary: "Nghỉ tự nguyện",
            reason_involuntary: "Sa thải",
            reason_end_of_contract: "Hết hạn hợp đồng",
            reason_retirement: "Nghỉ hưu",
            reason_mutual_agreement: "Thỏa thuận chấm dứt",
            reason_abandonment: "Bỏ việc không phép",
            reason_death: "Từ trần",
            reason_other: "Lý do khác",
        }
    }

    fn zh() -> Self {
        Self {
            template_sheet_name: "输入模板",
            instructions_sheet_name: "使用说明",
            title_instructions: "使用说明与有效离职原因",
            header_column: "列名",
            header_required: "是否必填?",
            header_desc: "说明",
            col_employee_code_desc: "唯一的员工编号",
            col_termination_date_desc: "最后工作日 (YYYY-MM-DD 或 DD/MM/YYYY)",
            col_termination_reason_desc: "离职原因代码 (必须从下方表格选择)",
            col_termination_notes_desc: "可选备注/说明",
            title_valid_reasons: "有效离职原因代码",
            reason_header_key: "termination_reason 值",
            reason_header_desc: "原因描述",
            yes_label: "是",
            no_label: "否",
            example_notes: "搬迁至其他城市",
            reason_voluntary: "自愿辞职",
            reason_involuntary: "解雇",
            reason_end_of_contract: "合同到期",
            reason_retirement: "退休",
            reason_mutual_agreement: "协商一致",
            reason_abandonment: "擅自离职",
            reason_death: "因公/因病去世",
            reason_other: "其他",
        }
    }
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
            fmt_msg(
                "Row {line} (Emp {emp}): Error",
                &[("line", "5"), ("emp", "NV001")]
            ),
            "Row 5 (Emp NV001): Error"
        );
    }

    #[test]
    fn test_fmt_msg_no_match() {
        assert_eq!(
            fmt_msg("No placeholders here", &[("x", "y")]),
            "No placeholders here"
        );
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

    // ── ExcelReportMessages tests ──────────────────────────────────────────

    #[test]
    fn test_excel_report_messages_en() {
        let msgs = ExcelReportMessages::for_lang("en");
        assert_eq!(msgs.summary_sheet_name, "Summary");
        assert_eq!(msgs.timesheet_sheet_name, "Timesheet View");
        assert_eq!(msgs.raw_details_sheet_name, "Raw Details");
        assert_eq!(msgs.summary_headers[0], "Employee Name");
        assert_eq!(msgs.detail_headers[0], "Date");
        assert_eq!(msgs.detail_headers.len(), 15);
    }

    #[test]
    fn test_excel_report_messages_vi() {
        let msgs = ExcelReportMessages::for_lang("vi");
        assert_eq!(msgs.summary_sheet_name, "Tổng hợp");
        assert_eq!(msgs.timesheet_sheet_name, "Bảng chấm công");
        assert_eq!(msgs.summary_headers[0], "Tên nhân viên");
        assert_eq!(msgs.timesheet_emp_header, "Nhân viên");
    }

    #[test]
    fn test_excel_report_messages_zh() {
        let msgs = ExcelReportMessages::for_lang("zh");
        assert_eq!(msgs.summary_sheet_name, "汇总");
        assert_eq!(msgs.timesheet_sheet_name, "考勤表");
        assert_eq!(msgs.summary_headers[0], "员工姓名");
        assert_eq!(msgs.timesheet_emp_header, "员工");
    }

    #[test]
    fn test_excel_report_messages_unknown_defaults_to_en() {
        let msgs = ExcelReportMessages::for_lang("ja");
        assert_eq!(msgs.summary_sheet_name, "Summary");
        assert_eq!(msgs.timesheet_emp_id_header, "Employee ID");
    }

    #[test]
    fn test_excel_report_messages_title_template() {
        let msgs = ExcelReportMessages::for_lang("vi");
        let title = msgs.summary_title_tpl.replace("{workspace}", "Công ty ABC");
        assert_eq!(title, "Báo cáo tổng hợp chấm công - Công ty ABC");
    }

    #[test]
    fn test_excel_report_messages_period_template() {
        let msgs = ExcelReportMessages::for_lang("en");
        let period = fmt_msg(
            &msgs
                .period_label
                .replace("{start}", "2026-01-01")
                .replace("{end}", "2026-01-31"),
            &[],
        );
        assert_eq!(period, "Period: 2026-01-01 to 2026-01-31");
    }

    #[test]
    fn test_termination_import_messages_vi() {
        let msgs = TerminationImportMessages::for_lang("vi");
        assert_eq!(msgs.template_sheet_name, "Mẫu nhập");
        assert_eq!(msgs.reason_voluntary, "Nghỉ tự nguyện");
    }

    #[test]
    fn test_termination_import_messages_en() {
        let msgs = TerminationImportMessages::for_lang("en");
        assert_eq!(msgs.template_sheet_name, "Template");
        assert_eq!(msgs.reason_voluntary, "Voluntary Resignation");
    }
}
