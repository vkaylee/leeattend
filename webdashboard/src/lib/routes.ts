/**
 * Centralized route definitions — single source of truth for all app paths.
 *
 * Usage:
 *   import { ROUTES } from "@/lib/routes";
 *   <Link to={ROUTES.EMPLOYEES}>...</Link>
 *   navigate(ROUTES.SETTINGS.CONNECTIVITY);
 *
 * Route paths are defined by TanStack Router file-based routes in src/routes/.
 * This file provides typed constants for type-safe navigation.
 */
export const ROUTES = {
  // Auth
  LOGIN: "/login",
  REGISTER: "/register",
  ONBOARDING: "/onboarding",

  // Main
  HOME: "/",
  EMPLOYEES: "/employees",
  DEPARTMENTS: "/departments",
  PEOPLE: "/workspace/people",

  // Attendance & Leave
  ATTENDANCE_LOGS: "/attendance-logs",
  LEAVE_REQUESTS: "/leave-requests",
  LEAVE_BALANCES: "/leave-balances",
  ATTENDANCE_ADJUSTMENTS: "/attendance-adjustments",
  APPROVALS_OT: "/approvals/ot",

  // Operations
  REPORTS: "/reports",
  SHIFTS: "/shifts",
  SHIFT_SWAPS: "/shift-swaps",
  PAYROLL: "/payroll",
  AGENTS: "/agents",
  DEVICES: "/devices",
  PROFILE: "/profile",

  // Working patterns
  WORKING_PATTERNS: "/working-patterns",
  ASSIGNMENT_RULES: "/assignment-rules",

  // Infra & System
  INFRA_MONITOR: "/infra/monitor",
  SYSTEM_ROLES: "/system/roles",

  // Employee Self-Service
  ESS_PROFILE: "/ess/profile",
  ESS_ATTENDANCE: "/ess/attendance",
  ESS_CLOCK: "/ess/clock",

  // Workspace Settings (Unified Control Panel)
  SETTINGS: {
    ROOT: "/workspace/settings",
    OVERVIEW: "/workspace/settings/overview",
    ROLES: "/workspace/settings/roles",
    PERMISSIONS: "/workspace/settings/permissions",
    GENERAL: "/workspace/settings/general",
    ORGANIZATION: "/workspace/settings/organization",
    TIME: "/workspace/settings/time",
    PAYROLL: "/workspace/settings/payroll",
    LEAVE: "/workspace/settings/leave",
    HOLIDAYS: "/workspace/settings/holidays",
    CONNECTIVITY: "/workspace/settings/connectivity",
    AUDIT_LOGS: "/workspace/settings/audit-logs",
    BILLING: "/workspace/settings/billing",
    DANGER: "/workspace/settings/danger",
  },
} as const;
