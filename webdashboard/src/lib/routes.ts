/**
 * Centralized route definitions — single source of truth for all app paths.
 *
 * Usage:
 *   import { ROUTES } from "@/lib/routes";
 *   <Link to={ROUTES.EMPLOYEES}>...</Link>
 *   navigate(ROUTES.SETTINGS.CONNECTIVITY);
 *
 * Note: AppRoutes.tsx uses React Router relative segments and does NOT
 *       consume these constants (changing to full paths would break nesting).
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
  MEMBERS: "/members",

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

  // Infra
  INFRA_MONITOR: "/infra/monitor",

  // Employee Self-Service
  ESS_PROFILE: "/ess/profile",
  ESS_ATTENDANCE: "/ess/attendance",
  ESS_CLOCK: "/ess/clock",

  // Workspace Settings
  SETTINGS: {
    ROOT: "/workspace/settings",
    GENERAL: "/workspace/settings/general",
    PEOPLE: "/workspace/settings/people",
    TIME: "/workspace/settings/time",
    PAYROLL: "/workspace/settings/payroll",
    LEAVE: "/workspace/settings/leave",
    HOLIDAYS: "/workspace/settings/holidays",
    CONNECTIVITY: "/workspace/settings/connectivity",
    AUDIT_LOGS: "/workspace/settings/audit-logs",
  },
} as const;
