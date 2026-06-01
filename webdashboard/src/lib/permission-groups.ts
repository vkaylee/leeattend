import type { LucideIcon } from "lucide-react";

import {
  Building2,
  Users,
  Clock,
  Banknote,
  ShieldCheck,
  Calendar,
  BarChart3,
  Monitor,
  FileText,
  Zap,
  UserCheck,
  CalendarDays,
  ClipboardList,
  Smartphone,
} from "lucide-react";

export interface PermissionDef {
  desc: string;
  id: string;
  name: string;
}

export interface PermissionGroup {
  icon: LucideIcon;
  id: string;
  name: string;
  permissions: PermissionDef[];
}

/**
 * Permission groups mapped 1:1 to the backend Permission enum string IDs.
 * Source of truth: apiserver/src/domain/permissions.rs → Permission::all_permission_strings()
 *
 * ⚠️  When adding a new Permission variant in Rust, you MUST also add it here
 *     AND update the PermissionSync.test.ts constant.
 */
export const permissionGroups: PermissionGroup[] = [
  {
    id: "workspace",
    name: "Workspace & Setup",
    icon: Building2,
    permissions: [
      { id: "workspace:update", name: "Edit Workspace", desc: "Change name, timezone, logo" },
      { id: "workspace:delete", name: "Delete Workspace", desc: "Permanently delete workspace" },
      { id: "workspace:settings:read", name: "View Settings", desc: "Access workspace configuration" },
      { id: "workspace:settings:write", name: "Edit Settings", desc: "Modify workspace configuration" },
      { id: "approval_workflows:read", name: "View Approval Workflows", desc: "See configured workflows" },
      { id: "approval_workflows:manage", name: "Manage Approval Workflows", desc: "Create and edit approval flows" },
    ]
  },
  {
    id: "members",
    name: "Members & Roles",
    icon: Users,
    permissions: [
      { id: "members:read", name: "View Members", desc: "See workspace member list" },
      { id: "members:invite", name: "Invite Members", desc: "Send invitation to new members" },
      { id: "members:roles:manage", name: "Manage Roles", desc: "Create, edit, delete custom roles" },
      { id: "members:remove", name: "Remove Members", desc: "Remove members from workspace" },
    ]
  },
  {
    id: "employees",
    name: "Employees",
    icon: UserCheck,
    permissions: [
      { id: "employees:read", name: "View Employees", desc: "Access employee directory" },
      { id: "employees:create", name: "Create Employees", desc: "Add new employee records" },
      { id: "employees:update", name: "Update Employees", desc: "Edit employee information" },
      { id: "employees:delete", name: "Delete Employees", desc: "Remove employee records" },
      { id: "employees:rehire", name: "Rehire Employees", desc: "Re-activate terminated employees" },
      { id: "employees:upload", name: "Upload Employees", desc: "Bulk import employee data" },
      { id: "employees:bulk_edit", name: "Bulk Edit", desc: "Mass update employee records" },
    ]
  },
  {
    id: "departments",
    name: "Departments",
    icon: Building2,
    permissions: [
      { id: "departments:read", name: "View Departments", desc: "Access organizational structure" },
      { id: "departments:create", name: "Create Departments", desc: "Add new departments" },
      { id: "departments:update", name: "Update Departments", desc: "Edit department details" },
      { id: "departments:delete", name: "Delete Departments", desc: "Remove departments" },
      { id: "departments:transfer", name: "Transfer Employees", desc: "Move employees between departments" },
    ]
  },
  {
    id: "attendance",
    name: "Time & Attendance",
    icon: Clock,
    permissions: [
      { id: "attendance:logs:read", name: "View Logs", desc: "See attendance time logs" },
      { id: "attendance:logs:import", name: "Import Logs", desc: "Upload attendance records" },
      { id: "attendance:summaries:read", name: "View Summaries", desc: "Access attendance reports" },
      { id: "attendance:adjustments:submit", name: "Submit Adjustments", desc: "Request time corrections" },
      { id: "attendance:adjustments:approve", name: "Approve Adjustments", desc: "Review and approve corrections" },
      { id: "attendance:reprocess", name: "Reprocess Data", desc: "Recalculate attendance records" },
      { id: "attendance:staging:manage", name: "Manage Staging", desc: "Handle staged attendance data" },
      { id: "attendance:force_clock_out", name: "Force Clock Out", desc: "Emergency force clock-out action" },
    ]
  },
  {
    id: "overtime",
    name: "Overtime",
    icon: Zap,
    permissions: [
      { id: "overtime:claim", name: "Claim OT", desc: "Submit overtime claims" },
      { id: "overtime:approve", name: "Approve OT", desc: "Review and approve OT claims" },
      { id: "overtime:claim_on_behalf", name: "Claim OT on Behalf", desc: "Submit overtime claims on behalf of others" },
      { id: "overtime:policies:manage", name: "Manage OT Policies", desc: "Create and edit OT rules" },
      { id: "overtime:paper:upload", name: "Upload OT Paper", desc: "Attach OT documentation" },
      { id: "overtime:request:create", name: "Create OT Request", desc: "Plan pre-overtime requests" },
      { id: "overtime:request:read", name: "View OT Requests", desc: "See overtime requests" },
      { id: "overtime:approve:workflow", name: "Workflow Approve OT", desc: "Approve OT in workflow" },
      { id: "overtime:quota:read", name: "View OT Quotas", desc: "See overtime quota balances" },
    ]
  },
  {
    id: "leave",
    name: "Leave Management",
    icon: Calendar,
    permissions: [
      { id: "leave:requests:read", name: "View Requests", desc: "See leave request list" },
      { id: "leave:requests:submit", name: "Submit Requests", desc: "Create leave requests for employees" },
      { id: "leave:requests:approve", name: "Approve Requests", desc: "Full approval authority" },
      { id: "leave:approve:l1", name: "Approve L1 (Team)", desc: "First-level approval (Team Lead)" },
      { id: "leave:approve:l2", name: "Approve L2 (HR)", desc: "Second-level approval (HR/Manager)" },
      { id: "leave:balances:read", name: "View Balances", desc: "See leave balance information" },
      { id: "leave:balances:adjust", name: "Adjust Balances", desc: "Manually modify leave balances" },
      { id: "leave:types:manage", name: "Manage Leave Types", desc: "Create and edit leave categories" },
      { id: "leave:policies:manage", name: "Manage Leave Policies", desc: "Configure accrual rules" },
      { id: "leave:accrual:run", name: "Run Accrual", desc: "Trigger leave accrual calculation" },
    ]
  },
  {
    id: "shifts",
    name: "Shifts & Scheduling",
    icon: CalendarDays,
    permissions: [
      { id: "shifts:read", name: "View Shifts", desc: "See shift definitions" },
      { id: "shifts:create", name: "Create Shifts", desc: "Define new shift schedules" },
      { id: "shifts:update", name: "Update Shifts", desc: "Edit shift configurations" },
      { id: "shifts:assign", name: "Assign Shifts", desc: "Assign shifts to employees" },
      { id: "shift_swaps:manage", name: "Manage Swaps", desc: "Handle shift swap requests" },
      { id: "shift_swaps:approve", name: "Approve Swaps", desc: "Approve shift swap requests" },
      { id: "patterns:read", name: "View Patterns", desc: "See working patterns" },
      { id: "patterns:create", name: "Create Patterns", desc: "Define new patterns" },
      { id: "patterns:assign", name: "Assign Patterns", desc: "Assign patterns to employees" },
      { id: "patterns:delete", name: "Delete Patterns", desc: "Remove working patterns" },
      { id: "rosters:read", name: "View Rosters", desc: "See roster schedules" },
      { id: "rosters:manage", name: "Manage Rosters", desc: "Create and edit rosters" },
      { id: "assignment_rules:manage", name: "Assignment Rules", desc: "Configure auto-assignment rules" },
    ]
  },
  {
    id: "payroll",
    name: "Payroll",
    icon: Banknote,
    permissions: [
      { id: "payroll:read", name: "View Payroll", desc: "Access payslip data" },
      { id: "payroll:calculate", name: "Calculate Payroll", desc: "Run payroll calculations" },
      { id: "payroll:settings:manage", name: "Payroll Settings", desc: "Configure payroll parameters" },
      { id: "payroll:components:manage", name: "Salary Components", desc: "Manage salary component definitions" },
      { id: "payroll:incentives:manage", name: "Incentive Policies", desc: "Configure incentive rules" },
      { id: "payroll:approve", name: "Approve Payroll", desc: "Approve calculated payroll batches" },
      { id: "payroll:disburse", name: "Disburse Payroll", desc: "Authorize payroll disbursement" },
      { id: "payroll:sensitive:read", name: "View Sensitive Data", desc: "Access base salary and confidential figures" },
    ]
  },
  {
    id: "holidays",
    name: "Holidays",
    icon: CalendarDays,
    permissions: [
      { id: "holidays:read", name: "View Holidays", desc: "See holiday calendar" },
      { id: "holidays:manage", name: "Manage Holidays", desc: "Create and edit holidays" },
    ]
  },
  {
    id: "reports",
    name: "Reports & Dashboard",
    icon: BarChart3,
    permissions: [
      { id: "reports:read", name: "View Reports", desc: "Access reports module" },
      { id: "reports:export", name: "Export Reports", desc: "Download report files" },
      { id: "dashboard:read", name: "View Dashboard", desc: "Access dashboard overview" },
      { id: "dashboard:preferences", name: "Dashboard Preferences", desc: "Customize dashboard layout" },
    ]
  },
  {
    id: "documents",
    name: "Documents",
    icon: FileText,
    permissions: [
      { id: "documents:medical:read", name: "Medical Documents", desc: "Access medical/health records" },
      { id: "documents:contract:read", name: "Contract Documents", desc: "Access employment contracts" },
    ]
  },
  {
    id: "security",
    name: "Security & Audit",
    icon: ShieldCheck,
    permissions: [
      { id: "audit_logs:read", name: "View Audit Logs", desc: "Access security event logs" },
      { id: "audit_logs:export", name: "Export Audit Logs", desc: "Download audit trail records" },
      { id: "security:settings:manage", name: "Security Settings", desc: "Configure security policies" },
    ]
  },
  {
    id: "infra",
    name: "Infrastructure",
    icon: Monitor,
    permissions: [
      { id: "agents:read", name: "View Agents", desc: "See registered agents" },
      { id: "agents:manage", name: "Manage Agents", desc: "Register and configure agents" },
      { id: "devices:read", name: "View Devices", desc: "See connected devices" },
      { id: "devices:manage", name: "Manage Devices", desc: "Add and configure devices" },
      { id: "feature_flags:read", name: "View Feature Flags", desc: "See feature toggle states" },
      { id: "feature_flags:manage", name: "Manage Feature Flags", desc: "Toggle feature flags" },
      { id: "infra:monitor", name: "Infrastructure Monitor", desc: "Access worker and job monitoring" },
    ]
  },
  {
    id: "recruitment",
    name: "Recruitment",
    icon: ClipboardList,
    permissions: [
      { id: "recruitment:read", name: "View Recruitment", desc: "Access recruitment pipeline" },
      { id: "recruitment:manage", name: "Manage Recruitment", desc: "Create and manage job postings" },
    ]
  },
  {
    id: "employment_phases",
    name: "Employment Phases",
    icon: FileText,
    permissions: [
      { id: "employment_phases:read", name: "View Employment Phases", desc: "See employee phases" },
      { id: "employment_phases:manage", name: "Manage Employment Phases", desc: "Configure phases and probation" },
    ]
  },
  {
    id: "ess",
    name: "Employee Self-Service",
    icon: Smartphone,
    permissions: [
      { id: "ess:profile", name: "ESS Profile", desc: "View and edit own profile" },
      { id: "ess:attendance", name: "ESS Attendance", desc: "View own attendance records" },
      { id: "ess:clock", name: "ESS Clock In/Out", desc: "Clock in and out" },
      { id: "ess:leave", name: "ESS Leave", desc: "Submit and view own leave" },
      { id: "ess:payslips", name: "ESS Payslips", desc: "View own payslips" },
      { id: "ess:shift_swaps", name: "ESS Shift Swaps", desc: "Request shift swaps" },
    ]
  },
];
