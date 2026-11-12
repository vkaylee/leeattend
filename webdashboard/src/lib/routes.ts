/**
 * Centralized route definitions — single source of truth for all app paths.
 *
 * Usage:
 *   import { ROUTES } from "@/lib/routes";
 *   <Link to={ROUTES.EMPLOYEES}>...</Link>
 *   navigate(ROUTES.SETTINGS.GENERAL);
 *
 * Route paths are defined by TanStack Router file-based routes in src/routes/.
 * This file provides typed constants for type-safe navigation.
 */
export const ROUTES = {
  // Auth
  LOGIN: "/login",
  REGISTER: "/register",
  FORGOT_PASSWORD: "/forgot-password",
  RESET_PASSWORD: "/reset-password",
  ONBOARDING: "/onboarding",

  // Main
  HOME: "/",
  DASHBOARD: "/dashboard",
  EMPLOYEES: "/employees",
  DEPARTMENTS: "/departments",
  ORG_CHART: "/org-chart",
  ECONTRACTS: "/econtracts",
  ASSIGNMENTS: "/assignments",
  WORKING_CONDITIONS: "/working-conditions",
  LIFECYCLE: "/lifecycle",
  EMPLOYMENT_PHASES: "/employment-phases",
  ATTENDANCE_POLICY_ASSIGNMENTS: "/attendance-policy-assignments",
  NOTIFICATIONS: "/notifications",

  // Attendance & Leave
  ATTENDANCE_LOGS: "/attendance-logs",
  LEAVE_REQUESTS: "/leave-requests",
  LEAVE_BALANCES: "/leave-balances",
  ATTENDANCE_ADJUSTMENTS: "/attendance-adjustments",
  APPROVALS: "/approvals",
  APPROVALS_OT: "/approvals/ot",
  HOLIDAYS: "/holidays",

  // Operations
  REPORTS: "/reports",
  SHIFTS: "/shifts",
  SHIFTS_NEW: "/shifts/new",
  SHIFTS_EDIT: "/shifts/$shiftId/edit",
  SHIFT_SWAPS: "/shift-swaps",
  PAYROLL: "/payroll",
  SALARY_MANAGEMENT: "/salary-management",
  DOCUMENTS: "/documents",
  PROFILE: "/profile",

  // Working patterns & Rules (unified path)
  SHIFTS_RULES: "/shifts/rules",
  WORKING_PATTERNS: "/shifts/rules",
  ASSIGNMENT_RULES: "/shifts/rules",

  // Infra & System
  INFRA_MONITOR: "/infra/monitor",
  SYSTEM_SMTP: "/system/smtp",
  AUDIT_LOGS: "/audit-logs",
  // Employee Self-Service
  ESS_PROFILE: "/ess/profile",
  ESS_ATTENDANCE: "/ess/attendance",
  ESS_CLOCK: "/ess/clock",
  ESS_OT: "/ess/ot",
  ESS_DOCUMENTS: "/ess/documents",

  // Organization Settings (Unified Control Panel)
  SETTINGS: {
    ROOT: "/settings",
    ROLES: "/settings/roles",
    PERMISSIONS: "/settings/permissions",
    MEMBERS: "/settings/members",
    GENERAL: "/settings/general",
    ORGANIZATION: "/settings/organization",
    TIME: "/settings/time",
    PAYROLL: "/settings/payroll",
    LEAVE: "/settings/leave",
    BILLING: "/settings/billing",
    OVERTIME: "/settings/overtime",
    LOCATIONS: "/settings/locations",
    WORKFLOWS: "/settings/workflows",
    CUSTOM_FIELDS: "/settings/custom-fields",
    LIFECYCLE_TEMPLATES: "/settings/lifecycle-templates",
    CONDITION_TYPES: "/settings/condition-types",
    STORAGE: "/settings/storage",
    ECONTRACTS: "/settings/econtracts",
    SMTP: "/settings/smtp",
  },
} as const;
