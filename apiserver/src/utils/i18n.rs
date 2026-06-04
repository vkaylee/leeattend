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
    "error.database_error",
    "error.cannot_revoke_last_admin",
    "error.behalf_assignment_missing_target",
    "error.behalf_assignment_dual_target",
    "error.upload_multipart",
    "error.upload_read_failed",
    "error.upload_file_too_large",
    "error.upload_no_file",
    "error.tenant_mismatch",
];

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
        _ => None,
    }
}

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
        "error.database_error" => Some("Lỗi cơ sở dữ liệu"),
        "error.cannot_revoke_last_admin" => Some("Không thể thu hồi platform_admin cuối cùng. Hãy gán quản trị viên khác trước."),
        "error.behalf_assignment_missing_target" => Some("Phải chỉ định nhân viên đích hoặc phòng ban đích"),
        "error.behalf_assignment_dual_target" => Some("Không thể chỉ định cả nhân viên đích và phòng ban đích"),
        "error.upload_multipart" => Some("Lỗi tải lên dữ liệu dạng multipart"),
        "error.upload_read_failed" => Some("Không thể đọc dữ liệu file tải lên"),
        "error.upload_file_too_large" => Some("File tải lên vượt quá dung lượng cho phép"),
        "error.upload_no_file" => Some("Không có file nào được tải lên"),
        "error.tenant_mismatch" => Some("Không khớp tenant"),
        _ => None,
    }
}

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
        _ => None,
    }
}

#[cfg(test)]
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

        const LEGACY_ALLOWLIST: &[&str] = &[
            "time_tracking_service.rs",
            "overtime_request.rs",
            "debug.rs",
            "agents.rs",
            "workspace_config_service.rs",
            "diesel_staging_log_repository.rs",
            "diesel_workspace_repository.rs",
            "diesel_attendance_policy_repository.rs",
            "payroll_service.rs",
            "shift_swap_service.rs",
            "employee_service.rs",
            "report_service.rs",
            "auth.rs",
            "report_export.rs",
            "shifts.rs",
            "leave_service.rs",
            "leaves.rs",
            "payslips.rs",
            "import.rs",
            "reports.rs",
            "overtime.rs",
            "paper_forms.rs",
            "errors.rs",
            "i18n.rs",
            "crud.rs",
            "permissions.rs",
            "members.rs",
            "roles.rs",
            "holidays.rs",
            "employee_contracts.rs",
            "staging.rs",
            "settings.rs",
            "employees.rs",
            "employee_assignments.rs",
            "location_service.rs",
            "employer_service.rs",
            "merge.rs",
            "system_tenants.rs",
            "jobs.rs",
            "system_auth.rs",
            "system_admin.rs",
            "approval_workflow.rs",
            "ess.rs",
            "worker_monitor.rs",
            "leave_types.rs",
            "feature_flags.rs",
            "common.rs",
            "auth_service.rs",
            "workspace_service.rs",
            "overtime_request_service.rs",
            "attendance_adjustment_service.rs",
            "recruitment_service.rs",
            "attendance_summary_service.rs",
            "agent_service.rs",
            "definition.rs",
            "engine.rs",
            "task_resolver.rs",
            "tests.rs",
            "attendance_import_service.rs",
            "attendance_log_service.rs",
            "audit_service.rs",
            "holiday_leave_service.rs",
            "leave_accrual_service.rs",
            "leave_import_service.rs",
            "pairing_service.rs",
            "shift_service.rs",
            "working_pattern_service.rs",
            "working_patterns.rs",
            "system_db_clusters.rs",
            "notifications.rs",
            "dashboard_preferences.rs",
            "assignment_rules.rs",
            "salary_bases.rs",
            "recruitment_sources.rs",
            "employment_phases.rs",
            "payroll_rules.rs",
            "shift_swaps.rs",
            "audit.rs",
            "data_management.rs",
            "departments.rs",
            "employers.rs",
            "locations.rs",
            "diesel_employment_phase_repository.rs",
            "diesel_leave_policy_repository.rs",
            "diesel_approval_workflow_repository.rs",
            "core.rs",
            "diesel_salary_base_repository.rs",
            "diesel_working_pattern_repository.rs",
            "diesel_employee_assignment_repository.rs",
            "diesel_employee_contract_repository.rs",
            "diesel_department_repository.rs",
            "diesel_holiday_repository.rs",
            "diesel_leave_type_repository.rs",
            "diesel_employee_repository.rs",
            "diesel_audit_repository.rs",
            "diesel_leave_repository.rs",
            "diesel_recruitment_source_repository.rs",
            "diesel_shift_repository.rs",
            "diesel_payroll_rule_repository.rs",
            "diesel_payroll_run_repository.rs",
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
                        if !rest.starts_with("error.")
                            && !rest.starts_with("dev.")
                            && !rest.starts_with('x')
                        {
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
