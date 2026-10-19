# Endpoint Coverage Gap (Rust contract vs Go implementation)

Generated from `endpoint-matrix.csv` compared with `apiserver/main.go` route table.

- Contract routes: **403**
- Go route registrations: **309**
- Contract routes covered: **284**
- Contract routes missing: **119**

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
| GET | `/api/v1/dashboard/recruitment-effectiveness` | Permission::DashboardRead |
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
| GET | `/api/v1/monitor/alerts` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/alerts/{alert_id}` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| POST | `/api/v1/monitor/alerts/{alert_id}/acknowledge` | SystemRoleType::PlatformAdmin |
| POST | `/api/v1/monitor/alerts/{alert_id}/resolve` | SystemRoleType::PlatformAdmin |
| GET | `/api/v1/monitor/correlations` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/jobs` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/overview` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/monitor/stream` | SystemRoleType::PlatformAdmin OR SystemRoleType::Support |
| GET | `/api/v1/workspaces/{ws_id}/approval-workflows/pending-tasks` | Permission::ApprovalWorkflowsRead |
| POST | `/api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve` | Permission::AttendanceAdjustmentsApprove |
| POST | `/api/v1/workspaces/{ws_id}/attendance/reprocess-all` | Permission::AttendanceReprocess |
| POST | `/api/v1/workspaces/{ws_id}/attendance/reprocess/{employee_id}` | Permission::AttendanceReprocess |
| GET | `/api/v1/workspaces/{ws_id}/attendance/summaries` | CONDITIONAL — Permission::AttendanceSummariesRead via has_permission; handler-specific self/scope behavior |
| GET | `/api/v1/workspaces/{ws_id}/audit-logs` | Permission::AuditLogsRead |
| POST | `/api/v1/workspaces/{ws_id}/documents/distributions` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/documents/folders` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/documents/folders` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/documents/templates/preview` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/documents/templates/{id}/generate` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/calculate-payroll` | Permission::PayrollCalculate |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/leave/adjust` | Permission::LeaveBalancesAdjust |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}/leave/history` | Permission::LeaveBalancesRead |
| GET | `/api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/employments/{id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/employments/{id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/files/upload` | NONE EVIDENCED — context extractor only; no permission guard |
| PUT | `/api/v1/workspaces/{ws_id}/files/{id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/files/{id}/download` | NONE EVIDENCED — context extractor only; no permission guard |
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
| POST | `/api/v1/workspaces/{ws_id}/leave/upload-excel` | Permission::LeaveRequestsSubmit |
| GET | `/api/v1/workspaces/{ws_id}/leave/upload-template` | Permission::LeaveRequestsSubmit |
| GET | `/api/v1/workspaces/{ws_id}/monitor/jobs/{job_id}` | NONE EVIDENCED — context extractor only; no permission guard |
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
| GET | `/api/v1/workspaces/{ws_id}/salary-bases` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/salary-bases` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}` | NONE EVIDENCED — context extractor only; no permission guard |
| GET | `/api/v1/workspaces/{ws_id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| POST | `/api/v1/workspaces/{ws_id}/salary-components` | NONE EVIDENCED — context extractor only; no permission guard |
| DELETE | `/api/v1/workspaces/{ws_id}/time/entries/{id}` | Permission::AttendanceLogsImport |
| PUT | `/api/v1/workspaces/{ws_id}/time/entries/{id}` | Permission::AttendanceLogsImport |
| GET | `/api/v1/ws` | NONE EVIDENCED — agent token/trusted-agent check; no resource:action permission |
| GET | `/api/v1/ws/reports/{job_id}` | NONE EVIDENCED — JWT plus workspace membership; no resource:action permission |
