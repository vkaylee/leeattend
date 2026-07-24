//! Backend internationalization (i18n) utility module.
//!
//! Provides context propagation using tokio task-local storage and type-safe
//! translation lookups for English, Vietnamese, and Chinese.

use std::str::FromStr;

// ---------------------------------------------------------------------------
// Source-of-truth macro — all five representations generated from one table
// ---------------------------------------------------------------------------

macro_rules! i18n_table {
    ($(
        $variant:ident => $key:literal => ($en:expr, $vi:expr, $zh:expr)
    ),* $(,)?) => {
        /// Every registered i18n key, in declaration order.
        pub const ALL_KEYS: &[&str] = &[$($key),*];

        /// Type-safe enumeration of every translatable key.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum I18nKey { $($variant),* }

        impl I18nKey {
            /// Map each variant back to its canonical `"error.xxx"` string.
            #[allow(clippy::too_many_lines)]
            pub fn as_str(&self) -> &'static str {
                match self { $(Self::$variant => $key),* }
            }
        }

        impl std::str::FromStr for I18nKey {
            type Err = ();
            #[allow(clippy::too_many_lines)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s { $($key => Ok(Self::$variant)),* , _ => Err(()) }
            }
        }

        #[allow(clippy::too_many_lines, clippy::match_same_arms)]
        fn lookup_en(key: &str) -> Option<&'static str> {
            match key { $($key => Some($en)),* , _ => None }
        }
        #[allow(clippy::too_many_lines, clippy::match_same_arms)]
        fn lookup_vi(key: &str) -> Option<&'static str> {
            match key { $($key => Some($vi)),* , _ => None }
        }
        #[allow(clippy::too_many_lines, clippy::match_same_arms)]
        fn lookup_zh(key: &str) -> Option<&'static str> {
            match key { $($key => Some($zh)),* , _ => None }
        }
    };
}

i18n_table! {
    Unauthorized => "error.unauthorized" => ("Authentication required", "Yêu cầu đăng nhập", "需要身份验证"),
    AuthenticationRequired => "error.authentication_required" => ("Authentication required", "Yêu cầu đăng nhập", "需要身份验证"),
    Forbidden => "error.forbidden" => ("Permission denied", "Không có quyền truy cập", "权限拒绝"),
    NotFound => "error.not_found" => ("Resource not found", "Không tìm thấy tài nguyên", "未找到资源"),
    Validation => "error.validation" => ("Validation failed", "Lỗi kiểm tra dữ liệu", "数据验证失败"),
    Internal => "error.internal" => ("An internal server error occurred.", "Đã xảy ra lỗi hệ thống.", "发生系统内部错误。"),
    Conflict => "error.conflict" => ("A conflict occurred.", "Đã xảy ra xung đột dữ liệu.", "数据冲突。"),
    ServiceUnavailable => "error.service_unavailable" => ("Service unavailable.", "Dịch vụ hiện không khả dụng.", "服务暂不可用。"),
    MissingWorkspace => "error.missing_workspace" => ("Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header.", "Thiếu thông tin Workspace. Vui lòng truyền 'ws_id' trong URL hoặc header 'X-Workspace-Id'.", "缺少工作空间上下文。请在路径中提供 'ws_id' 或在 'X-Workspace-Id' 请求头中提供。"),
    InvalidWorkspaceId => "error.invalid_workspace_id" => ("Invalid Workspace ID format", "Định dạng ID Workspace không hợp lệ", "工作空间 ID 格式无效"),
    NoEmployeeRecord => "error.no_employee_record" => ("No employee record linked to your account", "Không có thông tin nhân viên nào liên kết với tài khoản của bạn", "您的账户未关联任何员工档案"),
    SystemRouteTenantContext => "error.system_route_tenant_context" => ("Cannot use Tenant context (MemberContext) in a System route", "Không thể sử dụng context Tenant (MemberContext) trong route hệ thống", "系统路由中无法使用租户上下文 (MemberContext)"),
    NotWorkspaceMember => "error.not_workspace_member" => ("Not a member of this workspace", "Bạn không phải là thành viên của workspace này", "您不是该工作空间的成员"),
    UserNotFound => "error.user_not_found" => ("User not found", "Không tìm thấy người dùng", "未找到该用户"),
    EmployeeNotFound => "error.employee_not_found" => ("Employee not found", "Không tìm thấy nhân viên", "未找到员工"),
    EmploymentNotFound => "error.employment_not_found" => ("Employment not found", "Không tìm thấy quan hệ công việc", "未找到工作关系"),
    DepartmentNotFound => "error.department_not_found" => ("Department not found", "Không tìm thấy phòng ban", "未找到部门"),
    DatabaseError => "error.database_error" => ("Database error", "Lỗi cơ sở dữ liệu", "数据库错误"),
    CannotRevokeLastAdmin => "error.cannot_revoke_last_admin" => ("Cannot revoke the last platform_admin. Assign another admin first.", "Không thể thu hồi platform_admin cuối cùng. Hãy gán quản trị viên khác trước.", "无法撤销最后一个 platform_admin。请先分配另一个管理员。"),
    BehalfAssignmentMissingTarget => "error.behalf_assignment_missing_target" => ("Either target employee or target department must be specified", "Phải chỉ định nhân viên đích hoặc phòng ban đích", "必须指定目标员工或目标部门"),
    BehalfAssignmentDualTarget => "error.behalf_assignment_dual_target" => ("Cannot specify both target employee and target department", "Không thể chỉ định cả nhân viên đích và phòng ban đích", "不能同时指定目标员工和目标部门"),
    UploadMultipart => "error.upload_multipart" => ("Multipart form upload error", "Lỗi tải lên dữ liệu dạng multipart", "多部分表单上传错误"),
    UploadReadFailed => "error.upload_read_failed" => ("Failed to read uploaded file bytes", "Không thể đọc dữ liệu file tải lên", "无法读取上传的文件字节"),
    UploadFileTooLarge => "error.upload_file_too_large" => ("Uploaded file is too large", "File tải lên vượt quá dung lượng cho phép", "上传的文件太大"),
    UploadNoFile => "error.upload_no_file" => ("No file was uploaded", "Không có file nào được tải lên", "未上传文件"),
    TenantMismatch => "error.tenant_mismatch" => ("Tenant mismatch", "Không khớp tenant", "租户不匹配"),
    SupavisorApiUrlRequired => "error.supavisor_api_url_required" => ("SUPAVISOR_API_URL environment variable is required", "Yêu cầu biến môi trường SUPAVISOR_API_URL", "需要 SUPAVISOR_API_URL 环境变量"),
    ApiJwtSecretRequired => "error.api_jwt_secret_required" => ("API_JWT_SECRET (or SUPAVISOR_API_JWT) environment variable is required", "Yêu cầu biến môi trường API_JWT_SECRET (hoặc SUPAVISOR_API_JWT)", "API_JWT_SECRET (or SUPAVISOR_API_JWT) 环境变量"),
    StagingLogNotFound => "error.staging_log_not_found" => ("Staging log not found", "Không tìm thấy log tạm thời", "未找到临时日志"),
    StagingLogProcessed => "error.staging_log_processed" => ("Log already processed", "Log đã được xử lý", "日志已处理"),
    StagingLogConflict => "error.staging_log_conflict" => ("Log is being processed by another request", "Log đang được xử lý bởi yêu cầu khác", "日志正在被另一个请求处理"),
    LoginLocked => "error.login_locked" => ("Too many login attempts. Your account has been temporarily locked. Please try again later.", "Quá nhiều lần đăng nhập sai. Tài khoản của bạn đã bị khóa tạm thời. Vui lòng thử lại sau.", "登录尝试次数过多。您的帐户已被临时锁定。请稍后再试。"),
    HolidayNotFound => "error.holiday_not_found" => ("Holiday not found", "Không tìm thấy ngày nghỉ lễ", "未找到节假日"),
    HolidayAlreadyExists => "error.holiday_already_exists" => ("A holiday already exists on this date", "Đã tồn tại ngày nghỉ lễ vào ngày này", "该日期已存在节假日"),
    JobNotFound => "error.job_not_found" => ("Job not found", "Không tìm thấy tiến trình", "未找到任务"),
    ErrorFileNotFound => "error.error_file_not_found" => ("Error file not found or has expired", "Không tìm thấy file lỗi hoặc đã hết hạn", "未找到错误文件或已过期"),
    WorkspaceNotFound => "error.workspace_not_found" => ("Workspace not found", "Không tìm thấy Workspace", "未找到工作空间"),
    CannotRemoveSelf => "error.cannot_remove_self" => ("You cannot remove yourself from the workspace", "Bạn không thể tự xóa bản thân khỏi workspace", "您不能将自己从工作空间中移除"),
    RoleNotFound => "error.role_not_found" => ("Role not found", "Không tìm thấy vai trò", "未找到该角色"),
    RoleNotFoundOrSystem => "error.role_not_found_or_system" => ("Role not found or is a system role", "Không tìm thấy vai trò hoặc đây là vai trò hệ thống", "未找到角色或该角色为系统角色"),
    ExpirationFuture => "error.expiration_future" => ("Expiration time must be in the future", "Thời gian hết hạn phải ở tương lai", "过期时间必须在未来"),
    UnknownPermission => "error.unknown_permission" => ("Unknown permission. Use GET /permissions/available for valid values.", "Quyền không hợp lệ. Sử dụng GET /permissions/available để lấy các giá trị hợp lệ.", "未知权限。请使用 GET /permissions/available 获取有效值。"),
    EmployeeAssignmentNotFound => "error.employee_assignment_not_found" => ("Employee assignment not found", "Không tìm thấy điều động nhân viên", "未找到员工外派记录"),
    HomeHostLocationSame => "error.home_host_location_same" => ("Home location and host location cannot be the same", "Địa điểm nhà và địa điểm host không được giống nhau", "居住地和工作地不能相同"),
    ContractNotFound => "error.contract_not_found" => ("Contract not found", "Không tìm thấy hợp đồng", "未找到合同"),
    LeaveTypeNotFound => "error.leave_type_not_found" => ("Leave type not found", "Không tìm thấy loại nghỉ phép", "未找到请假类型"),
    PolicyNotFound => "error.policy_not_found" => ("Policy not found", "Không tìm thấy chính sách", "未找到政策"),
    AttendanceSummaryNotFound => "error.attendance_summary_not_found" => ("Attendance daily summary not found", "Không tìm thấy bảng tổng hợp chấm công hàng ngày", "未找到每日考勤汇总"),
    InvalidStatus => "error.invalid_status" => ("Invalid status", "Trạng thái không hợp lệ", "无效状态"),
    OvertimeRequestNotFound => "error.overtime_request_not_found" => ("Overtime request not found", "Không tìm thấy yêu cầu tăng ca", "未找到加班申请"),
    OvertimeRequestNoEmployees => "error.overtime_request_no_employees" => ("Overtime request has no employees assigned", "Yêu cầu tăng ca chưa được gán nhân viên nào", "加班申请未分配员工"),
    InvalidStartDateValue => "error.invalid_start_date_format" => ("Invalid start_date format (YYYY-MM-DD)", "Định dạng start_date không hợp lệ (YYYY-MM-DD)", "无效的 start_date 格式 (YYYY-MM-DD)"),
    InvalidEndDateValue => "error.invalid_end_date_format" => ("Invalid end_date format (YYYY-MM-DD)", "Định dạng end_date không hợp lệ (YYYY-MM-DD)", "无效的 end_date 格式 (YYYY-MM-DD)"),
    InvalidYearMonth => "error.invalid_year_month" => ("Invalid year or month format", "Định dạng năm hoặc tháng không hợp lệ", "无效的年份或月份格式"),
    InvalidDateRange => "error.invalid_date_range" => ("Invalid date range", "Khoảng ngày không hợp lệ", "无效的 date range"),
    InvalidDateFormat => "error.invalid_date_format" => ("Invalid date format (YYYY-MM-DD)", "Định dạng ngày không hợp lệ (YYYY-MM-DD)", "无效的日期格式 (YYYY-MM-DD)"),
    NoErrorFileAvailable => "error.no_error_file_available" => ("No error file available for this import", "Không có file lỗi nào khả dụng cho lượt nhập này", "此导入无可用错误文件"),
    WorkflowNotFound => "error.workflow_not_found" => ("Workflow not found", "Không tìm thấy quy trình phê duyệt", "未找到审批流"),
    WorkflowInstanceNotFound => "error.workflow_instance_not_found" => ("Workflow instance not found for this entity", "Không tìm thấy thực thể quy trình phê duyệt cho đối tượng này", "未找到该实体的审批实例"),
    InvalidFeatureFlagKey => "error.invalid_feature_flag_key" => ("flag_key must be between 1 and 255 characters", "flag_key phải nằm trong khoảng từ 1 đến 255 ký tự", "flag_key 长度必须在 1 到 255 个字符之间"),
    FeatureFlagNotFound => "error.feature_flag_not_found" => ("Feature flag not found", "Không tìm thấy feature flag", "未找到 feature flag"),
    ContactNotFound => "error.contact_not_found" => ("Contact not found", "Không tìm thấy thông tin liên hệ", "未找到联系人"),
    MinutesGreaterThanZero => "error.minutes_greater_than_zero" => ("Minutes must be greater than 0", "Số phút phải lớn hơn 0", "分钟数必须大于 0"),
    MultipartError => "error.multipart_error" => ("Multipart form parsing failed", "Lỗi phân tích cú pháp multipart form", "Multipart 表单解析失败"),
    FailedToReadFileBytes => "error.failed_read_file_bytes" => ("Failed to read uploaded file bytes", "Không thể đọc dữ liệu file tải lên", "无法读取上传的文件字节"),
    FileTooLarge => "error.file_too_large" => ("File size exceeds the limit", "Kích thước file vượt quá giới hạn", "文件大小超过限制"),
    NoFileUploaded => "error.no_file_uploaded" => ("No file uploaded", "Không có file nào được tải lên", "未上传文件"),
    InvalidFileFormat => "error.invalid_file_format" => ("Invalid file format. Only Excel files (.xlsx, .xls) are supported.", "Định dạng file không hỗ trợ. Chỉ hỗ trợ file Excel (.xlsx, .xls).", "不支持的文件格式。仅支持 Excel 文件 (.xlsx, .xls)。"),
    InvalidUtf8Csv => "error.invalid_utf8_csv" => ("CSV file must be encoded in UTF-8", "File CSV phải được mã hóa UTF-8", "CSV 文件必须是 UTF-8 编码"),
    CsvParsingError => "error.csv_parsing_error" => ("CSV parsing failed", "Lỗi parse CSV", "CSV 解析失败"),
    MaxRowsExceeded => "error.max_rows_exceeded" => ("Maximum allowed rows exceeded", "Vượt quá số dòng tối đa cho phép", "超过最大允许行数"),
    RosterBatchTooLarge => "error.roster_batch_too_large" => ("Too many days selected in one request. Please select fewer days.", "Chọn quá nhiều ngày trong một yêu cầu. Vui lòng chọn ít ngày hơn.", "单次请求选择的日期过多。请减少选择的日期数量。"),
    SystemRoleNotFound => "error.system_role_not_found" => ("System role not found", "Không tìm thấy vai trò hệ thống", "未找到系统角色"),
    UnknownSystemRole => "error.unknown_system_role" => ("Unknown system role", "Vai trò hệ thống không xác định", "未知系统角色"),
    RoleNameRequired => "error.role_name_required" => ("Role name is required", "Tên vai trò là bắt buộc", "角色名称是必填项"),
    SystemUserNotFound => "error.system_user_not_found" => ("System user not found", "Không tìm thấy người dùng hệ thống", "未找到系统用户"),
    PasswordTooShort => "error.password_too_short" => ("Password must be at least 8 characters", "Mật khẩu phải có ít nhất 8 ký tự", "密码必须至少为 8 个字符"),
    PasswordTooLong => "error.password_too_long" => ("Password must not exceed 128 characters", "Mật khẩu không được vượt quá 128 ký tự", "密码不能超过 128 个字符"),
    PasswordComplexity => "error.password_complexity" => ("Password must contain at least one digit or special character", "Mật khẩu phải chứa ít nhất một chữ số hoặc ký tự đặc biệt", "密码必须包含至少一个数字或特殊字符"),
    InvalidEmailFormat => "error.invalid_email_format" => ("Invalid email format", "Định dạng email không hợp lệ", "电子邮件格式无效"),
    FullNameRequired => "error.full_name_required" => ("Full name is required", "Họ và tên là bắt buộc", "姓名是必填项"),
    EmailAlreadyExists => "error.email_already_exists" => ("Email already exists", "Email đã tồn tại", "电子邮件已存在"),
    InvalidEmailOrPassword => "error.invalid_email_or_password" => ("Invalid email or password", "Email hoặc mật khẩu không đúng", "无效的电子邮件或密码"),
    NoRefreshToken => "error.no_refresh_token" => ("No refresh token provided", "Không có refresh token nào được cung cấp", "未提供 refresh token"),
    CurrentPasswordIncorrect => "error.current_password_incorrect" => ("Current password is incorrect", "Mật khẩu hiện tại không đúng", "当前密码不正确"),
    NoActiveEmployeesFound => "error.no_active_employees_found" => ("No active employees found for report", "Không tìm thấy nhân viên đang hoạt động nào cho báo cáo", "未找到报告的活动员工"),
    CannotCancelJob => "error.cannot_cancel_job" => ("Cannot cancel completed or failed job", "Không thể hủy công việc đã hoàn thành hoặc thất bại", "无法取消已完成或失败的任务"),
    NoReportFileGenerated => "error.no_report_file_generated" => ("No report file generated yet", "Chưa có file báo cáo nào được tạo", "尚未生成报告文件"),
    DeviceNotFound => "error.device_not_found" => ("Device not found", "Không tìm thấy thiết bị", "未找到设备"),
    AgentNotFound => "error.agent_not_found" => ("Agent not found", "Không tìm thấy Agent", "未找到 Agent"),
    NoFieldsToUpdate => "error.no_fields_to_update" => ("No fields to update", "Không có trường nào để cập nhật", "没有要更新的字段"),
    InvalidGroup => "error.invalid_group" => ("Invalid group", "Nhóm không hợp lệ", "无效的组"),
    InvalidTenantIdFormat => "error.invalid_tenant_id_format" => ("Invalid tenant ID format", "Định dạng ID tenant không hợp lệ", "无效的租户 ID 格式"),
    InvalidScopeType => "error.invalid_scope_type" => ("Invalid scope type", "Phạm vi không hợp lệ", "无效的范围类型"),
    LocationNotFound => "error.location_not_found" => ("Location not found", "Không tìm thấy địa điểm", "未找到地点"),
    EmployerNotFound => "error.employer_not_found" => ("Employer not found", "Không tìm thấy người sử dụng lao động", "未找到雇主"),
    RecruitmentSourceNotFound => "error.recruitment_source_not_found" => ("Recruitment source not found", "Không tìm thấy nguồn tuyển dụng", "未找到招聘来源"),
    ShiftNotFound => "error.shift_not_found" => ("Shift not found", "Không tìm thấy ca làm việc", "未找到班次"),
    ShiftAssignmentAlreadyExists => "error.shift_assignment_already_exists" => ("Shift assignment already exists for this date", "Đã tồn tại phân công ca cho ngày này", "该日期已存在班次分配"),
    EmployeePhaseNotFound => "error.employee_phase_not_found" => ("Employee phase not found", "Không tìm thấy giai đoạn nhân viên", "未找到员工阶段"),
    InvalidBaseGroup => "error.invalid_base_group" => ("Invalid base group", "Nhóm cơ bản không hợp lệ", "无效的基本组"),
    InvalidToken => "error.invalid_token" => ("Invalid or expired token", "Token không hợp lệ hoặc đã hết hạn", "无效或过期的令牌"),
    OverflowCalculation => "error.overflow_calculation" => ("Calculation overflow occurred", "Xảy ra lỗi tràn số khi tính toán", "计算溢出"),
    UserAlreadyLinked => "error.user_already_linked" => ("User is already linked to employee {name} in this workspace", "Tài khoản đã được gán cho nhân viên {name} trong workspace này", "该用户已在此工作空间中与员工 {name} 关联"),
    EmployeeAlreadyActive => "error.employee_already_active" => ("Employee is already active", "Nhân viên đang hoạt động", "员工已处于活动状态"),
    RehireDateInvalid => "error.rehire_date_invalid" => ("Rehire date must be after termination date", "Ngày tái tuyển dụng phải sau ngày chấm dứt hợp đồng", "重新雇佣日期必须在解雇日期之后"),
    NoActiveEmployment => "error.no_active_employment" => ("No active employment found for employee", "Không tìm thấy hợp đồng lao động đang hoạt động cho nhân viên", "没有为员工找到处于活动状态的雇佣关系"),
    NoEmploymentOnDate => "error.no_employment_on_date" => ("No employment covers the punch date", "Không có hợp đồng lao động nào bao phủ ngày chấm công", "没有雇佣关系覆盖打卡日期"),
    AmbiguousEmploymentRange => "error.ambiguous_employment_range" => ("Date range spans multiple employments, please specify employment_id", "Khoảng ngày bao phủ nhiều hợp đồng lao động, vui lòng chỉ định employment_id", "日期范围跨越多个雇佣关系，请指定 employment_id"),
    EmployeeAlreadyTerminated => "error.employee_already_terminated" => ("Employee is already terminated", "Nhân viên đã nghỉ việc từ trước", "该员工已被解雇"),
    OverlappingEmployment => "error.overlapping_employment" => ("Overlapping primary employments are not allowed", "Không cho phép các mối quan hệ công việc chính trùng lặp thời gian", "不允许重叠的主要雇佣关系"),
    CustomFieldNotFound => "error.custom_field_not_found" => ("Custom field not found", "Không tìm thấy trường tùy chỉnh", "未找到自定义字段"),
    TemplateNotFound => "error.template_not_found" => ("Lifecycle checklist template not found", "Không tìm thấy mẫu danh sách kiểm tra vòng đời", "未找到生命周期清单模板"),
    TemplateItemNotFound => "error.template_item_not_found" => ("Lifecycle checklist template item not found", "Không tìm thấy mục mẫu danh sách kiểm tra vòng đời", "未找到生命周期清单模板项"),
    TemplateNameEmpty => "error.template_name_empty" => ("Template name cannot be empty", "Tên mẫu không được để trống", "模板名称不能为空"),
    TemplateItemTitleEmpty => "error.template_item_title_empty" => ("Item title cannot be empty", "Tiêu đề mục không được để trống", "项目标题不能为空"),
    ChecklistNotFound => "error.checklist_not_found" => ("Lifecycle checklist not found", "Không tìm thấy danh sách kiểm tra vòng đời", "未找到生命周期清单"),
    ChecklistItemNotFound => "error.checklist_item_not_found" => ("Lifecycle checklist item not found", "Không tìm thấy mục danh sách kiểm tra vòng đời", "未找到生命周期清单项"),
    CannotOpenXlsx => "error.cannot_open_xlsx" => ("Cannot open Excel XLSX file: {error}", "Không thể mở file Excel XLSX: {error}", "无法打开 Excel XLSX 文件: {error}"),
    CannotOpenXls => "error.cannot_open_xls" => ("Cannot open Excel XLS file: {error}", "Không thể mở file Excel XLS: {error}", "无法打开 Excel XLS 文件: {error}"),
    EmptyExcelSheet => "error.empty_excel_sheet" => ("Excel file is empty or contains only headers.", "File Excel trống hoặc chỉ chứa dòng tiêu đề.", "Excel 文件为空或仅包含表头。"),
    DuplicateHeaders => "error.duplicate_headers" => ("Duplicate columns detected: {headers}", "Phát hiện các cột bị trùng lặp: {headers}", "检测到重复的表头: {headers}"),
    MissingMandatoryHeaders => "error.missing_mandatory_headers" => ("Missing mandatory headers: {headers}", "Thiếu các cột bắt buộc: {headers}", "缺少必填表头: {headers}"),
    NoSheetsFound => "error.no_sheets_found" => ("Excel file does not contain any sheets.", "File Excel không chứa bất kỳ sheet nào.", "Excel 文件中未找到任何工作表。"),
    CannotReadSheet => "error.cannot_read_sheet" => ("Cannot read sheet {name}: {error}", "Không thể đọc sheet {name}: {error}", "无法读取工作表 {name}: {error}"),
    EmploymentInactive => "error.employment_inactive" => ("Employment is inactive", "Quan hệ công việc không hoạt động", "雇佣关系处于非活动状态"),
    TimeTrackingTooEarly => "error.time_tracking_too_early" => ("Too early to punch in", "Chưa đến thời gian chấm công", "未到打卡时间"),
    TimeTrackingFailedFetchPunched => "error.time_tracking_failed_fetch_punched" => ("Failed to fetch punched entry", "Không thể tải thông tin lượt chấm công", "未能获取打卡记录"),
    TimeEntryNotFound => "error.time_entry_not_found" => ("Time entry not found", "Không tìm thấy lượt chấm công", "未找到打卡记录"),
    TimeTrackingEditOtherEmployee => "error.time_tracking_edit_other_employee" => ("Not allowed to edit entries from other employees", "Không được phép chỉnh sửa lượt chấm công của nhân viên khác", "不允许编辑其他员工的记录"),
    TimeTrackingEditOtherWorkspace => "error.time_tracking_edit_other_workspace" => ("Not allowed to edit entries from other workspaces", "Không được phép chỉnh sửa lượt chấm công của workspace khác", "不允许编辑其他工作空间的记录"),
    LeaveHalfDaySingleDay => "error.leave_half_day_single_day" => ("Half-day leave must be a single day (start_date == end_date)", "Nghỉ nửa ngày phải nằm trong cùng một ngày (start_date == end_date)", "半天请假必须在同一天内 (start_date == end_date)"),
    LeaveCustomHoursMissingTime => "error.leave_custom_hours_missing_time" => ("Custom hours leave requires both start_time and end_time", "Nghỉ phép theo giờ yêu cầu cả thời gian bắt đầu và kết thúc", "按小时请假需要同时提供开始时间和结束时间"),
    LeaveCustomHoursSingleDay => "error.leave_custom_hours_single_day" => ("Custom hours leave must be a single day (start_date == end_date)", "Nghỉ phép theo giờ phải nằm trong cùng một ngày (start_date == end_date)", "按小时请假必须在同一天内 (start_date == end_date)"),
    LeaveStartTimeBeforeEndTime => "error.leave_start_time_before_end_time" => ("start_time must be before end_time", "Thời gian bắt đầu phải trước thời gian kết thúc", "开始时间必须早于结束时间"),
    LeaveNotFoundAfterUpdate => "error.leave_not_found_after_update" => ("Not found after status update", "Không tìm thấy sau khi cập nhật trạng thái", "更新状态后未找到资源"),
    ResetTokenInvalid => "error.reset_token_invalid" => ("This reset link is invalid or has expired.", "Liên kết đặt lại mật khẩu không hợp lệ hoặc đã hết hạn.", "该密码重置链接无效或已过期。"),
    ResetTokenExpired => "error.reset_token_expired" => ("This reset link has expired.", "Liên kết đặt lại mật khẩu đã hết hạn.", "该密码重置链接已过期。"),
    // Compliance & Rest-Period Validation
    ShiftOverlapDetected => "error.shift_overlap_detected" => ("Shift overlap detected. The employee is already assigned to another shift at this time.", "Phát hiện trùng lặp ca làm việc. Nhân viên đã được phân công vào một ca khác vào thời gian này.", "检测到班次重叠。员工在该时间段已被分配到另一个班次。"),
    InsufficientRestPeriod => "error.insufficient_rest_period" => ("Insufficient rest period between shifts. Minimum required rest is {min_hours}h.", "Thời gian nghỉ giữa các ca không đủ. Yêu cầu nghỉ tối thiểu {min_hours} giờ.", "班次间休息时间不足。最低要求休息 {min_hours} 小时。"),
    WeeklyHoursExceeded => "error.weekly_hours_exceeded" => ("Weekly working hours exceeded. Current: {current}h, limit: {limit}h.", "Tổng giờ làm việc trong tuần vượt quá giới hạn. Hiện tại: {current}h, giới hạn: {limit}h.", "每周工作时间超标。当前：{current}h，限制：{limit}h。"),
    MonthlyOTExceeded => "error.monthly_ot_exceeded" => ("Monthly overtime hours exceeded. Current: {current} OT hours, limit: {limit}h.", "Giờ tăng ca trong tháng vượt quá giới hạn. Hiện tại: {current} giờ, giới hạn: {limit}h.", "每月加班时间超标。当前：{current} 小时，限制：{limit}h。"),
    ComplianceOverrideRequired => "error.compliance_override_required" => ("Compliance override reason is required to proceed with this assignment.", "Cần cung cấp lý do ghi đè tuân thủ để tiếp tục phân công này.", "需要提供合规性覆盖理由才能继续本次分配。"),
}

// ---------------------------------------------------------------------------
// Language / locale plumbing (unchanged)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Vi,
    En,
    Zh,
}

impl FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "vi" | "vi-vn" | "vietnamese" => Ok(Self::Vi),
            "zh" | "zh-cn" | "chinese" => Ok(Self::Zh),
            "en" | "en-us" | "english" => Ok(Self::En),
            _ => Err(()),
        }
    }
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vi => "vi",
            Self::En => "en",
            Self::Zh => "zh",
        }
    }
}

tokio::task_local! {
    /// Holds the resolved language code for the duration of the current async task.
    pub static CURRENT_LOCALE: String;
}

/// Retrieve the active request locale, defaulting to "vi" if not bound.
pub fn get_locale() -> String {
    CURRENT_LOCALE
        .try_with(Clone::clone)
        .unwrap_or_else(|_| "vi".to_string())
}

/// Look up a translation key for the specified language string.
pub fn translate(lang: &str, key: &str) -> String {
    let opt = match lang {
        "en" => lookup_en(key),
        "zh" => lookup_zh(key),
        "vi" => lookup_vi(key),
        _ => None,
    };

    // Fallback: request lang -> vi -> en -> key raw string
    let resolved = opt.or_else(|| lookup_vi(key)).or_else(|| lookup_en(key));
    match resolved {
        Some(s) => s.to_string(),
        None => key.to_string(),
    }
}

/// Look up a translation key using the active task-local locale context.
pub fn t(key: &str) -> String {
    let lang = get_locale();
    translate(&lang, key)
}

/// Look up a translation key using the active locale, replacing format arguments.
///
/// Placeholders are specified as `{key}`.
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    let mut text = t(key);
    for (k, v) in args {
        text = text.replace(&format!("{{{k}}}"), v);
    }
    text
}

/// Look up a translation key using the specified language, replacing format arguments.
///
/// Placeholders are specified as `{key}`.
pub fn translate_args(lang: &str, key: &str, args: &[(&str, &str)]) -> String {
    let mut text = translate(lang, key);
    for (k, v) in args {
        text = text.replace(&format!("{{{k}}}"), v);
    }
    text
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!("vi".parse::<Language>(), Ok(Language::Vi));
        assert_eq!("vi-VN".parse::<Language>(), Ok(Language::Vi));
        assert_eq!("en".parse::<Language>(), Ok(Language::En));
        assert_eq!("EN-US".parse::<Language>(), Ok(Language::En));
        assert_eq!("zh".parse::<Language>(), Ok(Language::Zh));
        assert_eq!("zh-cn".parse::<Language>(), Ok(Language::Zh));
        assert!("fr".parse::<Language>().is_err());
    }

    #[test]
    fn test_translate_fallback() {
        assert_eq!(translate("en", "error.forbidden"), "Permission denied");
        assert_eq!(
            translate("vi", "error.forbidden"),
            "Không có quyền truy cập"
        );
        assert_eq!(translate("zh", "error.forbidden"), "权限拒绝");
        assert_eq!(
            translate("fr", "error.forbidden"),
            "Không có quyền truy cập"
        ); // vi fallback
    }

    #[test]
    fn test_t_args() {
        // Run outside task context - defaults to vi
        let text = t_args("error.validation", &[]);
        assert_eq!(text, "Lỗi kiểm tra dữ liệu");
    }

    #[tokio::test]
    async fn test_task_local_locale() {
        CURRENT_LOCALE
            .scope("en".to_string(), async {
                assert_eq!(get_locale(), "en");
                assert_eq!(t("error.forbidden"), "Permission denied");
            })
            .await;

        CURRENT_LOCALE
            .scope("zh".to_string(), async {
                assert_eq!(get_locale(), "zh");
                assert_eq!(t("error.forbidden"), "权限拒绝");
            })
            .await;

        // Verify fallback
        assert_eq!(get_locale(), "vi");
    }

    #[test]
    fn test_all_keys_translated_in_all_languages() {
        for &key in ALL_KEYS {
            let en_val = lookup_en(key);
            let vi_val = lookup_vi(key);
            let zh_val = lookup_zh(key);

            assert!(
                en_val.is_some(),
                "Key '{key}' is missing from English translations"
            );
            assert!(
                vi_val.is_some(),
                "Key '{key}' is missing from Vietnamese translations"
            );
            assert!(
                zh_val.is_some(),
                "Key '{key}' is missing from Chinese translations"
            );
        }
    }

    #[test]
    fn test_all_ui_messages_use_i18n_keys() {
        use std::fs;
        use std::path::PathBuf;

        fn get_rs_files(dir: &PathBuf) -> Vec<PathBuf> {
            let mut files = Vec::new();
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(get_rs_files(&path));
                    } else if path.extension().is_some_and(|e| e == "rs") {
                        files.push(path);
                    }
                }
            }
            files
        }

        let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let rs_files = get_rs_files(&src_dir);
        let mut violations = Vec::new();

        for file_path in &rs_files {
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name == "i18n.rs" || file_name == "errors.rs" {
                continue;
            }

            let content = fs::read_to_string(file_path).unwrap_or_default();
            for (line_no, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                let is_app_error_call = trimmed.contains("AppError::Validation(")
                    || trimmed.contains("AppError::Auth(")
                    || trimmed.contains("AppError::Forbidden(")
                    || trimmed.contains("AppError::NotFound(")
                    || trimmed.contains("AppError::Conflict(")
                    || trimmed.contains("AppError::ServiceUnavailable(");

                if is_app_error_call {
                    if let Some(first_quote_idx) = trimmed.find('"') {
                        let rest = &trimmed[first_quote_idx + 1..];
                        if let Some(second_quote_idx) = rest.find('"') {
                            let key = &rest[..second_quote_idx];
                            if key.starts_with("error.") {
                                if !ALL_KEYS.contains(&key) {
                                    violations.push(format!(
                                        "  {}:{} -> Key '{}' is not registered in ALL_KEYS",
                                        file_name,
                                        line_no + 1,
                                        key
                                    ));
                                }
                            } else if !key.starts_with("dev.") && !key.starts_with('x') {
                                violations.push(format!(
                                    "  {}:{} -> Raw text message in UI error: {}",
                                    file_name,
                                    line_no + 1,
                                    trimmed
                                ));
                            }
                        }
                    }
                }
            }
        }

        assert!(
            violations.is_empty(),
            "\n\n🔴 UI MESSAGES I18N GUARD FAILURE\n\
             ════════════════════════════════════════════════════════════\n\
             The following files/lines return raw error messages to the UI without using i18n keys:\n\n{}\n\n\
             FIX: Replace raw messages with a key starting with 'error.' and add the key to utils/i18n.rs.\n\
             ════════════════════════════════════════════════════════════\n",
            violations.join("\n")
        );
    }
}
