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
  PEOPLE: "/workspace/people",
  MEMBERS: "/members",
  NOTIFICATIONS: "/notifications",

  // Attendance & Leave
  ATTENDANCE_LOGS: "/attendance-logs",
  LEAVE_REQUESTS: "/leave-requests",
  LEAVE_BALANCES: "/leave-balances",
  ATTENDANCE_ADJUSTMENTS: "/attendance-adjustments",
  APPROVALS_OT: "/approvals/ot",

  // Operations
  REPORTS: "/reports",
  SHIFTS: "/shifts",
  SHIFTS_NEW: "/shifts/new",
  SHIFTS_EDIT: "/shifts/$shiftId/edit",
  SHIFT_SWAPS: "/shift-swaps",
  PAYROLL: "/payroll",
  PROFILE: "/profile",

  // Working patterns & Rules (unified path)
  SHIFTS_RULES: "/shifts/rules",
  WORKING_PATTERNS: "/shifts/rules",
  ASSIGNMENT_RULES: "/shifts/rules",

  // Infra & System
  INFRA_MONITOR: "/infra/monitor",
  SYSTEM_SMTP: "/system/smtp",

  // Employee Self-Service
  ESS_PROFILE: "/ess/profile",
  ESS_ATTENDANCE: "/ess/attendance",
  ESS_CLOCK: "/ess/clock",
  ESS_OT: "/ess/ot",
  ESS_DOCUMENTS: "/ess/documents",

  // Workspace Settings (Unified Control Panel)
  SETTINGS: {
    ROOT: "/workspace/settings",
    ROLES: "/workspace/settings/roles",
    PERMISSIONS: "/workspace/settings/permissions",
    GENERAL: "/workspace/settings/general",
    ORGANIZATION: "/workspace/settings/organization",
    TIME: "/workspace/settings/time",
    PAYROLL: "/workspace/settings/payroll",
    LEAVE: "/workspace/settings/leave",
    HOLIDAYS: "/workspace/settings/holidays",
    AUDIT_LOGS: "/workspace/settings/audit-logs",
    BILLING: "/workspace/settings/billing",
    DANGER: "/workspace/settings/danger",
    OVERTIME: "/workspace/settings/overtime",
    LOCATIONS: "/workspace/settings/locations",
    WORKFLOWS: "/workspace/settings/workflows",
    CUSTOM_FIELDS: "/workspace/settings/custom-fields",
    LIFECYCLE_TEMPLATES: "/workspace/settings/lifecycle-templates",
    CONDITION_TYPES: "/workspace/settings/condition-types",
    STORAGE: "/workspace/settings/storage",
    ECONTRACTS: "/workspace/settings/econtracts",
  },
} as const;
