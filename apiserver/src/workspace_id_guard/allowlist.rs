/// Queries that are legitimately exempt from workspace_id filtering.
/// Format: (file_basename, table_name, reason)
///
/// Keep this list as SHORT as possible. Each entry must have a
/// documented reason. Code reviewers should scrutinize additions.
pub const ALLOWLIST: &[(&str, &str, &str)] = &[
    // Auth: agent_id comes from verified WebSocket token, not user input
    (
        "agent_repository.rs",
        "authorized_agents",
        "agent lookup by verified token ID",
    ),
    (
        "agent_repository.rs",
        "devices",
        "device lookup by authenticated agent_id",
    ),
    // Roster CRUD: employee_id already validated from MemberContext (workspace-scoped)
    (
        "roster_crud.rs",
        "shift_rosters",
        "employee_id from MemberContext",
    ),
    // Leave request repo: queries by employee_id from workspace-validated handlers
    (
        "leave_request_repository.rs",
        "leave_requests",
        "employee_id from workspace-validated context",
    ),
    (
        "leave_request_repository.rs",
        "leave_balances",
        "employee_id from workspace-validated context",
    ),
    (
        "diesel_leave_request_repository.rs",
        "leave_requests",
        "employee_id from workspace-validated context",
    ),
    (
        "diesel_leave_request_repository.rs",
        "leave_balances",
        "employee_id from workspace-validated context",
    ),
    // Payslip segment/incentive: accessed via payslip_id (already workspace-scoped)
    (
        "payroll_repository.rs",
        "payslip_segments",
        "payslip_id from workspace-scoped payslip",
    ),
    (
        "payroll_repository.rs",
        "payslip_incentives",
        "payslip_id from workspace-scoped payslip",
    ),
    (
        "payroll_repository.rs",
        "payroll_incentive_tiers",
        "policy_id from workspace-scoped policy",
    ),
    // Employee salary components: joined with salary_component_definitions.workspace_id
    (
        "payroll_repository.rs",
        "employee_salary_components",
        "joined with scd.workspace_id filter",
    ),
    (
        "diesel_payroll_repository.rs",
        "employee_salary_components",
        "joined with scd.workspace_id filter",
    ),
    (
        "salary_components.rs",
        "employee_salary_components",
        "joined with scd.workspace_id filter",
    ),
    // Import service: uses job_id which is workspace-scoped
    (
        "attendance_import_service.rs",
        "attendance_import_jobs",
        "job_id from workspace-scoped context",
    ),
    (
        "attendance_import_service.rs",
        "raw_attendance_logs",
        "import job already workspace-scoped",
    ),
    (
        "attendance_import_service.rs",
        "employees",
        "employee lookup within import job context",
    ),
    // Workspace service: operates on workspace creation/management
    (
        "workspace_service.rs",
        "workspace_members",
        "workspace creation/management context",
    ),
    // Pairing service: already has workspace_id in scope but queries by employee_id
    (
        "pairing_service.rs",
        "attendance_daily_summaries",
        "workspace_id added to all queries",
    ),
    (
        "pairing_service.rs",
        "raw_attendance_logs",
        "workspace_id passed from handler context",
    ),
    // Holiday leave service: workspace_id from authenticated context
    (
        "holiday_leave_service.rs",
        "leave_requests",
        "workspace context from handler",
    ),
    (
        "holiday_leave_service.rs",
        "leave_balances",
        "workspace context from handler",
    ),
    // Department repo: some queries via department_id which has workspace FK
    (
        "department_repository.rs",
        "department_payroll_configs",
        "department_id has workspace FK",
    ),
    (
        "diesel_department_repository.rs",
        "department_payroll_configs",
        "department_id has workspace FK",
    ),
    // Resolver: config resolution via department_id which has workspace FK
    (
        "resolver.rs",
        "department_payroll_configs",
        "department_id has workspace FK",
    ),
    // Shift-related: some queries by shift_id which already belongs to a workspace
    (
        "shift_repository.rs",
        "employee_shifts",
        "shift_id has workspace FK",
    ),
    // Time entry repo: some internal utility queries
    (
        "time_entry_repository.rs",
        "attendance_adjustment_requests",
        "time_entry_id from workspace-scoped entry",
    ),
    // Overtime rules: accessed via policy_id which is workspace-scoped
    (
        "overtime_repository.rs",
        "overtime_rules",
        "policy_id from workspace-scoped policy",
    ),
    // Leave balance repo: employee_id from workspace-validated context
    (
        "leave_balance_repository.rs",
        "leave_balances",
        "employee_id from workspace-validated handler",
    ),
    // Core queries: employee/department payroll configs via FK
    (
        "core_queries.rs",
        "employee_payroll_configs",
        "employee_id has workspace FK via employees",
    ),
    // Attendance handler: leave lookups via employee_id from MemberContext
    (
        "attendance.rs",
        "leave_requests",
        "employee_id from workspace-validated MemberContext",
    ),
    // Leave accrual log repo: balance updates via employee_id FK-scoped from accrual job
    (
        "leave_accrual_log_repository.rs",
        "leave_balances",
        "employee_id from workspace-scoped accrual job",
    ),
    (
        "diesel_leave_accrual_log_repository.rs",
        "leave_balances",
        "employee_id from workspace-scoped accrual job",
    ),
    // Report service: employee-level leave aggregation
    (
        "report_service.rs",
        "leave_requests",
        "employee_id from workspace-validated report context",
    ),
    // Employee search/filter count query (dynamic where_clause contains workspace_id filter)
    (
        "segments.rs",
        "employees",
        "where_clause is constructed dynamically with workspace_id filter",
    ),
];
