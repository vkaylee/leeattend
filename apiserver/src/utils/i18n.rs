//! Backend internationalization (i18n) utility module.
//!
//! Provides context propagation using tokio task-local storage and type-safe
//! translation lookups for English, Vietnamese, and Chinese.

use std::str::FromStr;

pub const ALL_KEYS: &[&str] = &[
    "error.unauthorized",
    "error.authentication_required",
    "error.forbidden",
    "error.not_found",
    "error.validation",
    "error.internal",
    "error.conflict",
    "error.service_unavailable",
    "error.missing_workspace",
    "error.invalid_workspace_id",
    "error.no_employee_record",
    "error.system_route_tenant_context",
    "error.not_workspace_member",
    "error.user_not_found",
    "error.employee_not_found",
    "error.employment_not_found",
    "error.department_not_found",
    "error.database_error",
    "error.cannot_revoke_last_admin",
    "error.behalf_assignment_missing_target",
    "error.behalf_assignment_dual_target",
    "error.upload_multipart",
    "error.upload_read_failed",
    "error.upload_file_too_large",
    "error.upload_no_file",
    "error.tenant_mismatch",
    "error.supavisor_api_url_required",
    "error.api_jwt_secret_required",
    "error.staging_log_not_found",
    "error.staging_log_processed",
    "error.staging_log_conflict",
    "error.login_locked",
    "error.holiday_not_found",
    "error.holiday_already_exists",
    "error.job_not_found",
    "error.error_file_not_found",
    "error.workspace_not_found",
    "error.cannot_remove_self",
    "error.role_not_found",
    "error.role_not_found_or_system",
    "error.expiration_future",
    "error.unknown_permission",
    "error.employee_assignment_not_found",
    "error.home_host_location_same",
    "error.contract_not_found",
    "error.leave_type_not_found",
    "error.policy_not_found",
    "error.attendance_summary_not_found",
    "error.invalid_status",
    "error.overtime_request_not_found",
    "error.overtime_request_no_employees",
    "error.invalid_start_date_format",
    "error.invalid_end_date_format",
    "error.invalid_year_month",
    "error.invalid_date_range",
    "error.invalid_date_format",
    "error.no_error_file_available",
    "error.workflow_not_found",
    "error.workflow_instance_not_found",
    "error.invalid_feature_flag_key",
    "error.feature_flag_not_found",
    "error.contact_not_found",
    "error.minutes_greater_than_zero",
    "error.multipart_error",
    "error.failed_read_file_bytes",
    "error.file_too_large",
    "error.no_file_uploaded",
    "error.invalid_utf8_csv",
    "error.csv_parsing_error",
    "error.max_rows_exceeded",
    "error.system_role_not_found",
    "error.unknown_system_role",
    "error.role_name_required",
    "error.system_user_not_found",
    "error.password_too_short",
    "error.password_too_long",
    "error.password_complexity",
    "error.invalid_email_format",
    "error.full_name_required",
    "error.email_already_exists",
    "error.invalid_email_or_password",
    "error.no_refresh_token",
    "error.current_password_incorrect",
    "error.no_active_employees_found",
    "error.cannot_cancel_job",
    "error.no_report_file_generated",
    "error.device_not_found",
    "error.agent_not_found",
    "error.no_fields_to_update",
    "error.invalid_group",
    "error.invalid_tenant_id_format",
    "error.invalid_scope_type",
    "error.location_not_found",
    "error.employer_not_found",
    "error.recruitment_source_not_found",
    "error.shift_not_found",
    "error.shift_assignment_already_exists",
    "error.employee_phase_not_found",
    "error.invalid_base_group",
    "error.invalid_token",
    "error.overflow_calculation",
    "error.user_already_linked",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I18nKey {
    Unauthorized,
    AuthenticationRequired,
    Forbidden,
    NotFound,
    Validation,
    Internal,
    Conflict,
    ServiceUnavailable,
    MissingWorkspace,
    InvalidWorkspaceId,
    NoEmployeeRecord,
    SystemRouteTenantContext,
    NotWorkspaceMember,
    UserNotFound,
    EmployeeNotFound,
    EmploymentNotFound,
    DepartmentNotFound,
    DatabaseError,
    CannotRevokeLastAdmin,
    BehalfAssignmentMissingTarget,
    BehalfAssignmentDualTarget,
    UploadMultipart,
    UploadReadFailed,
    UploadFileTooLarge,
    UploadNoFile,
    TenantMismatch,
    SupavisorApiUrlRequired,
    ApiJwtSecretRequired,
    StagingLogNotFound,
    StagingLogProcessed,
    StagingLogConflict,
    LoginLocked,
    HolidayNotFound,
    HolidayAlreadyExists,
    JobNotFound,
    ErrorFileNotFound,
    WorkspaceNotFound,
    CannotRemoveSelf,
    RoleNotFound,
    RoleNotFoundOrSystem,
    ExpirationFuture,
    UnknownPermission,
    EmployeeAssignmentNotFound,
    HomeHostLocationSame,
    ContractNotFound,
    LeaveTypeNotFound,
    PolicyNotFound,
    AttendanceSummaryNotFound,
    InvalidStatus,
    OvertimeRequestNotFound,
    OvertimeRequestNoEmployees,
    InvalidStartDateValue,
    InvalidEndDateValue,
    InvalidYearMonth,
    InvalidDateRange,
    InvalidDateFormat,
    NoErrorFileAvailable,
    WorkflowNotFound,
    WorkflowInstanceNotFound,
    InvalidFeatureFlagKey,
    FeatureFlagNotFound,
    ContactNotFound,
    MinutesGreaterThanZero,
    MultipartError,
    FailedToReadFileBytes,
    FileTooLarge,
    NoFileUploaded,
    InvalidUtf8Csv,
    CsvParsingError,
    MaxRowsExceeded,
    SystemRoleNotFound,
    UnknownSystemRole,
    RoleNameRequired,
    SystemUserNotFound,
    PasswordTooShort,
    PasswordTooLong,
    PasswordComplexity,
    InvalidEmailFormat,
    FullNameRequired,
    EmailAlreadyExists,
    InvalidEmailOrPassword,
    NoRefreshToken,
    CurrentPasswordIncorrect,
    NoActiveEmployeesFound,
    CannotCancelJob,
    NoReportFileGenerated,
    DeviceNotFound,
    AgentNotFound,
    NoFieldsToUpdate,
    InvalidGroup,
    InvalidTenantIdFormat,
    InvalidScopeType,
    LocationNotFound,
    EmployerNotFound,
    RecruitmentSourceNotFound,
    ShiftNotFound,
    ShiftAssignmentAlreadyExists,
    EmployeePhaseNotFound,
    InvalidBaseGroup,
    InvalidToken,
    OverflowCalculation,
    UserAlreadyLinked,
}

impl I18nKey {
    #[allow(clippy::too_many_lines)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unauthorized => "error.unauthorized",
            Self::AuthenticationRequired => "error.authentication_required",
            Self::Forbidden => "error.forbidden",
            Self::NotFound => "error.not_found",
            Self::Validation => "error.validation",
            Self::Internal => "error.internal",
            Self::Conflict => "error.conflict",
            Self::ServiceUnavailable => "error.service_unavailable",
            Self::MissingWorkspace => "error.missing_workspace",
            Self::InvalidWorkspaceId => "error.invalid_workspace_id",
            Self::NoEmployeeRecord => "error.no_employee_record",
            Self::SystemRouteTenantContext => "error.system_route_tenant_context",
            Self::NotWorkspaceMember => "error.not_workspace_member",
            Self::UserNotFound => "error.user_not_found",
            Self::EmployeeNotFound => "error.employee_not_found",
            Self::EmploymentNotFound => "error.employment_not_found",
            Self::DepartmentNotFound => "error.department_not_found",
            Self::DatabaseError => "error.database_error",
            Self::CannotRevokeLastAdmin => "error.cannot_revoke_last_admin",
            Self::BehalfAssignmentMissingTarget => "error.behalf_assignment_missing_target",
            Self::BehalfAssignmentDualTarget => "error.behalf_assignment_dual_target",
            Self::UploadMultipart => "error.upload_multipart",
            Self::UploadReadFailed => "error.upload_read_failed",
            Self::UploadFileTooLarge => "error.upload_file_too_large",
            Self::UploadNoFile => "error.upload_no_file",
            Self::TenantMismatch => "error.tenant_mismatch",
            Self::SupavisorApiUrlRequired => "error.supavisor_api_url_required",
            Self::ApiJwtSecretRequired => "error.api_jwt_secret_required",
            Self::StagingLogNotFound => "error.staging_log_not_found",
            Self::StagingLogProcessed => "error.staging_log_processed",
            Self::StagingLogConflict => "error.staging_log_conflict",
            Self::LoginLocked => "error.login_locked",
            Self::HolidayNotFound => "error.holiday_not_found",
            Self::HolidayAlreadyExists => "error.holiday_already_exists",
            Self::JobNotFound => "error.job_not_found",
            Self::ErrorFileNotFound => "error.error_file_not_found",
            Self::WorkspaceNotFound => "error.workspace_not_found",
            Self::CannotRemoveSelf => "error.cannot_remove_self",
            Self::RoleNotFound => "error.role_not_found",
            Self::RoleNotFoundOrSystem => "error.role_not_found_or_system",
            Self::ExpirationFuture => "error.expiration_future",
            Self::UnknownPermission => "error.unknown_permission",
            Self::EmployeeAssignmentNotFound => "error.employee_assignment_not_found",
            Self::HomeHostLocationSame => "error.home_host_location_same",
            Self::ContractNotFound => "error.contract_not_found",
            Self::LeaveTypeNotFound => "error.leave_type_not_found",
            Self::PolicyNotFound => "error.policy_not_found",
            Self::AttendanceSummaryNotFound => "error.attendance_summary_not_found",
            Self::InvalidStatus => "error.invalid_status",
            Self::OvertimeRequestNotFound => "error.overtime_request_not_found",
            Self::OvertimeRequestNoEmployees => "error.overtime_request_no_employees",
            Self::InvalidStartDateValue => "error.invalid_start_date_format",
            Self::InvalidEndDateValue => "error.invalid_end_date_format",
            Self::InvalidYearMonth => "error.invalid_year_month",
            Self::InvalidDateRange => "error.invalid_date_range",
            Self::InvalidDateFormat => "error.invalid_date_format",
            Self::NoErrorFileAvailable => "error.no_error_file_available",
            Self::WorkflowNotFound => "error.workflow_not_found",
            Self::WorkflowInstanceNotFound => "error.workflow_instance_not_found",
            Self::InvalidFeatureFlagKey => "error.invalid_feature_flag_key",
            Self::FeatureFlagNotFound => "error.feature_flag_not_found",
            Self::ContactNotFound => "error.contact_not_found",
            Self::MinutesGreaterThanZero => "error.minutes_greater_than_zero",
            Self::MultipartError => "error.multipart_error",
            Self::FailedToReadFileBytes => "error.failed_read_file_bytes",
            Self::FileTooLarge => "error.file_too_large",
            Self::NoFileUploaded => "error.no_file_uploaded",
            Self::InvalidUtf8Csv => "error.invalid_utf8_csv",
            Self::CsvParsingError => "error.csv_parsing_error",
            Self::MaxRowsExceeded => "error.max_rows_exceeded",
            Self::SystemRoleNotFound => "error.system_role_not_found",
            Self::UnknownSystemRole => "error.unknown_system_role",
            Self::RoleNameRequired => "error.role_name_required",
            Self::SystemUserNotFound => "error.system_user_not_found",
            Self::PasswordTooShort => "error.password_too_short",
            Self::PasswordTooLong => "error.password_too_long",
            Self::PasswordComplexity => "error.password_complexity",
            Self::InvalidEmailFormat => "error.invalid_email_format",
            Self::FullNameRequired => "error.full_name_required",
            Self::EmailAlreadyExists => "error.email_already_exists",
            Self::InvalidEmailOrPassword => "error.invalid_email_or_password",
            Self::NoRefreshToken => "error.no_refresh_token",
            Self::CurrentPasswordIncorrect => "error.current_password_incorrect",
            Self::NoActiveEmployeesFound => "error.no_active_employees_found",
            Self::CannotCancelJob => "error.cannot_cancel_job",
            Self::NoReportFileGenerated => "error.no_report_file_generated",
            Self::DeviceNotFound => "error.device_not_found",
            Self::AgentNotFound => "error.agent_not_found",
            Self::NoFieldsToUpdate => "error.no_fields_to_update",
            Self::InvalidGroup => "error.invalid_group",
            Self::InvalidTenantIdFormat => "error.invalid_tenant_id_format",
            Self::InvalidScopeType => "error.invalid_scope_type",
            Self::LocationNotFound => "error.location_not_found",
            Self::EmployerNotFound => "error.employer_not_found",
            Self::RecruitmentSourceNotFound => "error.recruitment_source_not_found",
            Self::ShiftNotFound => "error.shift_not_found",
            Self::ShiftAssignmentAlreadyExists => "error.shift_assignment_already_exists",
            Self::EmployeePhaseNotFound => "error.employee_phase_not_found",
            Self::InvalidBaseGroup => "error.invalid_base_group",
            Self::InvalidToken => "error.invalid_token",
            Self::OverflowCalculation => "error.overflow_calculation",
            Self::UserAlreadyLinked => "error.user_already_linked",
        }
    }
}

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

#[allow(clippy::too_many_lines, clippy::match_same_arms)]
fn lookup_en(key: &str) -> Option<&'static str> {
    match key {
        "error.unauthorized" | "error.authentication_required" => Some("Authentication required"),
        "error.forbidden" => Some("Permission denied"),
        "error.not_found" => Some("Resource not found"),
        "error.validation" => Some("Validation failed"),
        "error.internal" => Some("An internal server error occurred."),
        "error.conflict" => Some("A conflict occurred."),
        "error.service_unavailable" => Some("Service unavailable."),
        "error.missing_workspace" => {
            Some("Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header.")
        }
        "error.invalid_workspace_id" => Some("Invalid Workspace ID format"),
        "error.no_employee_record" => Some("No employee record linked to your account"),
        "error.system_route_tenant_context" => {
            Some("Cannot use Tenant context (MemberContext) in a System route")
        }
        "error.not_workspace_member" => Some("Not a member of this workspace"),
        "error.user_not_found" => Some("User not found"),
        "error.employee_not_found" => Some("Employee not found"),
        "error.employment_not_found" => Some("Employment not found"),
        "error.department_not_found" => Some("Department not found"),
        "error.database_error" => Some("Database error"),
        "error.cannot_revoke_last_admin" => {
            Some("Cannot revoke the last platform_admin. Assign another admin first.")
        }
        "error.behalf_assignment_missing_target" => {
            Some("Either target employee or target department must be specified")
        }
        "error.behalf_assignment_dual_target" => {
            Some("Cannot specify both target employee and target department")
        }
        "error.upload_multipart" => Some("Multipart form upload error"),
        "error.upload_read_failed" => Some("Failed to read uploaded file bytes"),
        "error.upload_file_too_large" => Some("Uploaded file is too large"),
        "error.upload_no_file" => Some("No file was uploaded"),
        "error.tenant_mismatch" => Some("Tenant mismatch"),
        "error.supavisor_api_url_required" => {
            Some("SUPAVISOR_API_URL environment variable is required")
        }
        "error.api_jwt_secret_required" => {
            Some("API_JWT_SECRET (or SUPAVISOR_API_JWT) environment variable is required")
        }
        "error.staging_log_not_found" => Some("Staging log not found"),
        "error.staging_log_processed" => Some("Log already processed"),
        "error.staging_log_conflict" => Some("Log is being processed by another request"),
        "error.login_locked" => {
            Some("Too many login attempts. Your account has been temporarily locked. Please try again later.")
        }
        "error.holiday_not_found" => Some("Holiday not found"),
        "error.holiday_already_exists" => Some("A holiday already exists on this date"),
        "error.job_not_found" => Some("Job not found"),
        "error.error_file_not_found" => Some("Error file not found or has expired"),
        "error.workspace_not_found" => Some("Workspace not found"),
        "error.cannot_remove_self" => Some("You cannot remove yourself from the workspace"),
        "error.role_not_found" => Some("Role not found"),
        "error.role_not_found_or_system" => Some("Role not found or is a system role"),
        "error.expiration_future" => Some("Expiration time must be in the future"),
        "error.unknown_permission" => Some("Unknown permission. Use GET /permissions/available for valid values."),
        "error.employee_assignment_not_found" => Some("Employee assignment not found"),
        "error.home_host_location_same" => Some("Home location and host location cannot be the same"),
        "error.contract_not_found" => Some("Contract not found"),
        "error.leave_type_not_found" => Some("Leave type not found"),
        "error.policy_not_found" => Some("Policy not found"),
        "error.attendance_summary_not_found" => Some("Attendance daily summary not found"),
        "error.invalid_status" => Some("Invalid status"),
        "error.overtime_request_not_found" => Some("Overtime request not found"),
        "error.overtime_request_no_employees" => Some("Overtime request has no employees assigned"),
        "error.invalid_start_date_format" => Some("Invalid start_date format (YYYY-MM-DD)"),
        "error.invalid_end_date_format" => Some("Invalid end_date format (YYYY-MM-DD)"),
        "error.invalid_year_month" => Some("Invalid year or month format"),
        "error.invalid_date_range" => Some("Invalid date range"),
        "error.invalid_date_format" => Some("Invalid date format (YYYY-MM-DD)"),
        "error.no_error_file_available" => Some("No error file available for this import"),
        "error.workflow_not_found" => Some("Workflow not found"),
        "error.workflow_instance_not_found" => Some("Workflow instance not found for this entity"),
        "error.invalid_feature_flag_key" => Some("flag_key must be between 1 and 255 characters"),
        "error.feature_flag_not_found" => Some("Feature flag not found"),
        "error.contact_not_found" => Some("Contact not found"),
        "error.minutes_greater_than_zero" => Some("Minutes must be greater than 0"),
        "error.multipart_error" => Some("Multipart form parsing failed"),
        "error.failed_read_file_bytes" => Some("Failed to read uploaded file bytes"),
        "error.file_too_large" => Some("File size exceeds the limit"),
        "error.no_file_uploaded" => Some("No file uploaded"),
        "error.invalid_utf8_csv" => Some("CSV file must be encoded in UTF-8"),
        "error.csv_parsing_error" => Some("CSV parsing failed"),
        "error.max_rows_exceeded" => Some("Maximum allowed rows exceeded"),
        "error.system_role_not_found" => Some("System role not found"),
        "error.unknown_system_role" => Some("Unknown system role"),
        "error.role_name_required" => Some("Role name is required"),
        "error.system_user_not_found" => Some("System user not found"),
        "error.password_too_short" => Some("Password must be at least 8 characters"),
        "error.password_too_long" => Some("Password must not exceed 128 characters"),
        "error.password_complexity" => Some("Password must contain at least one digit or special character"),
        "error.invalid_email_format" => Some("Invalid email format"),
        "error.full_name_required" => Some("Full name is required"),
        "error.email_already_exists" => Some("Email already exists"),
        "error.invalid_email_or_password" => Some("Invalid email or password"),
        "error.no_refresh_token" => Some("No refresh token provided"),
        "error.current_password_incorrect" => Some("Current password is incorrect"),
        "error.no_active_employees_found" => Some("No active employees found for report"),
        "error.cannot_cancel_job" => Some("Cannot cancel completed or failed job"),
        "error.no_report_file_generated" => Some("No report file generated yet"),
        "error.device_not_found" => Some("Device not found"),
        "error.agent_not_found" => Some("Agent not found"),
        "error.no_fields_to_update" => Some("No fields to update"),
        "error.invalid_group" => Some("Invalid group"),
        "error.invalid_tenant_id_format" => Some("Invalid tenant ID format"),
        "error.invalid_scope_type" => Some("Invalid scope type"),
        "error.location_not_found" => Some("Location not found"),
        "error.employer_not_found" => Some("Employer not found"),
        "error.recruitment_source_not_found" => Some("Recruitment source not found"),
        "error.shift_not_found" => Some("Shift not found"),
        "error.shift_assignment_already_exists" => Some("Shift assignment already exists for this date"),
        "error.employee_phase_not_found" => Some("Employee phase not found"),
        "error.invalid_base_group" => Some("Invalid base group"),
        "error.invalid_token" => Some("Invalid or expired token"),
        "error.overflow_calculation" => Some("Calculation overflow occurred"),
        "error.user_already_linked" => Some("User is already linked to employee {name} in this workspace"),
        _ => None,
    }
}

#[allow(clippy::too_many_lines, clippy::match_same_arms)]
fn lookup_vi(key: &str) -> Option<&'static str> {
    match key {
        "error.unauthorized" | "error.authentication_required" => Some("Yêu cầu đăng nhập"),
        "error.forbidden" => Some("Không có quyền truy cập"),
        "error.not_found" => Some("Không tìm thấy tài nguyên"),
        "error.validation" => Some("Lỗi kiểm tra dữ liệu"),
        "error.internal" => Some("Đã xảy ra lỗi hệ thống."),
        "error.conflict" => Some("Đã xảy ra xung đột dữ liệu."),
        "error.service_unavailable" => Some("Dịch vụ hiện không khả dụng."),
        "error.missing_workspace" => Some("Thiếu thông tin Workspace. Vui lòng truyền 'ws_id' trong URL hoặc header 'X-Workspace-Id'."),
        "error.invalid_workspace_id" => Some("Định dạng ID Workspace không hợp lệ"),
        "error.no_employee_record" => Some("Không có thông tin nhân viên nào liên kết với tài khoản của bạn"),
        "error.system_route_tenant_context" => Some("Không thể sử dụng context Tenant (MemberContext) trong route hệ thống"),
        "error.not_workspace_member" => Some("Bạn không phải là thành viên của workspace này"),
        "error.user_not_found" => Some("Không tìm thấy người dùng"),
        "error.employee_not_found" => Some("Không tìm thấy nhân viên"),
        "error.employment_not_found" => Some("Không tìm thấy quan hệ công việc"),
        "error.department_not_found" => Some("Không tìm thấy phòng ban"),
        "error.database_error" => Some("Lỗi cơ sở dữ liệu"),
        "error.cannot_revoke_last_admin" => Some("Không thể thu hồi platform_admin cuối cùng. Hãy gán quản trị viên khác trước."),
        "error.behalf_assignment_missing_target" => Some("Phải chỉ định nhân viên đích hoặc phòng ban đích"),
        "error.behalf_assignment_dual_target" => Some("Không thể chỉ định cả nhân viên đích và phòng ban đích"),
        "error.upload_multipart" => Some("Lỗi tải lên dữ liệu dạng multipart"),
        "error.upload_read_failed" => Some("Không thể đọc dữ liệu file tải lên"),
        "error.upload_file_too_large" => Some("File tải lên vượt quá dung lượng cho phép"),
        "error.upload_no_file" => Some("Không có file nào được tải lên"),
        "error.tenant_mismatch" => Some("Không khớp tenant"),
        "error.supavisor_api_url_required" => Some("Yêu cầu biến môi trường SUPAVISOR_API_URL"),
        "error.api_jwt_secret_required" => Some("Yêu cầu biến môi trường API_JWT_SECRET (hoặc SUPAVISOR_API_JWT)"),
        "error.staging_log_not_found" => Some("Không tìm thấy log tạm thời"),
        "error.staging_log_processed" => Some("Log đã được xử lý"),
        "error.staging_log_conflict" => Some("Log đang được xử lý bởi yêu cầu khác"),
        "error.login_locked" => {
            Some("Quá nhiều lần đăng nhập sai. Tài khoản của bạn đã bị khóa tạm thời. Vui lòng thử lại sau.")
        }
        "error.holiday_not_found" => Some("Không tìm thấy ngày nghỉ lễ"),
        "error.holiday_already_exists" => Some("Đã tồn tại ngày nghỉ lễ vào ngày này"),
        "error.job_not_found" => Some("Không tìm thấy tiến trình"),
        "error.error_file_not_found" => Some("Không tìm thấy file lỗi hoặc đã hết hạn"),
        "error.workspace_not_found" => Some("Không tìm thấy Workspace"),
        "error.cannot_remove_self" => Some("Bạn không thể tự xóa bản thân khỏi workspace"),
        "error.role_not_found" => Some("Không tìm thấy vai trò"),
        "error.role_not_found_or_system" => Some("Không tìm thấy vai trò hoặc đây là vai trò hệ thống"),
        "error.expiration_future" => Some("Thời gian hết hạn phải ở tương lai"),
        "error.unknown_permission" => Some("Quyền không hợp lệ. Sử dụng GET /permissions/available để lấy các giá trị hợp lệ."),
        "error.employee_assignment_not_found" => Some("Không tìm thấy điều động nhân viên"),
        "error.home_host_location_same" => Some("Địa điểm nhà và địa điểm host không được giống nhau"),
        "error.contract_not_found" => Some("Không tìm thấy hợp đồng"),
        "error.leave_type_not_found" => Some("Không tìm thấy loại nghỉ phép"),
        "error.policy_not_found" => Some("Không tìm thấy chính sách"),
        "error.attendance_summary_not_found" => Some("Không tìm thấy bảng tổng hợp chấm công hàng ngày"),
        "error.invalid_status" => Some("Trạng thái không hợp lệ"),
        "error.overtime_request_not_found" => Some("Không tìm thấy yêu cầu tăng ca"),
        "error.overtime_request_no_employees" => Some("Yêu cầu tăng ca chưa được gán nhân viên nào"),
        "error.invalid_start_date_format" => Some("Định dạng start_date không hợp lệ (YYYY-MM-DD)"),
        "error.invalid_end_date_format" => Some("Định dạng end_date không hợp lệ (YYYY-MM-DD)"),
        "error.invalid_year_month" => Some("Định dạng năm hoặc tháng không hợp lệ"),
        "error.invalid_date_range" => Some("Khoảng ngày không hợp lệ"),
        "error.invalid_date_format" => Some("Định dạng ngày không hợp lệ (YYYY-MM-DD)"),
        "error.no_error_file_available" => Some("Không có file lỗi nào khả dụng cho lượt nhập này"),
        "error.workflow_not_found" => Some("Không tìm thấy quy trình phê duyệt"),
        "error.workflow_instance_not_found" => Some("Không tìm thấy thực thể quy trình phê duyệt cho đối tượng này"),
        "error.invalid_feature_flag_key" => Some("flag_key phải nằm trong khoảng từ 1 đến 255 ký tự"),
        "error.feature_flag_not_found" => Some("Không tìm thấy feature flag"),
        "error.contact_not_found" => Some("Không tìm thấy thông tin liên hệ"),
        "error.minutes_greater_than_zero" => Some("Số phút phải lớn hơn 0"),
        "error.multipart_error" => Some("Lỗi phân tích cú pháp multipart form"),
        "error.failed_read_file_bytes" => Some("Không thể đọc dữ liệu file tải lên"),
        "error.file_too_large" => Some("Kích thước file vượt quá giới hạn"),
        "error.no_file_uploaded" => Some("Không có file nào được tải lên"),
        "error.invalid_utf8_csv" => Some("File CSV phải được mã hóa UTF-8"),
        "error.csv_parsing_error" => Some("Lỗi parse CSV"),
        "error.max_rows_exceeded" => Some("Vượt quá số dòng tối đa cho phép"),
        "error.system_role_not_found" => Some("Không tìm thấy vai trò hệ thống"),
        "error.unknown_system_role" => Some("Vai trò hệ thống không xác định"),
        "error.role_name_required" => Some("Tên vai trò là bắt buộc"),
        "error.system_user_not_found" => Some("Không tìm thấy người dùng hệ thống"),
        "error.password_too_short" => Some("Mật khẩu phải có ít nhất 8 ký tự"),
        "error.password_too_long" => Some("Mật khẩu không được vượt quá 128 ký tự"),
        "error.password_complexity" => Some("Mật khẩu phải chứa ít nhất một chữ số hoặc ký tự đặc biệt"),
        "error.invalid_email_format" => Some("Định dạng email không hợp lệ"),
        "error.full_name_required" => Some("Họ và tên là bắt buộc"),
        "error.email_already_exists" => Some("Email đã tồn tại"),
        "error.invalid_email_or_password" => Some("Email hoặc mật khẩu không đúng"),
        "error.no_refresh_token" => Some("Không có refresh token nào được cung cấp"),
        "error.current_password_incorrect" => Some("Mật khẩu hiện tại không đúng"),
        "error.no_active_employees_found" => Some("Không tìm thấy nhân viên đang hoạt động nào cho báo cáo"),
        "error.cannot_cancel_job" => Some("Không thể hủy công việc đã hoàn thành hoặc thất bại"),
        "error.no_report_file_generated" => Some("Chưa có file báo cáo nào được tạo"),
        "error.device_not_found" => Some("Không tìm thấy thiết bị"),
        "error.agent_not_found" => Some("Không tìm thấy Agent"),
        "error.no_fields_to_update" => Some("Không có trường nào để cập nhật"),
        "error.invalid_group" => Some("Nhóm không hợp lệ"),
        "error.invalid_tenant_id_format" => Some("Định dạng ID tenant không hợp lệ"),
        "error.invalid_scope_type" => Some("Phạm vi không hợp lệ"),
        "error.location_not_found" => Some("Không tìm thấy địa điểm"),
        "error.employer_not_found" => Some("Không tìm thấy người sử dụng lao động"),
        "error.recruitment_source_not_found" => Some("Không tìm thấy nguồn tuyển dụng"),
        "error.shift_not_found" => Some("Không tìm thấy ca làm việc"),
        "error.shift_assignment_already_exists" => Some("Đã tồn tại phân công ca cho ngày này"),
        "error.employee_phase_not_found" => Some("Không tìm thấy giai đoạn nhân viên"),
        "error.invalid_base_group" => Some("Nhóm cơ bản không hợp lệ"),
        "error.invalid_token" => Some("Token không hợp lệ hoặc đã hết hạn"),
        "error.overflow_calculation" => Some("Xảy ra lỗi tràn số khi tính toán"),
        "error.user_already_linked" => Some("Tài khoản đã được gán cho nhân viên {name} trong workspace này"),
        _ => None,
    }
}

#[allow(clippy::too_many_lines, clippy::match_same_arms)]
fn lookup_zh(key: &str) -> Option<&'static str> {
    match key {
        "error.unauthorized" | "error.authentication_required" => Some("需要身份验证"),
        "error.forbidden" => Some("权限拒绝"),
        "error.not_found" => Some("未找到资源"),
        "error.validation" => Some("数据验证失败"),
        "error.internal" => Some("发生系统内部错误。"),
        "error.conflict" => Some("数据冲突。"),
        "error.service_unavailable" => Some("服务暂不可用。"),
        "error.missing_workspace" => {
            Some("缺少工作空间上下文。请在路径中提供 'ws_id' 或在 'X-Workspace-Id' 请求头中提供。")
        }
        "error.invalid_workspace_id" => Some("工作空间 ID 格式无效"),
        "error.no_employee_record" => Some("您的账户未关联任何员工档案"),
        "error.system_route_tenant_context" => Some("系统路由中无法使用租户上下文 (MemberContext)"),
        "error.not_workspace_member" => Some("您不是该工作空间的成员"),
        "error.user_not_found" => Some("未找到该用户"),
        "error.employee_not_found" => Some("未找到员工"),
        "error.employment_not_found" => Some("未找到工作关系"),
        "error.department_not_found" => Some("未找到部门"),
        "error.database_error" => Some("数据库错误"),
        "error.cannot_revoke_last_admin" => {
            Some("无法撤销最后一个 platform_admin。请先分配另一个管理员。")
        }
        "error.behalf_assignment_missing_target" => Some("必须指定目标员工或目标部门"),
        "error.behalf_assignment_dual_target" => Some("不能同时指定目标员工和目标部门"),
        "error.upload_multipart" => Some("多部分表单上传错误"),
        "error.upload_read_failed" => Some("无法读取上传的文件字节"),
        "error.upload_file_too_large" => Some("上传的文件太大"),
        "error.upload_no_file" => Some("未上传文件"),
        "error.tenant_mismatch" => Some("租户不匹配"),
        "error.supavisor_api_url_required" => Some("需要 SUPAVISOR_API_URL 环境变量"),
        "error.api_jwt_secret_required" => Some("API_JWT_SECRET (or SUPAVISOR_API_JWT) 环境变量"),
        "error.staging_log_not_found" => Some("未找到临时日志"),
        "error.staging_log_processed" => Some("日志已处理"),
        "error.staging_log_conflict" => Some("日志正在被另一个请求处理"),
        "error.login_locked" => Some("登录尝试次数过多。您的帐户已被临时锁定。请稍后再试。"),
        "error.holiday_not_found" => Some("未找到节假日"),
        "error.holiday_already_exists" => Some("该日期已存在节假日"),
        "error.job_not_found" => Some("未找到任务"),
        "error.error_file_not_found" => Some("未找到错误文件或已过期"),
        "error.workspace_not_found" => Some("未找到工作空间"),
        "error.cannot_remove_self" => Some("您不能将自己从工作空间中移除"),
        "error.role_not_found" => Some("未找到该角色"),
        "error.role_not_found_or_system" => Some("未找到角色或该角色为系统角色"),
        "error.expiration_future" => Some("过期时间必须在未来"),
        "error.unknown_permission" => {
            Some("未知权限。请使用 GET /permissions/available 获取有效值。")
        }
        "error.employee_assignment_not_found" => Some("未找到员工外派记录"),
        "error.home_host_location_same" => Some("居住地和工作地不能相同"),
        "error.contract_not_found" => Some("未找到合同"),
        "error.leave_type_not_found" => Some("未找到请假类型"),
        "error.policy_not_found" => Some("未找到政策"),
        "error.attendance_summary_not_found" => Some("未找到每日考勤汇总"),
        "error.invalid_status" => Some("无效状态"),
        "error.overtime_request_not_found" => Some("未找到加班申请"),
        "error.overtime_request_no_employees" => Some("加班申请未分配员工"),
        "error.invalid_start_date_format" => Some("无效的 start_date 格式 (YYYY-MM-DD)"),
        "error.invalid_end_date_format" => Some("无效的 end_date 格式 (YYYY-MM-DD)"),
        "error.invalid_year_month" => Some("无效的年份或月份格式"),
        "error.invalid_date_range" => Some("无效的 date range"),
        "error.invalid_date_format" => Some("无效的日期格式 (YYYY-MM-DD)"),
        "error.no_error_file_available" => Some("此导入无可用错误文件"),
        "error.workflow_not_found" => Some("未找到审批流"),
        "error.workflow_instance_not_found" => Some("未找到该实体的审批实例"),
        "error.invalid_feature_flag_key" => Some("flag_key 长度必须在 1 到 255 个字符之间"),
        "error.feature_flag_not_found" => Some("未找到 feature flag"),
        "error.contact_not_found" => Some("未找到联系人"),
        "error.minutes_greater_than_zero" => Some("分钟数必须大于 0"),
        "error.multipart_error" => Some("Multipart 表单解析失败"),
        "error.failed_read_file_bytes" => Some("无法读取上传的文件字节"),
        "error.file_too_large" => Some("文件大小超过限制"),
        "error.no_file_uploaded" => Some("未上传文件"),
        "error.invalid_utf8_csv" => Some("CSV 文件必须是 UTF-8 编码"),
        "error.csv_parsing_error" => Some("CSV 解析失败"),
        "error.max_rows_exceeded" => Some("超过最大允许行数"),
        "error.system_role_not_found" => Some("未找到系统角色"),
        "error.unknown_system_role" => Some("未知系统角色"),
        "error.role_name_required" => Some("角色名称是必填项"),
        "error.system_user_not_found" => Some("未找到系统用户"),
        "error.password_too_short" => Some("密码必须至少为 8 个字符"),
        "error.password_too_long" => Some("密码不能超过 128 个字符"),
        "error.password_complexity" => Some("密码必须包含至少一个数字或特殊字符"),
        "error.invalid_email_format" => Some("电子邮件格式无效"),
        "error.full_name_required" => Some("姓名是必填项"),
        "error.email_already_exists" => Some("电子邮件已存在"),
        "error.invalid_email_or_password" => Some("无效的电子邮件或密码"),
        "error.no_refresh_token" => Some("未提供 refresh token"),
        "error.current_password_incorrect" => Some("当前密码不正确"),
        "error.no_active_employees_found" => Some("未找到报告的活动员工"),
        "error.cannot_cancel_job" => Some("无法取消已完成或失败的任务"),
        "error.no_report_file_generated" => Some("尚未生成报告文件"),
        "error.device_not_found" => Some("未找到设备"),
        "error.agent_not_found" => Some("未找到 Agent"),
        "error.no_fields_to_update" => Some("没有要更新的字段"),
        "error.invalid_group" => Some("无效的组"),
        "error.invalid_tenant_id_format" => Some("无效的租户 ID 格式"),
        "error.invalid_scope_type" => Some("无效的范围类型"),
        "error.location_not_found" => Some("未找到地点"),
        "error.employer_not_found" => Some("未找到雇主"),
        "error.recruitment_source_not_found" => Some("未找到招聘来源"),
        "error.shift_not_found" => Some("未找到班次"),
        "error.shift_assignment_already_exists" => Some("该日期已存在班次分配"),
        "error.employee_phase_not_found" => Some("未找到员工阶段"),
        "error.invalid_base_group" => Some("无效的基本组"),
        "error.invalid_token" => Some("无效或过期的令牌"),
        "error.overflow_calculation" => Some("计算溢出"),
        "error.user_already_linked" => Some("该用户已在此工作空间中与员工 {name} 关联"),
        _ => None,
    }
}

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
    fn test_i18n_keys_match_all_keys() {
        use std::collections::HashSet;

        let all_keys_set: HashSet<&str> = ALL_KEYS.iter().copied().collect();

        let variants = vec![
            I18nKey::Unauthorized,
            I18nKey::AuthenticationRequired,
            I18nKey::Forbidden,
            I18nKey::NotFound,
            I18nKey::Validation,
            I18nKey::Internal,
            I18nKey::Conflict,
            I18nKey::ServiceUnavailable,
            I18nKey::MissingWorkspace,
            I18nKey::InvalidWorkspaceId,
            I18nKey::NoEmployeeRecord,
            I18nKey::SystemRouteTenantContext,
            I18nKey::NotWorkspaceMember,
            I18nKey::UserNotFound,
            I18nKey::EmployeeNotFound,
            I18nKey::EmploymentNotFound,
            I18nKey::DepartmentNotFound,
            I18nKey::DatabaseError,
            I18nKey::CannotRevokeLastAdmin,
            I18nKey::BehalfAssignmentMissingTarget,
            I18nKey::BehalfAssignmentDualTarget,
            I18nKey::UploadMultipart,
            I18nKey::UploadReadFailed,
            I18nKey::UploadFileTooLarge,
            I18nKey::UploadNoFile,
            I18nKey::TenantMismatch,
            I18nKey::SupavisorApiUrlRequired,
            I18nKey::ApiJwtSecretRequired,
            I18nKey::StagingLogNotFound,
            I18nKey::StagingLogProcessed,
            I18nKey::StagingLogConflict,
            I18nKey::LoginLocked,
            I18nKey::HolidayNotFound,
            I18nKey::HolidayAlreadyExists,
            I18nKey::JobNotFound,
            I18nKey::ErrorFileNotFound,
            I18nKey::WorkspaceNotFound,
            I18nKey::CannotRemoveSelf,
            I18nKey::RoleNotFound,
            I18nKey::RoleNotFoundOrSystem,
            I18nKey::ExpirationFuture,
            I18nKey::UnknownPermission,
            I18nKey::EmployeeAssignmentNotFound,
            I18nKey::HomeHostLocationSame,
            I18nKey::ContractNotFound,
            I18nKey::LeaveTypeNotFound,
            I18nKey::PolicyNotFound,
            I18nKey::AttendanceSummaryNotFound,
            I18nKey::InvalidStatus,
            I18nKey::OvertimeRequestNotFound,
            I18nKey::OvertimeRequestNoEmployees,
            I18nKey::InvalidStartDateValue,
            I18nKey::InvalidEndDateValue,
            I18nKey::InvalidYearMonth,
            I18nKey::InvalidDateRange,
            I18nKey::InvalidDateFormat,
            I18nKey::NoErrorFileAvailable,
            I18nKey::WorkflowNotFound,
            I18nKey::WorkflowInstanceNotFound,
            I18nKey::InvalidFeatureFlagKey,
            I18nKey::FeatureFlagNotFound,
            I18nKey::ContactNotFound,
            I18nKey::MinutesGreaterThanZero,
            I18nKey::MultipartError,
            I18nKey::FailedToReadFileBytes,
            I18nKey::FileTooLarge,
            I18nKey::NoFileUploaded,
            I18nKey::InvalidUtf8Csv,
            I18nKey::CsvParsingError,
            I18nKey::MaxRowsExceeded,
            I18nKey::SystemRoleNotFound,
            I18nKey::UnknownSystemRole,
            I18nKey::RoleNameRequired,
            I18nKey::SystemUserNotFound,
            I18nKey::PasswordTooShort,
            I18nKey::PasswordTooLong,
            I18nKey::PasswordComplexity,
            I18nKey::InvalidEmailFormat,
            I18nKey::FullNameRequired,
            I18nKey::EmailAlreadyExists,
            I18nKey::InvalidEmailOrPassword,
            I18nKey::NoRefreshToken,
            I18nKey::CurrentPasswordIncorrect,
            I18nKey::NoActiveEmployeesFound,
            I18nKey::CannotCancelJob,
            I18nKey::NoReportFileGenerated,
            I18nKey::DeviceNotFound,
            I18nKey::AgentNotFound,
            I18nKey::NoFieldsToUpdate,
            I18nKey::InvalidGroup,
            I18nKey::InvalidTenantIdFormat,
            I18nKey::InvalidScopeType,
            I18nKey::LocationNotFound,
            I18nKey::EmployerNotFound,
            I18nKey::RecruitmentSourceNotFound,
            I18nKey::ShiftNotFound,
            I18nKey::ShiftAssignmentAlreadyExists,
            I18nKey::EmployeePhaseNotFound,
            I18nKey::InvalidBaseGroup,
            I18nKey::InvalidToken,
            I18nKey::OverflowCalculation,
            I18nKey::UserAlreadyLinked,
        ];

        for variant in &variants {
            let key = variant.as_str();
            assert!(
                all_keys_set.contains(key),
                "I18nKey variant maps to key '{key}' which is missing from ALL_KEYS"
            );
        }

        assert_eq!(
            variants.len(),
            ALL_KEYS.len(),
            "Number of I18nKey variants ({}) does not match ALL_KEYS ({})",
            variants.len(),
            ALL_KEYS.len()
        );
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

        const LEGACY_ALLOWLIST: &[&str] = &[
            "time_tracking_service.rs",
            "payroll_service.rs",
            "shift_swap_service.rs",
            "employee_service.rs",
            "leave_service.rs",
            "attendance_summary_service.rs",
        ];

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
            if LEGACY_ALLOWLIST.contains(&file_name) {
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
