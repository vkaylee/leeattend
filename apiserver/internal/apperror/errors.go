package apperror

import (
	"errors"
	"fmt"
	"net/http"
	"strings"
)

// ErrorCode is the canonical wire error code string matching Rust AppError.
type ErrorCode string

const (
	CodeDatabaseError      ErrorCode = "DATABASE_ERROR"
	CodeRedisError         ErrorCode = "REDIS_ERROR"
	CodeUnauthorized       ErrorCode = "UNAUTHORIZED"
	CodeForbidden          ErrorCode = "FORBIDDEN"
	CodeNotFound           ErrorCode = "NOT_FOUND"
	CodeValidationError    ErrorCode = "VALIDATION_ERROR"
	CodePayloadTooLarge    ErrorCode = "PAYLOAD_TOO_LARGE"
	CodeInternalError      ErrorCode = "INTERNAL_ERROR"
	CodeConflict           ErrorCode = "CONFLICT"
	CodeServiceUnavailable ErrorCode = "SERVICE_UNAVAILABLE"
	CodeBackgroundJobError ErrorCode = "BACKGROUND_JOB_ERROR"
	CodeTemplateError      ErrorCode = "TEMPLATE_ERROR"
	CodeRateLimited        ErrorCode = "RATE_LIMITED"
	CodeTooManyRequests    ErrorCode = "TOO_MANY_REQUESTS"
)

// DefaultStatus maps canonical error codes to standard HTTP status codes.
func (c ErrorCode) DefaultStatus() int {
	switch c {
	case CodeDatabaseError, CodeRedisError, CodeInternalError, CodeBackgroundJobError, CodeTemplateError:
		return http.StatusInternalServerError
	case CodeUnauthorized:
		return http.StatusUnauthorized
	case CodeForbidden:
		return http.StatusForbidden
	case CodeNotFound:
		return http.StatusNotFound
	case CodeValidationError:
		return http.StatusBadRequest
	case CodePayloadTooLarge:
		return http.StatusRequestEntityTooLarge
	case CodeConflict:
		return http.StatusConflict
	case CodeServiceUnavailable:
		return http.StatusServiceUnavailable
	case CodeRateLimited, CodeTooManyRequests:
		return http.StatusTooManyRequests
	default:
		return http.StatusInternalServerError
	}
}

// Canonical translatable I18nKey constants cataloged in error-contract.md.
const (
	KeyInternal                   = "error.internal"
	KeyUnauthorized               = "error.unauthorized"
	KeyAuthenticationRequired     = "error.authentication_required"
	KeyForbidden                  = "error.forbidden"
	KeyNotFound                   = "error.not_found"
	KeyValidation                 = "error.validation"
	KeyConflict                   = "error.conflict"
	KeyServiceUnavailable         = "error.service_unavailable"
	KeyMissingWorkspace           = "error.missing_workspace"
	KeyInvalidWorkspaceId         = "error.invalid_workspace_id"
	KeyNoEmployeeRecord           = "error.no_employee_record"
	KeySystemRouteTenantContext   = "error.system_route_tenant_context"
	KeyNotWorkspaceMember         = "error.not_workspace_member"
	KeyUserNotFound               = "error.user_not_found"
	KeyEmployeeNotFound           = "error.employee_not_found"
	KeyEmploymentNotFound         = "error.employment_not_found"
	KeyDepartmentNotFound         = "error.department_not_found"
	KeyDatabaseError              = "error.database_error"
	KeyUploadFileTooLarge         = "error.upload_file_too_large"
	KeyFileTooLarge               = "error.file_too_large"
	KeyLoginLocked                = "error.login_locked"
	KeyHolidayNotFound            = "error.holiday_not_found"
	KeyHolidayAlreadyExists       = "error.holiday_already_exists"
	KeyLeaveTypeNotFound          = "error.leave_type_not_found"
	KeyJobNotFound                = "error.job_not_found"
	KeyCannotCancelJob            = "error.cannot_cancel_job"
	KeyNoReportFileGenerated      = "error.no_report_file_generated"
	KeyErrorFileNotFound          = "error.error_file_not_found"
	KeyWorkspaceNotFound          = "error.workspace_not_found"
	KeyRoleNotFound               = "error.role_not_found"
	KeyUnknownPermission          = "error.unknown_permission"
	KeyRoleNotFoundOrSystem       = "error.role_not_found_or_system"
	KeyCannotRemoveSelf           = "error.cannot_remove_self"
	KeyExpirationFuture           = "error.expiration_future"
	KeyCannotRevokeLastAdmin      = "error.cannot_revoke_last_admin"
	KeySystemRoleNotFound         = "error.system_role_not_found"
	KeyUnknownSystemRole          = "error.unknown_system_role"
	KeyInvalidTimezone            = "error.invalid_timezone"
	KeyInvalidDateFormat          = "error.invalid_date_format"
	KeyInvalidStartDate           = "error.invalid_start_date_format"
	KeyInvalidEndDate             = "error.invalid_end_date_format"
	KeyInvalidYearMonth           = "error.invalid_year_month"
	KeyInvalidDateRange           = "error.invalid_date_range"
	KeyInvalidEmailFormat         = "error.invalid_email_format"
	KeyPasswordTooShort           = "error.password_too_short"
	KeyPasswordTooLong            = "error.password_too_long"
	KeyPasswordComplexity         = "error.password_complexity"
	KeyCurrentPasswordIncorrect   = "error.current_password_incorrect"
	KeyFullNameRequired           = "error.full_name_required"
	KeyEmailAlreadyExists         = "error.email_already_exists"
	KeyInvalidEmailOrPassword     = "error.invalid_email_or_password"
	KeyNoRefreshToken             = "error.no_refresh_token"
	KeyInvalidFeatureFlagKey      = "error.invalid_feature_flag_key"
	KeyFeatureFlagNotFound        = "error.feature_flag_not_found"
	KeyShiftNotFound              = "error.shift_not_found"
	KeyInvalidToken               = "error.invalid_token"
	KeyResetTokenInvalid          = "error.reset_token_invalid"
	KeyResetTokenExpired          = "error.reset_token_expired"
	KeyShiftOverlapDetected       = "error.shift_overlap_detected"
	KeyInsufficientRestPeriod     = "error.insufficient_rest_period"
	KeyWeeklyHoursExceeded        = "error.weekly_hours_exceeded"
	KeyMonthlyOTExceeded          = "error.monthly_ot_exceeded"
	KeyComplianceOverrideRequired = "error.compliance_override_required"
	KeyInvalidBaseGroup           = "error.invalid_base_group"
)

// AppError is the canonical application error type matching Rust AppError.
type AppError struct {
	Code        ErrorCode         `json:"-"`
	Status      int               `json:"-"`
	I18nKey     string            `json:"-"`
	Args        map[string]string `json:"-"`
	Retry       any               `json:"-"`
	InternalErr error             `json:"-"`
}

func (e *AppError) Error() string {
	if e == nil {
		return ""
	}
	if e.InternalErr != nil {
		return fmt.Sprintf("[%s:%d] %s: %v", e.Code, e.Status, e.I18nKey, e.InternalErr)
	}
	return fmt.Sprintf("[%s:%d] %s", e.Code, e.Status, e.I18nKey)
}

func (e *AppError) Unwrap() error {
	if e == nil {
		return nil
	}
	return e.InternalErr
}

// Is reports whether target matches this AppError's code.
func (e *AppError) Is(target error) bool {
	if target == nil || e == nil {
		return e == target
	}
	var targetAppErr *AppError
	if errors.As(target, &targetAppErr) {
		return e.Code == targetAppErr.Code
	}
	return false
}

// IsInternal returns true for 500 family errors that require wire message masking.
func (e *AppError) IsInternal() bool {
	if e == nil {
		return false
	}
	switch e.Code {
	case CodeDatabaseError, CodeRedisError, CodeInternalError, CodeBackgroundJobError, CodeTemplateError:
		return true
	default:
		return e.Status == http.StatusInternalServerError
	}
}

// Translate resolves the exposed client message respecting i18n and internal redaction.
func (e *AppError) Translate(lang string) string {
	if e == nil {
		return ""
	}
	// Inviolable Rule: 500 internal errors are unconditionally masked to error.internal
	if e.IsInternal() {
		return Translate(lang, KeyInternal)
	}
	if len(e.Args) > 0 {
		return FormatMessage(lang, e.I18nKey, e.Args)
	}
	return Translate(lang, e.I18nKey)
}

// Constructors

func New(code ErrorCode, status int, key string) *AppError {
	return &AppError{
		Code:    code,
		Status:  status,
		I18nKey: key,
	}
}

func NewWithArgs(code ErrorCode, status int, key string, args map[string]string) *AppError {
	return &AppError{
		Code:    code,
		Status:  status,
		I18nKey: key,
		Args:    args,
	}
}

func Unauthorized(key string) *AppError {
	if key == "" {
		key = KeyUnauthorized
	}
	return New(CodeUnauthorized, http.StatusUnauthorized, key)
}

func UnauthorizedArgs(key string, args map[string]string) *AppError {
	if key == "" {
		key = KeyUnauthorized
	}
	return NewWithArgs(CodeUnauthorized, http.StatusUnauthorized, key, args)
}

func Forbidden(key string) *AppError {
	if key == "" {
		key = KeyForbidden
	}
	return New(CodeForbidden, http.StatusForbidden, key)
}

func ForbiddenArgs(key string, args map[string]string) *AppError {
	if key == "" {
		key = KeyForbidden
	}
	return NewWithArgs(CodeForbidden, http.StatusForbidden, key, args)
}

func NotFound(key string) *AppError {
	if key == "" {
		key = KeyNotFound
	}
	return New(CodeNotFound, http.StatusNotFound, key)
}

func NotFoundArgs(key string, args map[string]string) *AppError {
	if key == "" {
		key = KeyNotFound
	}
	return NewWithArgs(CodeNotFound, http.StatusNotFound, key, args)
}

func Validation(key string) *AppError {
	if key == "" {
		key = KeyValidation
	}
	return New(CodeValidationError, http.StatusBadRequest, key)
}

func ValidationArgs(key string, args map[string]string) *AppError {
	if key == "" {
		key = KeyValidation
	}
	return NewWithArgs(CodeValidationError, http.StatusBadRequest, key, args)
}

func ValidationDetails(details string) *AppError {
	return ValidationArgs(KeyValidation, map[string]string{"details": details})
}

func PayloadTooLarge(key string) *AppError {
	if key == "" {
		key = KeyFileTooLarge
	}
	return New(CodePayloadTooLarge, http.StatusRequestEntityTooLarge, key)
}

func PayloadTooLargeArgs(key string, args map[string]string) *AppError {
	if key == "" {
		key = KeyFileTooLarge
	}
	return NewWithArgs(CodePayloadTooLarge, http.StatusRequestEntityTooLarge, key, args)
}

func Conflict(key string) *AppError {
	if key == "" {
		key = KeyConflict
	}
	return New(CodeConflict, http.StatusConflict, key)
}

func ConflictArgs(key string, args map[string]string) *AppError {
	if key == "" {
		key = KeyConflict
	}
	return NewWithArgs(CodeConflict, http.StatusConflict, key, args)
}

func ConflictWithRetry(key string, args map[string]string, retry any) *AppError {
	err := ConflictArgs(key, args)
	err.Retry = retry
	return err
}

func ServiceUnavailable(key string) *AppError {
	if key == "" {
		key = KeyServiceUnavailable
	}
	return New(CodeServiceUnavailable, http.StatusServiceUnavailable, key)
}

func ServiceUnavailableArgs(key string, args map[string]string) *AppError {
	if key == "" {
		key = KeyServiceUnavailable
	}
	return NewWithArgs(CodeServiceUnavailable, http.StatusServiceUnavailable, key, args)
}

func Internal(err error) *AppError {
	return &AppError{
		Code:        CodeInternalError,
		Status:      http.StatusInternalServerError,
		I18nKey:     KeyInternal,
		InternalErr: err,
	}
}

func InternalString(msg string) *AppError {
	return Internal(errors.New(msg))
}

func Database(err error) *AppError {
	return &AppError{
		Code:        CodeDatabaseError,
		Status:      http.StatusInternalServerError,
		I18nKey:     KeyInternal,
		InternalErr: err,
	}
}

func Redis(err error) *AppError {
	return &AppError{
		Code:        CodeRedisError,
		Status:      http.StatusInternalServerError,
		I18nKey:     KeyInternal,
		InternalErr: err,
	}
}

func BackgroundJob(err error) *AppError {
	return &AppError{
		Code:        CodeBackgroundJobError,
		Status:      http.StatusInternalServerError,
		I18nKey:     KeyInternal,
		InternalErr: err,
	}
}

func Template(err error) *AppError {
	return &AppError{
		Code:        CodeTemplateError,
		Status:      http.StatusInternalServerError,
		I18nKey:     KeyInternal,
		InternalErr: err,
	}
}

// ponytail: in-memory translation table matches 152 static keys in Rust utils/i18n.rs. Upgrade to dynamic catalog if runtime localization bundles needed.
type translationEntry struct {
	en string
	vi string
	zh string
}

func ResolveLanguage(s string) string {
	switch strings.ToLower(strings.TrimSpace(s)) {
	case "en", "en-us", "english":
		return "en"
	case "zh", "zh-cn", "chinese":
		return "zh"
	case "vi", "vi-vn", "vietnamese":
		return "vi"
	default:
		return "vi"
	}
}

func Translate(lang, key string) string {
	entry, ok := i18nCatalog[key]
	if !ok {
		return key
	}
	switch ResolveLanguage(lang) {
	case "en":
		if entry.en != "" {
			return entry.en
		}
	case "zh":
		if entry.zh != "" {
			return entry.zh
		}
	case "vi":
		if entry.vi != "" {
			return entry.vi
		}
	}
	// Fallback chain: requested -> vi -> en -> raw key
	if entry.vi != "" {
		return entry.vi
	}
	if entry.en != "" {
		return entry.en
	}
	return key
}

func FormatMessage(lang, key string, args map[string]string) string {
	prefix := Translate(lang, key)
	if details, ok := args["details"]; ok {
		if prefix == key || prefix == "" {
			return details
		}
		return fmt.Sprintf("%s: %s", prefix, details)
	}
	text := prefix
	for k, v := range args {
		text = strings.ReplaceAll(text, "{"+k+"}", v)
	}
	return text
}

var i18nCatalog = map[string]translationEntry{
	"error.unauthorized": {
		en: "Authentication required",
		vi: "Yêu cầu đăng nhập",
		zh: "需要身份验证",
	},
	"error.authentication_required": {
		en: "Authentication required",
		vi: "Yêu cầu đăng nhập",
		zh: "需要身份验证",
	},
	"error.forbidden": {
		en: "Permission denied",
		vi: "Không có quyền truy cập",
		zh: "权限拒绝",
	},
	"error.not_found": {
		en: "Resource not found",
		vi: "Không tìm thấy tài nguyên",
		zh: "未找到资源",
	},
	"error.validation": {
		en: "Validation failed",
		vi: "Lỗi kiểm tra dữ liệu",
		zh: "数据验证失败",
	},
	"error.internal": {
		en: "An internal server error occurred.",
		vi: "Đã xảy ra lỗi hệ thống.",
		zh: "发生系统内部错误。",
	},
	"error.conflict": {
		en: "A conflict occurred.",
		vi: "Đã xảy ra xung đột dữ liệu.",
		zh: "数据冲突。",
	},
	"error.service_unavailable": {
		en: "Service unavailable.",
		vi: "Dịch vụ hiện không khả dụng.",
		zh: "服务暂不可用。",
	},
	"error.missing_workspace": {
		en: "Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header.",
		vi: "Thiếu thông tin Workspace. Vui lòng truyền 'ws_id' trong URL hoặc header 'X-Workspace-Id'.",
		zh: "缺少工作空间上下文。请在路径中提供 'ws_id' 或在 'X-Workspace-Id' 请求头中提供。",
	},
	"error.invalid_workspace_id": {
		en: "Invalid Workspace ID format",
		vi: "Định dạng ID Workspace không hợp lệ",
		zh: "工作空间 ID 格式无效",
	},
	"error.no_employee_record": {
		en: "No employee record linked to your account",
		vi: "Không có thông tin nhân viên nào liên kết với tài khoản của bạn",
		zh: "您的账户未关联任何员工档案",
	},
	"error.system_route_tenant_context": {
		en: "Cannot use Tenant context (MemberContext) in a System route",
		vi: "Không thể sử dụng context Tenant (MemberContext) trong route hệ thống",
		zh: "系统路由中无法使用租户上下文 (MemberContext)",
	},
	"error.not_workspace_member": {
		en: "Not a member of this workspace",
		vi: "Bạn không phải là thành viên của workspace này",
		zh: "您不是该工作空间的成员",
	},
	"error.user_not_found": {
		en: "User not found",
		vi: "Không tìm thấy người dùng",
		zh: "未找到该用户",
	},
	"error.employee_not_found": {
		en: "Employee not found",
		vi: "Không tìm thấy nhân viên",
		zh: "未找到员工",
	},
	"error.employment_not_found": {
		en: "Employment not found",
		vi: "Không tìm thấy quan hệ công việc",
		zh: "未找到工作关系",
	},
	"error.department_not_found": {
		en: "Department not found",
		vi: "Không tìm thấy phòng ban",
		zh: "未找到部门",
	},
	"error.database_error": {
		en: "Database error",
		vi: "Lỗi cơ sở dữ liệu",
		zh: "数据库错误",
	},
	"error.cannot_revoke_last_admin": {
		en: "Cannot revoke the last platform_admin. Assign another admin first.",
		vi: "Không thể thu hồi platform_admin cuối cùng. Hãy gán quản trị viên khác trước.",
		zh: "无法撤销最后一个 platform_admin。请先分配另一个管理员。",
	},
	"error.behalf_assignment_missing_target": {
		en: "Either target employee or target department must be specified",
		vi: "Phải chỉ định nhân viên đích hoặc phòng ban đích",
		zh: "必须指定目标员工或目标部门",
	},
	"error.behalf_assignment_dual_target": {
		en: "Cannot specify both target employee and target department",
		vi: "Không thể chỉ định cả nhân viên đích và phòng ban đích",
		zh: "不能同时指定目标员工和目标部门",
	},
	"error.upload_multipart": {
		en: "Multipart form upload error",
		vi: "Lỗi tải lên dữ liệu dạng multipart",
		zh: "多部分表单上传错误",
	},
	"error.upload_read_failed": {
		en: "Failed to read uploaded file bytes",
		vi: "Không thể đọc dữ liệu file tải lên",
		zh: "无法读取上传的文件字节",
	},
	"error.upload_file_too_large": {
		en: "Uploaded file is too large",
		vi: "File tải lên vượt quá dung lượng cho phép",
		zh: "上传的文件太大",
	},
	"error.upload_no_file": {
		en: "No file was uploaded",
		vi: "Không có file nào được tải lên",
		zh: "未上传文件",
	},
	"error.tenant_mismatch": {
		en: "Tenant mismatch",
		vi: "Không khớp tenant",
		zh: "租户不匹配",
	},
	"error.supavisor_api_url_required": {
		en: "SUPAVISOR_API_URL environment variable is required",
		vi: "Yêu cầu biến môi trường SUPAVISOR_API_URL",
		zh: "需要 SUPAVISOR_API_URL 环境变量",
	},
	"error.api_jwt_secret_required": {
		en: "API_JWT_SECRET (or SUPAVISOR_API_JWT) environment variable is required",
		vi: "Yêu cầu biến môi trường API_JWT_SECRET (hoặc SUPAVISOR_API_JWT)",
		zh: "API_JWT_SECRET (or SUPAVISOR_API_JWT) 环境变量",
	},
	"error.staging_log_not_found": {
		en: "Staging log not found",
		vi: "Không tìm thấy log tạm thời",
		zh: "未找到临时日志",
	},
	"error.staging_log_processed": {
		en: "Log already processed",
		vi: "Log đã được xử lý",
		zh: "日志已处理",
	},
	"error.staging_log_conflict": {
		en: "Log is being processed by another request",
		vi: "Log đang được xử lý bởi yêu cầu khác",
		zh: "日志正在被另一个请求处理",
	},
	"error.login_locked": {
		en: "Too many login attempts. Your account has been temporarily locked. Please try again later.",
		vi: "Quá nhiều lần đăng nhập sai. Tài khoản của bạn đã bị khóa tạm thời. Vui lòng thử lại sau.",
		zh: "登录尝试次数过多。您的帐户已被临时锁定。请稍后再试。",
	},
	"error.holiday_not_found": {
		en: "Holiday not found",
		vi: "Không tìm thấy ngày nghỉ lễ",
		zh: "未找到节假日",
	},
	"error.holiday_already_exists": {
		en: "A holiday already exists on this date",
		vi: "Đã tồn tại ngày nghỉ lễ vào ngày này",
		zh: "该日期已存在节假日",
	},
	"error.job_not_found": {
		en: "Job not found",
		vi: "Không tìm thấy tiến trình",
		zh: "未找到任务",
	},
	"error.error_file_not_found": {
		en: "Error file not found or has expired",
		vi: "Không tìm thấy file lỗi hoặc đã hết hạn",
		zh: "未找到错误文件或已过期",
	},
	"error.workspace_not_found": {
		en: "Workspace not found",
		vi: "Không tìm thấy Workspace",
		zh: "未找到工作空间",
	},
	"error.cannot_remove_self": {
		en: "You cannot remove yourself from the workspace",
		vi: "Bạn không thể tự xóa bản thân khỏi workspace",
		zh: "您不能将自己从工作空间中移除",
	},
	"error.role_not_found": {
		en: "Role not found",
		vi: "Không tìm thấy vai trò",
		zh: "未找到该角色",
	},
	"error.role_not_found_or_system": {
		en: "Role not found or is a system role",
		vi: "Không tìm thấy vai trò hoặc đây là vai trò hệ thống",
		zh: "未找到角色或该角色为系统角色",
	},
	"error.expiration_future": {
		en: "Expiration time must be in the future",
		vi: "Thời gian hết hạn phải ở tương lai",
		zh: "过期时间必须在未来",
	},
	"error.unknown_permission": {
		en: "Unknown permission. Use GET /permissions/available for valid values.",
		vi: "Quyền không hợp lệ. Sử dụng GET /permissions/available để lấy các giá trị hợp lệ.",
		zh: "未知权限。请使用 GET /permissions/available 获取有效值。",
	},
	"error.employee_assignment_not_found": {
		en: "Employee assignment not found",
		vi: "Không tìm thấy điều động nhân viên",
		zh: "未找到员工外派记录",
	},
	"error.home_host_location_same": {
		en: "Home location and host location cannot be the same",
		vi: "Địa điểm nhà và địa điểm host không được giống nhau",
		zh: "居住地和工作地不能相同",
	},
	"error.contract_not_found": {
		en: "Contract not found",
		vi: "Không tìm thấy hợp đồng",
		zh: "未找到合同",
	},
	"error.leave_type_not_found": {
		en: "Leave type not found",
		vi: "Không tìm thấy loại nghỉ phép",
		zh: "未找到请假类型",
	},
	"error.policy_not_found": {
		en: "Policy not found",
		vi: "Không tìm thấy chính sách",
		zh: "未找到政策",
	},
	"error.attendance_summary_not_found": {
		en: "Attendance daily summary not found",
		vi: "Không tìm thấy bảng tổng hợp chấm công hàng ngày",
		zh: "未找到每日考勤汇总",
	},
	"error.invalid_status": {
		en: "Invalid status",
		vi: "Trạng thái không hợp lệ",
		zh: "无效状态",
	},
	"error.overtime_request_not_found": {
		en: "Overtime request not found",
		vi: "Không tìm thấy yêu cầu tăng ca",
		zh: "未找到加班申请",
	},
	"error.overtime_request_no_employees": {
		en: "Overtime request has no employees assigned",
		vi: "Yêu cầu tăng ca chưa được gán nhân viên nào",
		zh: "加班申请未分配员工",
	},
	"error.invalid_start_date_format": {
		en: "Invalid start_date format (YYYY-MM-DD)",
		vi: "Định dạng start_date không hợp lệ (YYYY-MM-DD)",
		zh: "无效的 start_date 格式 (YYYY-MM-DD)",
	},
	"error.invalid_end_date_format": {
		en: "Invalid end_date format (YYYY-MM-DD)",
		vi: "Định dạng end_date không hợp lệ (YYYY-MM-DD)",
		zh: "无效的 end_date 格式 (YYYY-MM-DD)",
	},
	"error.invalid_year_month": {
		en: "Invalid year or month format",
		vi: "Định dạng năm hoặc tháng không hợp lệ",
		zh: "无效的年份或月份格式",
	},
	"error.invalid_date_range": {
		en: "Invalid date range",
		vi: "Khoảng ngày không hợp lệ",
		zh: "无效的 date range",
	},
	"error.invalid_date_format": {
		en: "Invalid date format (YYYY-MM-DD)",
		vi: "Định dạng ngày không hợp lệ (YYYY-MM-DD)",
		zh: "无效的日期格式 (YYYY-MM-DD)",
	},
	"error.invalid_timezone": {
		en: "Invalid timezone. Expected an IANA name such as 'Asia/Ho_Chi_Minh'",
		vi: "Múi giờ không hợp lệ. Cần tên IANA, ví dụ 'Asia/Ho_Chi_Minh'",
		zh: "无效的时区。需要 IANA 名称，例如 'Asia/Ho_Chi_Minh'",
	},
	"error.no_error_file_available": {
		en: "No error file available for this import",
		vi: "Không có file lỗi nào khả dụng cho lượt nhập này",
		zh: "此导入无可用错误文件",
	},
	"error.workflow_not_found": {
		en: "Workflow not found",
		vi: "Không tìm thấy quy trình phê duyệt",
		zh: "未找到审批流",
	},
	"error.workflow_instance_not_found": {
		en: "Workflow instance not found for this entity",
		vi: "Không tìm thấy thực thể quy trình phê duyệt cho đối tượng này",
		zh: "未找到该实体的审批实例",
	},
	"error.invalid_feature_flag_key": {
		en: "flag_key must be between 1 and 255 characters",
		vi: "flag_key phải nằm trong khoảng từ 1 đến 255 ký tự",
		zh: "flag_key 长度必须在 1 到 255 个字符之间",
	},
	"error.feature_flag_not_found": {
		en: "Feature flag not found",
		vi: "Không tìm thấy feature flag",
		zh: "未找到 feature flag",
	},
	"error.contact_not_found": {
		en: "Contact not found",
		vi: "Không tìm thấy thông tin liên hệ",
		zh: "未找到联系人",
	},
	"error.minutes_greater_than_zero": {
		en: "Minutes must be greater than 0",
		vi: "Số phút phải lớn hơn 0",
		zh: "分钟数必须大于 0",
	},
	"error.multipart_error": {
		en: "Multipart form parsing failed",
		vi: "Lỗi phân tích cú pháp multipart form",
		zh: "Multipart 表单解析失败",
	},
	"error.failed_read_file_bytes": {
		en: "Failed to read uploaded file bytes",
		vi: "Không thể đọc dữ liệu file tải lên",
		zh: "无法读取上传的文件字节",
	},
	"error.file_too_large": {
		en: "File size exceeds the limit",
		vi: "File tải lên vượt quá dung lượng cho phép",
		zh: "文件大小超过限制",
	},
	"error.no_file_uploaded": {
		en: "No file uploaded",
		vi: "Không có file nào được tải lên",
		zh: "未上传文件",
	},
	"error.invalid_file_format": {
		en: "Invalid file format. Only Excel files (.xlsx, .xls) are supported.",
		vi: "Định dạng file không hỗ trợ. Chỉ hỗ trợ file Excel (.xlsx, .xls).",
		zh: "不支持的文件格式。仅支持 Excel 文件 (.xlsx, .xls)。",
	},
	"error.invalid_utf8_csv": {
		en: "CSV file must be encoded in UTF-8",
		vi: "File CSV phải được mã hóa UTF-8",
		zh: "CSV 文件必须是 UTF-8 编码",
	},
	"error.csv_parsing_error": {
		en: "CSV parsing failed",
		vi: "Lỗi parse CSV",
		zh: "CSV 解析失败",
	},
	"error.max_rows_exceeded": {
		en: "Maximum allowed rows exceeded",
		vi: "Vượt quá số dòng tối đa cho phép",
		zh: "超过最大允许行数",
	},
	"error.roster_batch_too_large": {
		en: "Too many days selected in one request. Please select fewer days.",
		vi: "Chọn quá nhiều ngày trong một yêu cầu. Vui lòng chọn ít ngày hơn.",
		zh: "单次请求选择的日期过多。请减少选择的日期数量。",
	},
	"error.system_role_not_found": {
		en: "System role not found",
		vi: "Không tìm thấy vai trò hệ thống",
		zh: "未找到系统角色",
	},
	"error.unknown_system_role": {
		en: "Unknown system role",
		vi: "Vai trò hệ thống không xác định",
		zh: "未知系统角色",
	},
	"error.role_name_required": {
		en: "Role name is required",
		vi: "Tên vai trò là bắt buộc",
		zh: "角色名称是必填项",
	},
	"error.system_user_not_found": {
		en: "System user not found",
		vi: "Không tìm thấy người dùng hệ thống",
		zh: "未找到系统用户",
	},
	"error.password_too_short": {
		en: "Password must be at least 8 characters",
		vi: "Mật khẩu phải có ít nhất 8 ký tự",
		zh: "密码必须至少为 8 个字符",
	},
	"error.password_too_long": {
		en: "Password must not exceed 128 characters",
		vi: "Mật khẩu không được vượt quá 128 ký tự",
		zh: "密码不能超过 128 个字符",
	},
	"error.password_complexity": {
		en: "Password must contain at least one digit or special character",
		vi: "Mật khẩu phải chứa ít nhất một chữ số hoặc ký tự đặc biệt",
		zh: "密码必须包含至少一个数字或特殊字符",
	},
	"error.invalid_email_format": {
		en: "Invalid email format",
		vi: "Định dạng email không hợp lệ",
		zh: "电子邮件格式无效",
	},
	"error.full_name_required": {
		en: "Full name is required",
		vi: "Họ và tên là bắt buộc",
		zh: "姓名是必填项",
	},
	"error.email_already_exists": {
		en: "Email already exists",
		vi: "Email đã tồn tại",
		zh: "电子邮件已存在",
	},
	"error.invalid_email_or_password": {
		en: "Invalid email or password",
		vi: "Email hoặc mật khẩu không đúng",
		zh: "无效的电子邮件或密码",
	},
	"error.no_refresh_token": {
		en: "No refresh token provided",
		vi: "Không có refresh token nào được cung cấp",
		zh: "未提供 refresh token",
	},
	"error.current_password_incorrect": {
		en: "Current password is incorrect",
		vi: "Mật khẩu hiện tại không đúng",
		zh: "当前密码不正确",
	},
	"error.no_active_employees_found": {
		en: "No active employees found for report",
		vi: "Không tìm thấy nhân viên đang hoạt động nào cho báo cáo",
		zh: "未找到报告的活动员工",
	},
	"error.cannot_cancel_job": {
		en: "Cannot cancel completed or failed job",
		vi: "Không thể hủy công việc đã hoàn thành hoặc thất bại",
		zh: "无法取消已完成或失败的任务",
	},
	"error.no_report_file_generated": {
		en: "No report file generated yet",
		vi: "Chưa có file báo cáo nào được tạo",
		zh: "尚未生成报告文件",
	},
	"error.no_fields_to_update": {
		en: "No fields to update",
		vi: "Không có trường nào để cập nhật",
		zh: "没有要更新的字段",
	},
	"error.invalid_group": {
		en: "Invalid group",
		vi: "Nhóm không hợp lệ",
		zh: "无效的组",
	},
	"error.invalid_tenant_id_format": {
		en: "Invalid tenant ID format",
		vi: "Định dạng ID tenant không hợp lệ",
		zh: "无效的租户 ID 格式",
	},
	"error.invalid_scope_type": {
		en: "Invalid scope type",
		vi: "Phạm vi không hợp lệ",
		zh: "无效的范围类型",
	},
	"error.location_not_found": {
		en: "Location not found",
		vi: "Không tìm thấy địa điểm",
		zh: "未找到地点",
	},
	"error.employer_not_found": {
		en: "Employer not found",
		vi: "Không tìm thấy người sử dụng lao động",
		zh: "未找到雇主",
	},
	"error.recruitment_source_not_found": {
		en: "Recruitment source not found",
		vi: "Không tìm thấy nguồn tuyển dụng",
		zh: "未找到招聘来源",
	},
	"error.shift_not_found": {
		en: "Shift not found",
		vi: "Không tìm thấy ca làm việc",
		zh: "未找到班次",
	},
	"error.shift_assignment_already_exists": {
		en: "Shift assignment already exists for this date",
		vi: "Đã tồn tại phân công ca cho ngày này",
		zh: "该日期已存在班次分配",
	},
	"error.employee_phase_not_found": {
		en: "Employee phase not found",
		vi: "Không tìm thấy giai đoạn nhân viên",
		zh: "未找到员工阶段",
	},
	"error.invalid_base_group": {
		en: "Invalid base group",
		vi: "Nhóm cơ bản không hợp lệ",
		zh: "无效的基本组",
	},
	"error.invalid_token": {
		en: "Invalid or expired token",
		vi: "Token không hợp lệ hoặc đã hết hạn",
		zh: "无效或过期的令牌",
	},
	"error.overflow_calculation": {
		en: "Calculation overflow occurred",
		vi: "Xảy ra lỗi tràn số khi tính toán",
		zh: "计算溢出",
	},
	"error.user_already_linked": {
		en: "User is already linked to employee {name} in this workspace",
		vi: "Tài khoản đã được gán cho nhân viên {name} trong workspace này",
		zh: "该用户已在此工作空间中与员工 {name} 关联",
	},
	"error.employee_already_active": {
		en: "Employee is already active",
		vi: "Nhân viên đang hoạt động",
		zh: "员工已处于活动状态",
	},
	"error.rehire_date_invalid": {
		en: "Rehire date must be after termination date",
		vi: "Ngày tái tuyển dụng phải sau ngày chấm dứt hợp đồng",
		zh: "重新雇佣日期必须在解雇日期之后",
	},
	"error.no_active_employment": {
		en: "No active employment found for employee",
		vi: "Không tìm thấy hợp đồng lao động đang hoạt động cho nhân viên",
		zh: "没有为员工找到处于活动状态的雇佣关系",
	},
	"error.no_employment_on_date": {
		en: "No employment covers the punch date",
		vi: "Không có hợp đồng lao động nào bao phủ ngày chấm công",
		zh: "没有雇佣关系覆盖打卡日期",
	},
	"error.ambiguous_employment_range": {
		en: "Date range spans multiple employments, please specify employment_id",
		vi: "Khoảng ngày bao phủ nhiều hợp đồng lao động, vui lòng chỉ định employment_id",
		zh: "日期范围跨越多个雇佣关系，请指定 employment_id",
	},
	"error.ambiguous_employment_on_date": {
		en: "Several employments cover the punch date, please specify employment_id",
		vi: "Nhiều hợp đồng lao động bao phủ ngày chấm công, vui lòng chỉ định employment_id",
		zh: "多个雇佣关系覆盖打卡日期，请指定 employment_id",
	},
	"error.employment_not_covering_punch": {
		en: "The selected employment does not cover the punch date",
		vi: "Hợp đồng lao động đã chọn không bao phủ ngày chấm công",
		zh: "所选雇佣关系未覆盖打卡日期",
	},
	"error.employee_already_terminated": {
		en: "Employee is already terminated",
		vi: "Nhân viên đã nghỉ việc từ trước",
		zh: "该员工已被解雇",
	},
	"error.overlapping_employment": {
		en: "Overlapping primary employments are not allowed",
		vi: "Không cho phép các mối quan hệ công việc chính trùng lặp thời gian",
		zh: "不允许重叠的主要雇佣关系",
	},
	"error.custom_field_not_found": {
		en: "Custom field not found",
		vi: "Không tìm thấy trường tùy chỉnh",
		zh: "未找到自定义字段",
	},
	"error.template_not_found": {
		en: "Lifecycle checklist template not found",
		vi: "Không tìm thấy mẫu danh sách kiểm tra vòng đời",
		zh: "未找到生命周期清单模板",
	},
	"error.template_item_not_found": {
		en: "Lifecycle checklist template item not found",
		vi: "Không tìm thấy mục mẫu danh sách kiểm tra vòng đời",
		zh: "未找到生命周期清单模板项",
	},
	"error.template_name_empty": {
		en: "Template name cannot be empty",
		vi: "Tên mẫu không được để trống",
		zh: "模板名称不能为空",
	},
	"error.template_item_title_empty": {
		en: "Item title cannot be empty",
		vi: "Tiêu đề mục không được để trống",
		zh: "项目标题不能为空",
	},
	"error.checklist_not_found": {
		en: "Lifecycle checklist not found",
		vi: "Không tìm thấy danh sách kiểm tra vòng đời",
		zh: "未找到生命周期清单",
	},
	"error.checklist_item_not_found": {
		en: "Lifecycle checklist item not found",
		vi: "Không tìm thấy mục danh sách kiểm tra vòng đời",
		zh: "未找到生命周期清单项",
	},
	"error.cannot_open_xlsx": {
		en: "Cannot open Excel XLSX file: {error}",
		vi: "Không thể mở file Excel XLSX: {error}",
		zh: "无法打开 Excel XLSX 文件: {error}",
	},
	"error.cannot_open_xls": {
		en: "Cannot open Excel XLS file: {error}",
		vi: "Không thể mở file Excel XLS: {error}",
		zh: "无法打开 Excel XLS 文件: {error}",
	},
	"error.empty_excel_sheet": {
		en: "Excel file is empty or contains only headers.",
		vi: "File Excel trống hoặc chỉ chứa dòng tiêu đề.",
		zh: "Excel 文件为空或仅包含表头。",
	},
	"error.duplicate_headers": {
		en: "Duplicate columns detected: {headers}",
		vi: "Phát hiện các cột bị trùng lặp: {headers}",
		zh: "检测到重复的表头: {headers}",
	},
	"error.missing_mandatory_headers": {
		en: "Missing mandatory headers: {headers}",
		vi: "Thiếu các cột bắt buộc: {headers}",
		zh: "缺少必填表头: {headers}",
	},
	"error.no_sheets_found": {
		en: "Excel file does not contain any sheets.",
		vi: "File Excel không chứa bất kỳ sheet nào.",
		zh: "Excel 文件中未找到任何工作表。",
	},
	"error.cannot_read_sheet": {
		en: "Cannot read sheet {name}: {error}",
		vi: "Không thể đọc sheet {name}: {error}",
		zh: "无法读取工作表 {name}: {error}",
	},
	"error.employment_inactive": {
		en: "Employment is inactive",
		vi: "Quan hệ công việc không hoạt động",
		zh: "雇佣关系处于非活动状态",
	},
	"error.time_tracking_too_early": {
		en: "Too early to punch in",
		vi: "Chưa đến thời gian chấm công",
		zh: "未到打卡时间",
	},
	"error.time_tracking_failed_fetch_punched": {
		en: "Failed to fetch punched entry",
		vi: "Không thể tải thông tin lượt chấm công",
		zh: "未能获取打卡记录",
	},
	"error.time_entry_not_found": {
		en: "Time entry not found",
		vi: "Không tìm thấy lượt chấm công",
		zh: "未找到打卡记录",
	},
	"error.time_tracking_edit_other_employee": {
		en: "Not allowed to edit entries from other employees",
		vi: "Không được phép chỉnh sửa lượt chấm công của nhân viên khác",
		zh: "不允许编辑其他员工的记录",
	},
	"error.time_tracking_edit_other_workspace": {
		en: "Not allowed to edit entries from other workspaces",
		vi: "Không được phép chỉnh sửa lượt chấm công của workspace khác",
		zh: "不允许编辑其他工作空间的记录",
	},
	"error.leave_half_day_single_day": {
		en: "Half-day leave must be a single day (start_date == end_date)",
		vi: "Nghỉ nửa ngày phải nằm trong cùng một ngày (start_date == end_date)",
		zh: "半天请假必须在同一天内 (start_date == end_date)",
	},
	"error.leave_custom_hours_missing_time": {
		en: "Custom hours leave requires both start_time and end_time",
		vi: "Nghỉ phép theo giờ yêu cầu cả thời gian bắt đầu và kết thúc",
		zh: "按小时请假需要同时提供开始时间和结束时间",
	},
	"error.leave_custom_hours_single_day": {
		en: "Custom hours leave must be a single day (start_date == end_date)",
		vi: "Nghỉ phép theo giờ phải nằm trong cùng một ngày (start_date == end_date)",
		zh: "按小时请假必须在同一天内 (start_date == end_date)",
	},
	"error.leave_start_time_before_end_time": {
		en: "start_time must be before end_time",
		vi: "Thời gian bắt đầu phải trước thời gian kết thúc",
		zh: "开始时间必须早于结束时间",
	},
	"error.leave_not_found_after_update": {
		en: "Not found after status update",
		vi: "Không tìm thấy sau khi cập nhật trạng thái",
		zh: "更新状态后未找到资源",
	},
	"error.reset_token_invalid": {
		en: "This reset link is invalid or has expired.",
		vi: "Liên kết đặt lại mật khẩu không hợp lệ hoặc đã hết hạn.",
		zh: "该密码重置链接无效或已过期。",
	},
	"error.reset_token_expired": {
		en: "This reset link has expired.",
		vi: "Liên kết đặt lại mật khẩu đã hết hạn.",
		zh: "该密码重置链接已过期。",
	},
	"error.shift_overlap_detected": {
		en: "Shift overlap detected. The employee is already assigned to another shift at this time.",
		vi: "Phát hiện trùng lặp ca làm việc. Nhân viên đã được phân công vào một ca khác vào thời gian này.",
		zh: "检测到班次重叠。员工在该时间段已被分配到另一个班次。",
	},
	"error.insufficient_rest_period": {
		en: "Insufficient rest period between shifts. Minimum required rest is {min_hours}h.",
		vi: "Thời gian nghỉ giữa các ca không đủ. Yêu cầu nghỉ tối thiểu {min_hours} giờ.",
		zh: "班次间休息时间不足。最低要求休息 {min_hours} 小时。",
	},
	"error.weekly_hours_exceeded": {
		en: "Weekly working hours exceeded. Current: {current}h, limit: {limit}h.",
		vi: "Tổng giờ làm việc trong tuần vượt quá giới hạn. Hiện tại: {current}h, giới hạn: {limit}h.",
		zh: "每周工作时间超标。当前：{current}h，限制：{limit}h。",
	},
	"error.monthly_ot_exceeded": {
		en: "Monthly overtime hours exceeded. Current: {current} OT hours, limit: {limit}h.",
		vi: "Giờ tăng ca trong tháng vượt quá giới hạn. Hiện tại: {current} giờ, giới hạn: {limit}h.",
		zh: "每月加班时间超标。当前：{current} 小时，限制：{limit}h。",
	},
	"error.compliance_override_required": {
		en: "Compliance override reason is required to proceed with this assignment.",
		vi: "Cần cung cấp lý do ghi đè tuân thủ để tiếp tục phân công này.",
		zh: "需要提供合规性覆盖理由才能继续本次分配。",
	},
	"termination.multiple_non_terminated_warning": {
		en: "Found {count} non-terminated employments. Automatically selected the latest one (starting {start_date}).",
		vi: "Tìm thấy {count} hợp đồng chưa kết thúc. Đã tự động chọn hợp đồng mới nhất (bắt đầu ngày {start_date}).",
		zh: "发现 {count} 个未终止的雇佣关系。已自动选择最新的一个（开始于 {start_date}）。",
	},
	"error.termination_date_before_start": {
		en: "Termination date cannot be before employment start date ({start_date})",
		vi: "Ngày nghỉ việc không được trước ngày bắt đầu làm việc ({start_date})",
		zh: "离职日期不能早于雇佣开始日期 ({start_date})",
	},
	"error.termination_reason_invalid": {
		en: "Invalid termination reason: '{reason}'. Valid values: {valid}",
		vi: "Lý do nghỉ việc không hợp lệ: '{reason}'. Các giá trị hợp lệ: {valid}",
		zh: "无效的离职原因: '{reason}'。有效值: {valid}",
	},
	"error.termination_duplicate_code": {
		en: "Duplicate employee code in the file",
		vi: "Mã nhân viên bị lặp lại trong tệp",
		zh: "文件中员工代码重复",
	},
	"error.work_segment_not_found": {
		en: "Work segment not found",
		vi: "Không tìm thấy work segment",
		zh: "未找到工作段",
	},
	"error.work_segment_batch_too_large": {
		en: "Too many segments in one bulk request. Please reduce the batch size.",
		vi: "Quá nhiều segment trong một yêu cầu hàng loạt. Vui lòng giảm kích thước lô.",
		zh: "批量请求中的段数过多。请减少批次大小。",
	},
}
