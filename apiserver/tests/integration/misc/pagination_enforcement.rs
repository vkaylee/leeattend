//! Static analysis test to enforce pagination on all GET list endpoints.
//! This scans the handlers source code to verify that no GET endpoints return raw arrays (Vec/brackets)
//! unless they are explicitly whitelisted.

use std::fs;
use std::path::{Path, PathBuf};

// Whitelist of legacy or bounded GET list endpoints that are allowed to return raw arrays
const GET_LIST_WHITELIST: &[&str] = &[
    // Static permissions and metadata
    "get_available_permissions",
    "get_my_permissions",
    "get_role_permissions",
    "get_member_roles",
    "list_system_roles",
    "get_user_system_roles",
    "get_dashboard_preferences",
    // File downloads (returns binary [u8] data, not collection lists)
    "download_leave_import_errors",
    "download_leave_import_template",
    "download_report",
    "download_import_error_file",
    // Legacy bounded lists to be paginated in future phases (e.g. employee history, schedules, adjustments)
    "get_employee_department_history",
    "list_employee_assignments",
    "get_shifts",
    "get_workspace_schedules",
    "get_assignment_history",
    "get_employee_rosters",
    "list_patterns",
    "get_employee_pattern_history",
    "list_shift_swaps",
    "get_my_leave_requests",
    "get_employee_leave_balances",
    "get_workspace_leave_requests",
    "get_employee_leave_history",
    "get_leave_policies",
    "get_leave_policy_assignments",
    "list_leave_behalf_assignments",
    "list_ot_requests",
    "list_policies",
    "get_employee_payslips",
    "get_workspace_payslips",
    "get_salary_components",
    "get_employee_salary_components",
    "get_incentive_policies",
    "get_department_payroll_configs",
    "get_employee_payroll_configs",
    "get_salary_bases",
    "get_approval_history",
    "get_pending_tasks",
    "get_payroll_rule_definitions",
    "get_payroll_rule_tiers",
    "get_payroll_rule_assignments",
    "list_db_clusters",
    "list_tenants",
    "list_stale_tenants",
    "get_monitor_jobs",
    "get_notifications",
    "get_my_emergency_contacts",
    "get_my_documents",
    "get_my_payslips",
    "get_my_attendance",
    "get_my_time_entries",
    "get_my_leave_balances",
    "ess_get_my_leave_requests",
    "ess_get_my_shift_swaps",
    "list_employee_contracts",
    "get_time_history",
    "get_attendance_logs",
    "get_raw_attendance_logs",
    "list_configs",
    "get_users",
    "get_devices",
    "get_agents",
    "get_authorized_agents",
    "list_leave_types",
    "list_holidays",
    "get_staging_logs",
    "list_adjustment_behalf_assignments",
    "get_my_attendance_adjustments",
    "get_workspace_attendance_adjustments",
    "list_behalf_assignments",
    "get_attendance_summaries",
    "get_generic_job_status",
    "get_employment_phase_definitions",
    "get_employee_phases",
    "get_recruitment_source_breakdown",
    "get_turnover_analytics",
    "get_assignment_rules",
    "list_default_attendance_policy_versions",
    "get_payroll_settings_history",
    "get_workspace_members",
    "list_policy_assignments",
    "get_pattern_details",
    "list_feature_flags",
    "get_policy",
    "get_attendance_trend",
    "get_department_attendance",
    "get_unexcused_absences",
    "get_overtime_trend",
    "get_peak_times",
    "get_leave_types_breakdown",
    "get_lateness_by_day",
    "get_workspaces",
    "get_member_extra_permissions",
];

fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn extract_utoipa_paths(content: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut rest = content;

    while let Some(start_idx) = rest.find("#[utoipa::path(") {
        let macro_start = start_idx + "#[utoipa::path(".len();
        let mut depth = 1;
        let mut end_idx = macro_start;
        let bytes = rest.as_bytes();

        while end_idx < bytes.len() && depth > 0 {
            if bytes[end_idx] == b'(' {
                depth += 1;
            } else if bytes[end_idx] == b')' {
                depth -= 1;
            }
            end_idx += 1;
        }

        if depth == 0 {
            let path_macro = &rest[start_idx..end_idx];

            let after_macro = &rest[end_idx..];
            let mut fn_name = String::new();
            if let Some(fn_idx) = after_macro.find("pub async fn ") {
                let fn_start = fn_idx + "pub async fn ".len();
                let mut fn_end = fn_start;
                let after_bytes = after_macro.as_bytes();
                while fn_end < after_bytes.len()
                    && (after_bytes[fn_end].is_ascii_alphanumeric() || after_bytes[fn_end] == b'_')
                {
                    fn_end += 1;
                }
                fn_name = after_macro[fn_start..fn_end].to_string();
            } else if let Some(fn_idx) = after_macro.find("async fn ") {
                let fn_start = fn_idx + "async fn ".len();
                let mut fn_end = fn_start;
                let after_bytes = after_macro.as_bytes();
                while fn_end < after_bytes.len()
                    && (after_bytes[fn_end].is_ascii_alphanumeric() || after_bytes[fn_end] == b'_')
                {
                    fn_end += 1;
                }
                fn_name = after_macro[fn_start..fn_end].to_string();
            }

            if !fn_name.is_empty() {
                results.push((fn_name, path_macro.to_string()));
            }
            rest = &rest[end_idx..];
        } else {
            break;
        }
    }

    results
}

#[test]
fn test_enforce_pagination_on_get_lists() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let handlers_dir = Path::new(&manifest_dir).join("src").join("handlers");

    let mut files = Vec::new();
    visit_dirs(&handlers_dir, &mut files).unwrap();

    let mut violations = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file).unwrap();
        let paths = extract_utoipa_paths(&content);

        for (fn_name, path_macro) in paths {
            // Check if it is a GET request
            let is_get = path_macro.contains("get,") || path_macro.contains("method = get");
            if !is_get {
                continue;
            }

            // Check if the response body contains a list (Vec or array [T])
            let has_list_response = path_macro.contains("body = Vec<")
                || path_macro.contains("body = [")
                || path_macro.contains("body =  [")
                || path_macro.contains("body =  Vec<");

            if has_list_response {
                // If it is a list response, check if it's whitelisted
                if !GET_LIST_WHITELIST.contains(&fn_name.as_str()) {
                    violations.push(format!(
                        "Handler: {} in file: {:?}\nMacro:\n{}",
                        fn_name,
                        file.strip_prefix(&manifest_dir).unwrap_or(&file),
                        path_macro
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n🔴 PAGINATION ENFORCEMENT FAILURE:\n\
             The following GET list endpoints return raw arrays (Vec<T> or [T]) and are not whitelisted.\n\
             All new GET list endpoints MUST implement pagination and return a PaginatedResponse structure.\n\n\
             Violations:\n{}\n",
            violations.join("\n----------------------------------------\n")
        );
    }
}
