# Endpoint Coverage Gap (Rust contract vs Go implementation)

Generated from `endpoint-matrix.csv` compared with `apiserver/main.go` route table.

- Contract routes: **403**
- Go route registrations: **135**
- Contract routes covered: **115**
- Contract routes missing: **288**

## Missing routes

| Method | Path | Permission |
|---|---|---|
| POST | `/api/v1/agent/{agent_id}/command` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/agents` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/authorized-agents` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/authorized-agents` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/authorized-agents/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/authorized-agents/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PATCH | `/api/v1/authorized-agents/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/authorized-agents/{id}/trust` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/dashboard/attendance-trend` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/department-attendance` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/executive-overview` | Permission::DashboardRead; conditional: Permission::PayrollSensitiveRead (controls overtime cost inclusion & dashboard.sensitive.read audit log) |
| GET | `/api/v1/dashboard/lateness-by-day` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/leave-types` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/overtime-breakdown` | Permission::DashboardRead; conditional: Permission::PayrollSensitiveRead (controls overtime cost inclusion & dashboard.sensitive.read audit log) |
| GET | `/api/v1/dashboard/overtime-trend` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/peak-times` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/preferences` | Permission::DashboardPreferences |
| PUT | `/api/v1/dashboard/preferences` | Permission::DashboardPreferences |
| GET | `/api/v1/dashboard/recruitment-effectiveness` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/recruitment-sources` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/turnover-analytics` | Permission::DashboardRead |
| GET | `/api/v1/dashboard/unexcused-absences` | Permission::DashboardEmployeeAbsenceRead OR Permission::DashboardRead |
| GET | `/api/v1/dashboard/workspace-stats` | Permission::DashboardRead |
| GET | `/api/v1/devices` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/devices` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/devices/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/devices/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/devices/{id}/info` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/employees/{id}/leave/requests` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/attendance` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/documents` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/documents/pending` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/ess/documents/{id}/acknowledge` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/leave/balances` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/leave/requests` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/payslips` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/ess/punch` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/shift-swaps` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/ess/shift-swaps` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/ess/shift-swaps/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/ess/shift-swaps/{id}/respond` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/ess/time-entries` | NONE EVIDENCED — context extractor only; no permission guard |
| PATCH | `/api/v1/me` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/me/permissions` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/monitor/alerts` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/alerts/{alert_id}` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| POST | `/api/v1/monitor/alerts/{alert_id}/acknowledge` | SystemRoleType::PlatformAdmin |
| POST | `/api/v1/monitor/alerts/{alert_id}/resolve` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/monitor/correlations` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/jobs` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/overview` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/stream` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/system/db-clusters` | SystemRoleType::PlatformAdmin |
| POST | `/api/v1/system/db-clusters` | SystemRoleType::PlatformAdmin |
| DELETE | `/api/v1/system/db-clusters/{id}` | SystemRoleType::PlatformAdmin |
| PATCH | `/api/v1/system/db-clusters/{id}/status` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/system/health` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/system/roles` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/system/smtp/config` | SystemRoleType::PlatformAdmin |
| PUT | `/api/v1/system/smtp/config` | SystemRoleType::PlatformAdmin |
| POST | `/api/v1/system/smtp/test` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/system/tenants` | SystemRoleType::PlatformAdmin |
| POST | `/api/v1/system/tenants` | SystemRoleType::PlatformAdmin |
| DELETE | `/api/v1/system/tenants/{tenant_id}` | SystemRoleType::PlatformAdmin |
| PUT | `/api/v1/system/tenants/{tenant_id}/database` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/system/tenants/{tenant_id}/stats` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/system/users/{user_id}/roles` | SystemRoleType::PlatformAdmin |
| POST | `/api/v1/system/users/{user_id}/roles` | SystemRoleType::PlatformAdmin |
| DELETE | `/api/v1/system/users/{user_id}/roles/{role_name}` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/users` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/approval-workflows/pending-tasks` | Permission::ApprovalWorkflowsRead |
| DELETE | `/api/v1/workspaces/{ws_id}/approval-workflows/{id}` | Permission::ApprovalWorkflowsManage |
| GET | `/api/v1/workspaces/{ws_id}/approval-workflows/{id}` | Permission::ApprovalWorkflowsRead |
| PUT | `/api/v1/workspaces/{ws_id}/approval-workflows/{id}` | Permission::ApprovalWorkflowsManage |
| GET | `/api/v1/workspaces/{ws_id}/assignment-history` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/assignment-rules` | Permission::AssignmentRulesManage |
| POST | `/api/v1/workspaces/{ws_id}/assignment-rules` | Permission::AssignmentRulesManage |
| DELETE | `/api/v1/workspaces/{ws_id}/assignment-rules/{rule_id}` | Permission::AssignmentRulesManage |
| PATCH | `/api/v1/workspaces/{ws_id}/assignment-rules/{rule_id}` | Permission::AssignmentRulesManage |
| GET | `/api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments` | Permission::AttendanceAdjustmentsManageBehalf |
| GET | `/api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/my-targets` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/{id}` | Permission::AttendanceAdjustmentsManageBehalf |
| POST | `/api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve` | Permission::AttendanceAdjustmentsApprove |
| POST | `/api/v1/workspaces/{ws_id}/attendance/approve-ot/{employee_id}` | Permission::OvertimeApprove (scoped) |
| POST | `/api/v1/workspaces/{ws_id}/attendance/bulk-upload-paper-ot` | Permission::OvertimePaperUpload |
| GET | `/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments` | Permission::OvertimePoliciesManage |
| DELETE | `/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{id}` | Permission::OvertimePoliciesManage |
| POST | `/api/v1/workspaces/{ws_id}/attendance/claim-ot/{employee_id}` | CONDITIONAL — self target: no permission check; delegated target: Permission::OvertimeApprove OR Permission::OvertimeClaimOnBehalf |
| POST | `/api/v1/workspaces/{ws_id}/attendance/enter-paper-ot/{employee_id}` | Permission::OvertimePaperUpload |
| GET | `/api/v1/workspaces/{ws_id}/attendance/import-jobs/latest` | Permission::AttendanceLogsImport |
| GET | `/api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}` | Permission::AttendanceLogsImport |
| GET | `/api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}/error-file` | Permission::AttendanceLogsImport |
| GET | `/api/v1/workspaces/{ws_id}/attendance/logs` | Permission::AttendanceLogsRead |
| POST | `/api/v1/workspaces/{ws_id}/attendance/logs/upload` | Permission::AttendanceLogsImport |
| GET | `/api/v1/workspaces/{ws_id}/attendance/policies/assignments` | Permission::WorkspaceSettingsRead |
| POST | `/api/v1/workspaces/{ws_id}/attendance/policies/assignments` | Permission::WorkspaceSettingsWrite |
| DELETE | `/api/v1/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}` | Permission::WorkspaceSettingsWrite |
| GET | `/api/v1/workspaces/{ws_id}/attendance/raw-logs` | Permission::AttendanceLogsRead |
| POST | `/api/v1/workspaces/{ws_id}/attendance/reprocess-all` | Permission::AttendanceReprocess |
| POST | `/api/v1/workspaces/{ws_id}/attendance/reprocess/{employee_id}` | Permission::AttendanceReprocess |
| GET | `/api/v1/workspaces/{ws_id}/attendance/rules` | Permission::WorkspaceSettingsRead |
| PATCH | `/api/v1/workspaces/{ws_id}/attendance/rules` | Permission::WorkspaceSettingsWrite |
| GET | `/api/v1/workspaces/{ws_id}/attendance/rules/versions` | Permission::WorkspaceSettingsRead |
| GET | `/api/v1/workspaces/{ws_id}/attendance/staging-logs` | Permission::AttendanceStagingManage |
| DELETE | `/api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}` | Permission::AttendanceStagingManage |
| POST | `/api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}/resolve` | Permission::AttendanceStagingManage |
| GET | `/api/v1/workspaces/{ws_id}/attendance/summaries` | CONDITIONAL — Permission::AttendanceSummariesRead via has_permission; handler-specific self/scope behavior |
| POST | `/api/v1/workspaces/{ws_id}/attendance/upload` | Permission::AttendanceLogsImport |
| GET | `/api/v1/workspaces/{ws_id}/audit-logs` | Permission::AuditLogsRead |
| GET | `/api/v1/workspaces/{ws_id}/contracts` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/documents/distributions` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/documents/folders` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/documents/folders` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/documents/templates/preview` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/documents/templates/{id}/generate` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/import-termination/confirm` | Permission::EmployeesUpdate |
| POST | `/api/v1/workspaces/{ws_id}/employees/import-termination/preview` | Permission::EmployeesUpdate |
| GET | `/api/v1/workspaces/{ws_id}/employees/import-termination/template` | Permission::EmployeesUpdate |
| POST | `/api/v1/workspaces/{ws_id}/employees/upload` | Permission::EmployeesUpload |
| GET | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PATCH | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PATCH | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PATCH | `/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/employees/{id}` | Permission::EmployeesUpdate (scoped) |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/calculate-payroll` | Permission::PayrollCalculate |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}/compensation-history` | Permission::EmployeesRead |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}/conditions` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/conditions` | Permission::EmployeesUpdate (scoped) |
| DELETE | `/api/v1/workspaces/{ws_id}/employees/{id}/conditions/{cond_id}` | Permission::EmployeesUpdate (scoped) |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}/departments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/departments` | Permission::DepartmentsTransfer (scoped) |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/leave/adjust` | Permission::LeaveBalancesAdjust |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}/leave/history` | Permission::LeaveBalancesRead |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employees/{user_id}/phases` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{user_id}/phases` | Permission::EmploymentPhasesManage |
| PUT | `/api/v1/workspaces/{ws_id}/employees/{user_id}/phases/{phase_id}/status` | Permission::EmploymentPhasesManage |
| GET | `/api/v1/workspaces/{ws_id}/employers` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employers` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/employers/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employers/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/employers/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employment-phases` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employment-phases` | Permission::EmploymentPhasesManage |
| GET | `/api/v1/workspaces/{ws_id}/employments/{emp_id}/lifecycle` | Permission::EmployeesRead |
| POST | `/api/v1/workspaces/{ws_id}/employments/{emp_id}/lifecycle` | Permission::EmployeesUpdate |
| GET | `/api/v1/workspaces/{ws_id}/employments/{employment_id}/phases` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employments/{employment_id}/phases` | Permission::EmploymentPhasesManage |
| PUT | `/api/v1/workspaces/{ws_id}/employments/{employment_id}/phases/{phase_id}/status` | Permission::EmploymentPhasesManage |
| GET | `/api/v1/workspaces/{ws_id}/employments/{id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employments/{id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/files/upload` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/files/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/files/{id}/download` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/files/{id}/versions` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/holidays` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/holidays` | Permission::HolidaysManage |
| DELETE | `/api/v1/workspaces/{ws_id}/holidays/{id}` | Permission::HolidaysManage |
| GET | `/api/v1/workspaces/{ws_id}/holidays/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PATCH | `/api/v1/workspaces/{ws_id}/holidays/{id}` | Permission::HolidaysManage |
| PUT | `/api/v1/workspaces/{ws_id}/holidays/{id}` | Permission::HolidaysManage |
| POST | `/api/v1/workspaces/{ws_id}/holidays/{id}/process-leave` | Permission::HolidaysManage |
| GET | `/api/v1/workspaces/{ws_id}/jobs/{job_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/jobs/{job_id}/status` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/leave/accrual/run` | Permission::LeaveAccrualRun |
| GET | `/api/v1/workspaces/{ws_id}/leave/balances` | Permission::LeaveBalancesRead |
| GET | `/api/v1/workspaces/{ws_id}/leave/behalf-assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/leave/behalf-assignments` | Permission::LeavePoliciesManage |
| GET | `/api/v1/workspaces/{ws_id}/leave/behalf-assignments/my-targets` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/leave/behalf-assignments/{id}` | Permission::LeavePoliciesManage |
| GET | `/api/v1/workspaces/{ws_id}/leave/import/latest` | Permission::LeaveRequestsSubmit |
| GET | `/api/v1/workspaces/{ws_id}/leave/import/{job_id}/errors` | Permission::LeaveRequestsSubmit |
| GET | `/api/v1/workspaces/{ws_id}/leave/policies` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/leave/policies` | Permission::LeavePoliciesManage |
| POST | `/api/v1/workspaces/{ws_id}/leave/policies/assign` | Permission::LeavePoliciesManage |
| GET | `/api/v1/workspaces/{ws_id}/leave/policies/assignments` | Permission::LeavePoliciesManage |
| DELETE | `/api/v1/workspaces/{ws_id}/leave/policies/assignments/{assignment_id}` | Permission::LeavePoliciesManage |
| DELETE | `/api/v1/workspaces/{ws_id}/leave/policies/{policy_id}` | Permission::LeavePoliciesManage |
| PATCH | `/api/v1/workspaces/{ws_id}/leave/policies/{policy_id}` | Permission::LeavePoliciesManage |
| DELETE | `/api/v1/workspaces/{ws_id}/leave/types/{id}` | Permission::LeaveTypesManage |
| PUT | `/api/v1/workspaces/{ws_id}/leave/types/{id}/config` | Permission::LeaveTypesManage |
| POST | `/api/v1/workspaces/{ws_id}/leave/types/{id}/restore` | Permission::LeaveTypesManage |
| POST | `/api/v1/workspaces/{ws_id}/leave/upload-excel` | Permission::LeaveRequestsSubmit |
| GET | `/api/v1/workspaces/{ws_id}/leave/upload-template` | Permission::LeaveRequestsSubmit |
| GET | `/api/v1/workspaces/{ws_id}/lifecycle-templates` | Permission::WorkspaceSettingsRead |
| POST | `/api/v1/workspaces/{ws_id}/lifecycle-templates` | Permission::WorkspaceSettingsWrite |
| DELETE | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{id}` | Permission::WorkspaceSettingsWrite |
| GET | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{id}` | Permission::WorkspaceSettingsRead |
| PATCH | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{id}` | Permission::WorkspaceSettingsWrite |
| GET | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{id}/items` | Permission::WorkspaceSettingsRead |
| POST | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{id}/items` | Permission::WorkspaceSettingsWrite |
| DELETE | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}` | Permission::WorkspaceSettingsWrite |
| GET | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}` | Permission::WorkspaceSettingsRead |
| PUT | `/api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}` | Permission::WorkspaceSettingsWrite |
| GET | `/api/v1/workspaces/{ws_id}/lifecycle/my-tasks` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/lifecycle/{id}` | Permission::EmployeesRead |
| PATCH | `/api/v1/workspaces/{ws_id}/lifecycle/{id}/items/{iid}` | Permission::EmployeesUpdate |
| GET | `/api/v1/workspaces/{ws_id}/locations` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/locations` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/locations/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/locations/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/locations/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/members` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/members` | Permission::MembersInvite |
| DELETE | `/api/v1/workspaces/{ws_id}/members/{user_id}` | Permission::MembersRemove |
| PATCH | `/api/v1/workspaces/{ws_id}/members/{user_id}` | Permission::MembersRolesManage |
| GET | `/api/v1/workspaces/{ws_id}/members/{user_id}/permissions` | Permission::MembersRead |
| POST | `/api/v1/workspaces/{ws_id}/members/{user_id}/permissions` | Permission::MembersRolesManage |
| DELETE | `/api/v1/workspaces/{ws_id}/members/{user_id}/permissions/{perm_id}` | Permission::MembersRolesManage |
| GET | `/api/v1/workspaces/{ws_id}/members/{user_id}/roles` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/members/{user_id}/roles` | Permission::MembersRolesManage |
| DELETE | `/api/v1/workspaces/{ws_id}/members/{user_id}/roles/{role_id}` | Permission::MembersRolesManage |
| GET | `/api/v1/workspaces/{ws_id}/monitor/jobs/{job_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/overtime/policies` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/overtime/policies` | Permission::OvertimePoliciesManage |
| POST | `/api/v1/workspaces/{ws_id}/overtime/policies/assign` | Permission::OvertimePoliciesManage |
| GET | `/api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}/rules` | Permission::OvertimePoliciesManage |
| GET | `/api/v1/workspaces/{ws_id}/overtime/requests` | Permission::OvertimeRequestRead |
| POST | `/api/v1/workspaces/{ws_id}/overtime/requests` | Permission::OvertimeRequestCreate |
| GET | `/api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments` | Permission::OvertimePoliciesManage |
| GET | `/api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/my-targets` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/{id}` | Permission::OvertimePoliciesManage |
| GET | `/api/v1/workspaces/{ws_id}/overtime/requests/{id}` | Permission::OvertimeRequestRead |
| POST | `/api/v1/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}` | CONDITIONAL — self target: no permission check; delegated target: Permission::OvertimeRequestCreateOnBehalf |
| POST | `/api/v1/workspaces/{ws_id}/overtime/requests/{id}/submit` | Permission::OvertimeRequestCreate |
| GET | `/api/v1/workspaces/{ws_id}/payroll-rules` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/payroll-rules` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers/{tier_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/payroll-settings` | NONE EVIDENCED — context extractor only; no permission guard |
| PATCH | `/api/v1/workspaces/{ws_id}/payroll-settings` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/payroll-settings/history` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/payroll/calculate-batch` | Permission::PayrollCalculate |
| GET | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/permissions/available` | Permission::MembersRolesManage |
| GET | `/api/v1/workspaces/{ws_id}/permissions/catalog` | Permission::MembersRolesManage |
| GET | `/api/v1/workspaces/{ws_id}/recruitment-sources` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/recruitment-sources` | Permission::RecruitmentManage |
| DELETE | `/api/v1/workspaces/{ws_id}/recruitment-sources/{id}` | Permission::RecruitmentManage |
| PUT | `/api/v1/workspaces/{ws_id}/recruitment-sources/{id}` | Permission::RecruitmentManage |
| GET | `/api/v1/workspaces/{ws_id}/roles` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/roles/{role_id}` | Permission::MembersRolesManage |
| PUT | `/api/v1/workspaces/{ws_id}/roles/{role_id}` | Permission::MembersRolesManage |
| GET | `/api/v1/workspaces/{ws_id}/roles/{role_id}/permissions` | Permission::MembersRolesManage |
| PUT | `/api/v1/workspaces/{ws_id}/roles/{role_id}/permissions` | Permission::MembersRolesManage |
| GET | `/api/v1/workspaces/{ws_id}/salary-bases` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/salary-bases` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/schedules` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/settings/storage` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/settings/storage` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/settings/storage` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/settings/storage/test` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/time/entries/{id}` | Permission::AttendanceLogsImport |
| PUT | `/api/v1/workspaces/{ws_id}/time/entries/{id}` | Permission::AttendanceLogsImport |
| GET | `/api/v1/ws` | NONE EVIDENCED — agent token/trusted-agent check; no resource:action permission |
| GET | `/api/v1/ws/reports/{job_id}` | NONE EVIDENCED — JWT plus workspace membership; no resource:action permission |
